mod common;
use chrono::{DateTime, FixedOffset, Utc};
use common::*;
use meter_core::accounts::*;
use meter_core::alerts::*;
use meter_core::dashboard::*;
use meter_core::insights::{self, Labels, NoteKind, PaceVerdict};
use meter_core::rollup::*;
use meter_core::{format, MenuSummary, Quota, ReaderOutput, Session, ToolId};
use std::collections::HashMap;
use tempfile::tempdir;

fn utc() -> FixedOffset {
    FixedOffset::east_opt(0).unwrap()
}

// ---- rollup

#[test]
fn rollup_groups_by_day_project_model_and_skips_subagents() {
    let mk = |id: &str, start: &str, input: i64, out: Option<i64>, turns: Option<i64>, sub: bool| {
        let mut s = Session::new(ToolId::Codex, id, d(start), d(start));
        s.model = Some("m".into());
        s.project = Some("/p".into());
        s.input_tokens = Some(input);
        s.output_tokens = out;
        s.turns = turns;
        s.is_subagent = sub;
        s
    };
    let days = days_from(
        &[
            mk("a", "2026-10-24T10:00:00Z", 100, Some(10), Some(2), false),
            mk("b", "2026-10-24T12:00:00Z", 50, Some(5), Some(1), false),
            mk("c", "2026-10-24T12:00:00Z", 999, None, None, true),
            mk("e", "2026-10-23T12:00:00Z", 7, None, None, false),
        ],
        utc(),
    );
    assert_eq!(days.len(), 2);
    let today = days.iter().find(|x| x.day == "2026-10-24").unwrap();
    assert_eq!((today.tokens(), today.turns, today.sessions), (165, 3, 2));
}

#[test]
fn rollup_store_keeps_old_days_and_replaces_window_without_double_counting() {
    let dir = tempdir().unwrap();
    let store = RollupStore::new(dir.path().join("rollup.json"));
    let day = |day: &str, t: i64| {
        let mut x = DayUsage::new(day, ToolId::ClaudeCode, "default", "/p", "m");
        x.input_tokens = t;
        x
    };
    store.merge(vec![day("2026-10-01", 5), day("2026-10-24", 10)], &[ToolId::ClaudeCode], "2026-10-01").unwrap();
    // The tool later cleans up Oct 1; a re-read returns only Oct 24 (now larger).
    let all = store.merge(vec![day("2026-10-24", 30)], &[ToolId::ClaudeCode], "2026-10-20").unwrap();
    assert_eq!(all.iter().map(|x| x.day.as_str()).collect::<Vec<_>>(), ["2026-10-01", "2026-10-24"]);
    assert_eq!(all.last().unwrap().tokens(), 30);
}

// ---- accounts

#[test]
fn accounts_add_hide_unhide_filter_and_effective_defaults() {
    let dir = tempdir().unwrap();
    let store = AccountStore::new(dir.path().join("accounts.json"));
    store.add(Account { id: "work".into(), tool: ToolId::Codex, label: "Work".into(), data_root: None, is_hidden: false }).unwrap();
    store.set_hidden(true, ToolId::Codex, "work").unwrap();
    assert_eq!(store.hidden_keys().into_iter().collect::<Vec<_>>(), ["codex/work"]);
    let mut out = ReaderOutput::default();
    for (id, acct) in [("1", "work"), ("2", "default")] {
        let mut s = Session::new(ToolId::Codex, id, d("2026-10-24T00:00:00Z"), d("2026-10-24T00:00:00Z"));
        s.account_id = acct.into();
        out.sessions.push(s);
    }
    let mut q = Quota::new(ToolId::Grok, "Weekly", 5.0, d("2026-10-24T00:00:00Z"));
    q.account_id = "default".into();
    out.quotas.push(q);
    let shown = out.excluding(&store.hidden_keys());
    assert_eq!(shown.sessions.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), ["2"]);
    assert_eq!(shown.quotas.len(), 1); // Grok's "default" is not hidden by Codex's "work"
    store.set_hidden(false, ToolId::Codex, "work").unwrap();
    assert!(store.hidden_keys().is_empty());
    assert_eq!(store.effective().len(), 6); // 1 stored + a built-in default for each of 5 tools
    store.set_hidden(true, ToolId::Grok, "default").unwrap();
    assert_eq!(store.hidden_keys().into_iter().collect::<Vec<_>>(), ["grok/default"]);
    assert_eq!(store.effective().len(), 6);
    store.remove(ToolId::Codex, "work").unwrap();
    assert_eq!(store.effective().len(), 5);
}

// ---- insights

fn weekly(used: f64, at: &str, tool: ToolId) -> Quota {
    let mut q = Quota::new(tool, "Weekly", used, d(at));
    q.window_minutes = Some(7 * 1440);
    q.resets_at = Some(d("2026-10-28T00:00:00Z"));
    q
}

