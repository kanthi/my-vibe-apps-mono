use crate::insights::{pace, when, PaceVerdict};
use crate::format::percent;
use crate::model::{Quota, ToolId};
use chrono::{DateTime, FixedOffset, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AlertKind {
    Threshold,
    Exhausted,
    Restored,
    Pace,
}

impl AlertKind {
    pub const ALL: [AlertKind; 4] = [AlertKind::Threshold, AlertKind::Exhausted, AlertKind::Restored, AlertKind::Pace];
    pub fn raw(self) -> &'static str {
        match self {
            AlertKind::Threshold => "threshold",
            AlertKind::Exhausted => "exhausted",
            AlertKind::Restored => "restored",
            AlertKind::Pace => "pace",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Alert {
    pub kind: AlertKind,
    pub tool: ToolId,
    pub account_id: String,
    pub label: String,
    pub title: String,
    pub body: String,
}

/// Alert delivery rules the user controls: which kinds, quiet hours, snooze. All off by default.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertPolicy {
    pub enabled_kinds: HashSet<String>,
    /// Minutes after midnight.
    pub quiet_start_minute: Option<u32>,
    pub quiet_end_minute: Option<u32>,
    pub snooze_until: Option<DateTime<Utc>>,
}

impl AlertPolicy {
    pub fn allows(&self, alert: &Alert, now: DateTime<Utc>, tz: FixedOffset) -> bool {
        if !self.enabled_kinds.contains(alert.kind.raw()) {
            return false;
        }
        if self.snooze_until.is_some_and(|s| now < s) {
            return false;
        }
        if let (Some(qs), Some(qe)) = (self.quiet_start_minute, self.quiet_end_minute) {
            if qs != qe {
                let local = now.with_timezone(&tz);
                let m = local.hour() * 60 + local.minute();
                // An overnight window (22:00 to 08:00) wraps midnight.
                let in_quiet = if qs < qe { m >= qs && m < qe } else { m >= qs || m < qe };
                if in_quiet {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Clone)]
struct Entry {
    resets_at: Option<DateTime<Utc>>,
    fired: HashSet<String>,
    exhausted: bool,
}

/// Decides which alerts fire. Each fires once per account, window and reset cycle. The first
/// evaluation only records a baseline, so a quota already high or over pace at launch stays quiet.
pub struct AlertEngine {
    pub thresholds: Vec<f64>,
    state: HashMap<String, Entry>,
    baselined: bool,
}

impl AlertEngine {
    pub fn new(mut thresholds: Vec<f64>) -> Self {
        thresholds.sort_by(f64::total_cmp);
        AlertEngine { thresholds, state: HashMap::new(), baselined: false }
    }

    pub fn evaluate(&mut self, quotas: &[Quota]) -> Vec<Alert> {
        let mut alerts = vec![];
        for q in quotas {
            let key = format!("{}/{}/{}", q.tool.raw(), q.account_id, q.label);
            let fresh = Entry { resets_at: q.resets_at, fired: HashSet::new(), exhausted: false };
            let mut e = self.state.get(&key).cloned().unwrap_or_else(|| fresh.clone());
            if self.state.contains_key(&key) && !same_cycle(&e, q) {
                if e.exhausted && self.baselined {
                    alerts.push(make(AlertKind::Restored, q, format!("{} {} has reset", q.tool.display_name(), q.label), "Room is available again."));
                }
                e = fresh;
            }
            let levels: Vec<f64> = self.thresholds.iter().copied().filter(|t| q.used_percent >= *t).collect();
            let mut current: HashSet<String> = levels.iter().map(|l| format!("t{}", *l as i64)).collect();
            if q.used_percent >= 100.0 {
                current.insert("x".into());
            }
            if let Some(top) = levels.last() {
                let k = format!("t{}", *top as i64);
                if self.baselined && !e.fired.contains(&k) && q.used_percent < 100.0 {
                    alerts.push(make(
                        AlertKind::Threshold,
                        q,
                        format!("{} {} at {}", q.tool.display_name(), q.label, percent(q.used_percent)),
                        format!("Crossed {}% of this window.", *top as i64),
                    ));
                }
            }
            if self.baselined && q.used_percent >= 100.0 && !e.exhausted {
                alerts.push(make(AlertKind::Exhausted, q, format!("{} {} is used up", q.tool.display_name(), q.label), "The window is at 100%."));
            }
            let verdict = pace(q);
            if self.baselined && q.used_percent < 100.0 && !e.fired.contains("p") {
                if let PaceVerdict::WillRunOut { at } = &verdict {
                    alerts.push(make(
                        AlertKind::Pace,
                        q,
                        format!("{} {} may run out", q.tool.display_name(), q.label),
                        format!("At this pace it runs out {}.", when(*at, q.observed_at)),
                    ));
                    e.fired.insert("p".into());
                }
            }
            e.fired.extend(current);
            // Already on course to run out at launch: part of the baseline, not news.
            if !self.baselined && matches!(verdict, PaceVerdict::WillRunOut { .. }) {
                e.fired.insert("p".into());
            }
            e.exhausted = q.used_percent >= 100.0;
            e.resets_at = q.resets_at;
            self.state.insert(key, e);
        }
        self.baselined = true;
        alerts
    }
}

/// A slightly shifted reset time is still the same cycle.
fn same_cycle(e: &Entry, q: &Quota) -> bool {
    match (e.resets_at, q.resets_at) {
        (Some(a), Some(b)) => {
            let tolerance = ((q.window_minutes.unwrap_or(0) * 60) as f64 / 2.0).max(300.0);
            ((a - b).num_seconds().abs() as f64) < tolerance
        }
        (None, None) => true,
        _ => false,
    }
}

fn make(kind: AlertKind, q: &Quota, title: String, body: impl Into<String>) -> Alert {
    Alert { kind, tool: q.tool, account_id: q.account_id.clone(), label: q.label.clone(), title, body: body.into() }
}
