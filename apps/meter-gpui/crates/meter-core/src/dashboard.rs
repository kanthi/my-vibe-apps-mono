use crate::accounts::Account;
use crate::format;
use crate::insights::{self, MenuSummary, Note};
use crate::model::{Quota, ReaderOutput};
use crate::readers;
use crate::rollup::{day_key, DayUsage};
use chrono::{DateTime, FixedOffset, Utc};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// The last result for one account. A failed refresh keeps the previous output.
#[derive(Clone, Debug)]
pub struct AccountRun {
    pub installed: bool,
    pub output: ReaderOutput,
    pub last_success: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl Default for AccountRun {
    fn default() -> Self {
        AccountRun { installed: true, output: ReaderOutput::default(), last_success: None, error: None }
    }
}

/// Reads every account concurrently. On failure the last good output stays and the error is recorded.
pub fn refresh(
    accounts: &[Account],
    previous: &HashMap<String, AccountRun>,
    now: DateTime<Utc>,
    since: DateTime<Utc>,
) -> HashMap<String, AccountRun> {
    std::thread::scope(|scope| {
        let handles: Vec<_> = accounts
            .iter()
            .map(|account| {
                let prior = previous.get(&account.key()).cloned();
                scope.spawn(move || {
                    let reader = readers::make(account.tool, account.data_root.as_ref().map(PathBuf::from));
                    if !reader.is_installed() {
                        return (account.key(), AccountRun { installed: false, ..Default::default() });
                    }
                    match reader.read(since) {
                        Ok(out) => (
                            account.key(),
                            AccountRun { installed: true, output: out.tagged(&account.id), last_success: Some(now), error: None },
                        ),
                        Err(_) => {
                            let mut kept = prior.unwrap_or_default();
                            kept.installed = true;
                            kept.error = Some(format!("Could not read {} data", account.tool.display_name()));
                            (account.key(), kept)
                        }
                    }
                })
            })
            .collect();
        handles.into_iter().filter_map(|h| h.join().ok()).collect()
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct AccountStatus {
    pub id: String,
    pub account: Account,
    pub installed: bool,
    /// Current windows only.
    pub quotas: Vec<Quota>,
    /// Windows whose reset time has passed.
    pub ended_windows: Vec<Quota>,
    pub tokens_today: i64,
    pub turns_today: i64,
    pub sessions_today: usize,
    pub top_model: Option<String>,
    pub top_project: Option<String>,
    pub last_success: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub is_stale: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Dashboard {
    pub accounts: Vec<AccountStatus>,
    pub notes: Vec<Note>,
    pub menu: MenuSummary,
    pub days: Vec<DayUsage>,
    pub visible_quotas: Vec<Quota>,
}

impl Dashboard {
    pub fn build(
        accounts: &[Account],
        runs: &HashMap<String, AccountRun>,
        days: &[DayUsage],
        tz: FixedOffset,
        now: DateTime<Utc>,
    ) -> Dashboard {
        let today = day_key(now, tz);
        let visible: HashSet<String> = accounts.iter().filter(|a| !a.is_hidden).map(|a| a.key()).collect();
        let mut statuses = vec![];
        for a in accounts {
            let run = runs.get(&a.key()).cloned().unwrap_or_default();
            let todays: Vec<_> = run
                .output
                .sessions
                .iter()
                .filter(|s| day_key(s.start, tz) == today && !s.is_subagent)
                .collect();
            let top = |f: &dyn Fn(&crate::model::Session) -> Option<String>| -> Option<String> {
                let mut w: HashMap<String, i64> = HashMap::new();
                for s in &todays {
                    if let Some(k) = f(s).filter(|k| !k.is_empty()) {
                        *w.entry(k).or_default() +=
                            s.input_tokens.unwrap_or(0) + s.output_tokens.unwrap_or(0) + s.turns.unwrap_or(0);
                    }
                }
                w.into_iter().max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0))).map(|(k, _)| k)
            };
            let mut quotas: Vec<Quota> = run.output.quotas.iter().filter(|q| q.is_current(now)).cloned().collect();
            quotas.sort_by(|a, b| a.label.cmp(&b.label));
            statuses.push(AccountStatus {
                id: a.key(),
                account: a.clone(),
                installed: run.installed,
                quotas,
                ended_windows: run.output.quotas.iter().filter(|q| !q.is_current(now)).cloned().collect(),
                tokens_today: todays.iter().map(|s| s.input_tokens.unwrap_or(0) + s.output_tokens.unwrap_or(0)).sum(),
                turns_today: todays.iter().map(|s| s.turns.unwrap_or(0)).sum(),
                sessions_today: todays.len(),
                top_model: top(&|s| s.model.clone()),
                top_project: top(&|s| s.project.clone()),
                last_success: run.last_success,
                error: run.error.clone(),
                is_stale: run.last_success.is_some_and(|t| (now - t).num_seconds() > 600),
            });
        }
        let quotas: Vec<Quota> =
            statuses.iter().filter(|s| visible.contains(&s.id) && s.installed).flat_map(|s| s.quotas.clone()).collect();
        let visible_days: Vec<DayUsage> =
            days.iter().filter(|d| visible.contains(&format!("{}/{}", d.tool.raw(), d.account_id))).cloned().collect();
        let labels: insights::Labels = accounts.iter().map(|a| (a.key(), a.label.clone())).collect();
        let notes = insights::notes(&quotas, &visible_days, &today, now, &labels);
        let mut menu = MenuSummary::build(&quotas, &notes, 3, &labels);
        if menu.lines.is_empty() {
            menu.lines = fallback_lines(&statuses);
        }
        Dashboard { accounts: statuses, notes, menu, days: visible_days, visible_quotas: quotas }
    }
}

/// When no tool records a quota, the menu falls back to tokens (or turns) today.
fn fallback_lines(statuses: &[AccountStatus]) -> Vec<String> {
    let live: Vec<&AccountStatus> = statuses.iter().filter(|s| !s.account.is_hidden && s.installed).collect();
    let mut by_tokens: Vec<&&AccountStatus> = live.iter().filter(|s| s.tokens_today > 0).collect();
    by_tokens.sort_by(|a, b| b.tokens_today.cmp(&a.tokens_today));
    if !by_tokens.is_empty() {
        return by_tokens
            .iter()
            .take(3)
            .map(|s| format!("{}  {} tokens today", s.account.tool.display_name(), format::tokens(s.tokens_today)))
            .collect();
    }
    let mut by_turns: Vec<&&AccountStatus> = live.iter().filter(|s| s.turns_today > 0).collect();
    by_turns.sort_by(|a, b| b.turns_today.cmp(&a.turns_today));
    by_turns.iter().take(3).map(|s| format!("{}  {} turns today", s.account.tool.display_name(), s.turns_today)).collect()
}