#[test]
fn pace_verdicts() {
    let at = "2026-10-24T12:00:00Z"; // day 3.5 of 7 = halfway
    assert_eq!(insights::pace(&weekly(40.0, at, ToolId::Codex)), PaceVerdict::OnCourse { projected_percent: 80.0 });
    assert!(matches!(insights::pace(&weekly(47.0, at, ToolId::Codex)), PaceVerdict::Tight { .. }));
    // 70% used at the halfway point: 100% is reached at 5 of 7 days.
    assert_eq!(
        insights::pace(&weekly(70.0, at, ToolId::Codex)),
        PaceVerdict::WillRunOut { at: d("2026-10-21T00:00:00Z") + chrono::Duration::milliseconds((3.5 * 86_400_000.0 / 0.7) as i64) }
    );
    assert_eq!(insights::pace(&weekly(0.0, at, ToolId::Codex)), PaceVerdict::NotEnoughData);
    assert_eq!(insights::pace(&weekly(5.0, "2026-10-21T01:00:00Z", ToolId::Codex)), PaceVerdict::NotEnoughData);
}

#[test]
fn tightest_and_notes() {
    let at = "2026-10-24T12:00:00Z";
    let q = vec![weekly(82.0, at, ToolId::Codex), weekly(3.0, at, ToolId::Grok)];
    assert_eq!(insights::tightest(&q).unwrap().tool, ToolId::Codex);
    let notes = insights::notes(&q, &[], "2026-10-24", d(at), &Labels::new());
    let kinds: Vec<_> = notes.iter().map(|n| n.kind).collect();
    assert!(kinds.contains(&NoteKind::ClosestToCap) && kinds.contains(&NoteKind::Pace) && kinds.contains(&NoteKind::IdleQuota));
    assert!(insights::notes(&[weekly(20.0, at, ToolId::Codex)], &[], "2026-10-24", Utc::now(), &Labels::new()).is_empty());
}

#[test]
fn added_accounts_are_named_by_their_label() {
    let mut work = weekly(90.0, "2026-10-24T12:00:00Z", ToolId::Codex);
    work.account_id = "abc".into();
    let labels: Labels = [("codex/abc".to_string(), "Work".to_string())].into();
    let notes = insights::notes(&[work.clone()], &[], "2026-10-24", d("2026-10-24T12:00:00Z"), &labels);
    assert!(notes[0].text.starts_with("Codex · Work Weekly is closest"));
    assert_eq!(MenuSummary::build(&[work], &[], 3, &labels).lines, ["Codex · Work Weekly  90%"]);
}

#[test]
fn out_of_pattern() {
    let u = |day: &str, t: i64| {
        let mut x = DayUsage::new(day, ToolId::ClaudeCode, "default", "/w/app", "m");
        x.input_tokens = t;
        x
    };
    let history = vec![u("2026-10-20", 100_000), u("2026-10-21", 120_000), u("2026-10-22", 80_000)];
    let with = |t: i64| [history.clone(), vec![u("2026-10-24", t)]].concat();
    assert_eq!(insights::out_of_pattern(&with(500_000), "2026-10-24", 3.0, 50_000).len(), 1);
    assert!(insights::out_of_pattern(&with(150_000), "2026-10-24", 3.0, 50_000).is_empty());
    assert!(insights::out_of_pattern(&[u("2026-10-22", 1), u("2026-10-24", 900_000)], "2026-10-24", 3.0, 50_000).is_empty());
}

#[test]
fn menu_summary_is_minimal() {
    let at = "2026-10-24T12:00:00Z";
    let q = vec![
        weekly(10.0, at, ToolId::Grok),
        weekly(82.0, at, ToolId::Codex),
        Quota::new(ToolId::Codex, "Session", 40.0, d(at)),
        Quota::new(ToolId::Cursor, "Limit", 1.0, d(at)),
    ];
    let s = MenuSummary::build(&q, &[], 3, &Labels::new());
    assert_eq!(s.lines, ["Codex Weekly  82%", "Codex Session  40%", "Grok Weekly  10%"]);
    assert!(s.warning.is_none());
    assert_eq!(s.tightest_percent, Some(82.0));
}

// ---- alerts

fn q(used: f64, reset: &str, at: &str) -> Quota {
    let mut x = Quota::new(ToolId::Codex, "Weekly", used, d(at));
    x.window_minutes = Some(7 * 1440);
    x.resets_at = Some(d(reset));
    x
}
const RESET: &str = "2026-10-28T00:00:00Z";
const AT: &str = "2026-10-24T12:00:00Z";

#[test]
fn alerts_baseline_then_once_per_threshold() {
    let mut e = AlertEngine::new(vec![80.0, 95.0]);
    assert!(e.evaluate(&[q(85.0, RESET, AT)]).is_empty()); // already high at launch: baseline only
    assert!(e.evaluate(&[q(86.0, RESET, AT)]).is_empty());
    assert_eq!(e.evaluate(&[q(96.0, RESET, AT)]).iter().map(|a| a.kind).collect::<Vec<_>>(), [AlertKind::Threshold]);
    assert!(e.evaluate(&[q(97.0, RESET, AT)]).is_empty()); // no repeat
}

#[test]
fn alerts_exhausted_then_restored_on_new_cycle_and_ignore_reset_jitter() {
    let mut e = AlertEngine::new(vec![80.0, 95.0]);
    e.evaluate(&[q(50.0, RESET, AT)]);
    assert_eq!(e.evaluate(&[q(100.0, RESET, AT)]).iter().map(|a| a.kind).collect::<Vec<_>>(), [AlertKind::Exhausted]);
    assert!(e.evaluate(&[q(100.0, "2026-10-28T00:03:00Z", AT)]).is_empty()); // reset time jitter = same cycle
    let after = e.evaluate(&[q(2.0, "2026-11-04T00:00:00Z", "2026-10-28T01:00:00Z")]);
    assert_eq!(after.iter().map(|a| a.kind).collect::<Vec<_>>(), [AlertKind::Restored]);
}

#[test]
fn alerts_pace_warning_once() {
    let mut e = AlertEngine::new(vec![80.0, 95.0]);
    e.evaluate(&[q(10.0, RESET, AT)]);
    assert!(e.evaluate(&[q(70.0, RESET, AT)]).iter().any(|a| a.kind == AlertKind::Pace));
    assert!(!e.evaluate(&[q(71.0, RESET, AT)]).iter().any(|a| a.kind == AlertKind::Pace));
}

#[test]
fn alert_policy_defaults_off_and_honors_quiet_hours_and_snooze() {
    let now = d(AT);
    let alert = Alert { kind: AlertKind::Threshold, tool: ToolId::Codex, account_id: "default".into(), label: "Weekly".into(), title: "t".into(), body: "b".into() };
    let mut p = AlertPolicy::default();
    assert!(!p.allows(&alert, now, utc())); // all off by default
    p.enabled_kinds.insert("threshold".into());
    assert!(p.allows(&alert, now, utc()));
    p.quiet_start_minute = Some(22 * 60);
    p.quiet_end_minute = Some(8 * 60); // overnight wraps midnight
    assert!(p.allows(&alert, now, utc())); // 12:00 is outside quiet hours
    assert!(!p.allows(&alert, d("2026-10-24T23:30:00Z"), utc()));
    assert!(!p.allows(&alert, d("2026-10-24T07:00:00Z"), utc()));
    p.quiet_start_minute = None;
    p.snooze_until = Some(now + chrono::Duration::hours(1));
    assert!(!p.allows(&alert, now, utc()));
}

// ---- dashboard

fn runs_for_dashboard() -> (Vec<Account>, HashMap<String, AccountRun>, DateTime<Utc>) {
    let now = d(AT);
    let codex = Account::default_for(ToolId::Codex);
    let grok = Account { is_hidden: true, ..Account::default_for(ToolId::Grok) };
    let mut s = Session::new(ToolId::Codex, "s", d("2026-10-24T09:00:00Z"), now);
    s.model = Some("gpt".into());
    s.project = Some("/p".into());
    s.input_tokens = Some(900);
    s.output_tokens = Some(100);
    let mut ended = weekly(10.0, AT, ToolId::Codex);
    ended.resets_at = Some(d("2026-10-23T00:00:00Z"));
    let mut runs = HashMap::new();
    runs.insert("codex/default".into(), AccountRun { output: ReaderOutput { sessions: vec![s], quotas: vec![weekly(60.0, AT, ToolId::Codex), ended] }, last_success: Some(now), ..Default::default() });
    runs.insert("grok/default".into(), AccountRun { output: ReaderOutput { sessions: vec![], quotas: vec![weekly(99.0, AT, ToolId::Grok)] }, last_success: Some(now), ..Default::default() });
    (vec![codex, grok], runs, now)
}

#[test]
fn dashboard_separates_ended_windows_and_excludes_hidden_accounts() {
    let (accounts, runs, now) = runs_for_dashboard();
    let dash = Dashboard::build(&accounts, &runs, &[], utc(), now);
    let c = dash.accounts.iter().find(|a| a.account.tool == ToolId::Codex).unwrap();
    assert_eq!((c.tokens_today, c.top_model.as_deref(), c.quotas.len(), c.ended_windows.len()), (1000, Some("gpt"), 1, 1));
    assert_eq!(dash.visible_quotas.iter().map(|q| q.tool).collect::<Vec<_>>(), [ToolId::Codex]); // hidden Grok (99%) is out
    assert_eq!(dash.menu.lines, ["Codex Weekly  60%"]);
    assert!(dash.accounts.iter().find(|a| a.account.tool == ToolId::Grok).unwrap().account.is_hidden);
}

#[test]
fn dashboard_marks_stale_after_ten_minutes_and_not_installed() {
    let now = d(AT);
    let a = Account::default_for(ToolId::Codex);
    let mut runs = HashMap::new();
    runs.insert("codex/default".into(), AccountRun { last_success: Some(now - chrono::Duration::seconds(700)), error: Some("Could not read Codex data".into()), ..Default::default() });
    runs.insert("cursor/default".into(), AccountRun { installed: false, ..Default::default() });
    let dash = Dashboard::build(&[a, Account::default_for(ToolId::Cursor)], &runs, &[], utc(), now);
    assert!(dash.accounts[0].is_stale && dash.accounts[0].error.is_some());
    assert!(!dash.accounts[1].installed);
}

#[test]
fn dashboard_menu_falls_back_to_tokens_then_turns() {
    let now = d(AT);
    let mut k = Session::new(ToolId::ClaudeCode, "1", d("2026-10-24T09:00:00Z"), now);
    k.input_tokens = Some(90_000);
    k.output_tokens = Some(4_000);
    let mut u = Session::new(ToolId::Cursor, "2", d("2026-10-24T09:00:00Z"), now);
    u.turns = Some(83);
    let mut runs = HashMap::new();
    runs.insert("claudeCode/default".into(), AccountRun { output: ReaderOutput { sessions: vec![k], quotas: vec![] }, last_success: Some(now), ..Default::default() });
    runs.insert("cursor/default".into(), AccountRun { output: ReaderOutput { sessions: vec![u], quotas: vec![] }, last_success: Some(now), ..Default::default() });
    let both = Dashboard::build(&[Account::default_for(ToolId::ClaudeCode), Account::default_for(ToolId::Cursor)], &runs, &[], utc(), now);
    assert_eq!(both.menu.lines, ["Claude Code  94k tokens today"]);
    let only = Dashboard::build(&[Account::default_for(ToolId::Cursor)], &runs, &[], utc(), now);
    assert_eq!(only.menu.lines, ["Cursor  83 turns today"]);
}

#[test]
fn refresh_marks_missing_data_folder_as_not_installed() {
    let missing = Account { id: "x".into(), tool: ToolId::Codex, label: "Gone".into(), data_root: Some("/nonexistent/path/xyz".into()), is_hidden: false };
    let runs = refresh(&[missing], &HashMap::new(), d(AT), chrono::DateTime::UNIX_EPOCH);
    assert!(!runs["codex/x"].installed);
}

#[test]
fn refresh_reads_every_tool_from_the_fake_home() {
    let home = tempdir().unwrap();
    meter_core::fakehome::build(home.path(), Utc::now()).unwrap();
    let accounts: Vec<Account> = ToolId::ALL
        .iter()
        .map(|t| {
            let root = match t {
                ToolId::Codex => ".codex",
                ToolId::Grok => ".grok",
                ToolId::ClaudeCode => ".claude",
                ToolId::Cursor => "Library/Application Support/Cursor",
                ToolId::Agy => ".gemini/antigravity-cli",
            };
            Account { data_root: Some(home.path().join(root).to_string_lossy().to_string()), ..Account::default_for(*t) }
        })
        .collect();
    let now = Utc::now();
    let runs = refresh(&accounts, &HashMap::new(), now, now - chrono::Duration::days(30));
    let dash = Dashboard::build(&accounts, &runs, &[], utc(), now);
    assert!(dash.accounts.iter().all(|a| a.installed && a.error.is_none()));
    let codex = dash.accounts.iter().find(|a| a.account.tool == ToolId::Codex).unwrap();
    assert_eq!(codex.quotas.iter().map(|q| q.used_percent).collect::<Vec<_>>(), [34.0, 62.0]); // Session, Weekly
}

// ---- format

#[test]
fn formats() {
    assert_eq!((format::tokens(999).as_str(), format::tokens(1_234).as_str(), format::tokens(94_351).as_str()), ("999", "1.2k", "94k"));
    assert_eq!(format::tokens(8_616_973), "8.6M");
    let now = d(AT);
    assert_eq!(format::until(now + chrono::Duration::seconds(30), now), "now");
    assert_eq!(format::until(now + chrono::Duration::hours(2), now), "in 2 h");
    assert_eq!(format::until(now + chrono::Duration::days(3), now), "in 3 d");
    assert_eq!(format::base_name("/Users/me/app"), "app");
}
