use crate::format::{percent, until};
use crate::model::{Quota, ToolId};
use crate::rollup::DayUsage;
use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub enum PaceVerdict {
    NotEnoughData,
    OnCourse { projected_percent: f64 },
    Tight { projected_percent: f64 },
    WillRunOut { at: DateTime<Utc> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteKind {
    ClosestToCap,
    Pace,
    OutOfPattern,
    IdleQuota,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub kind: NoteKind,
    pub text: String,
}

pub type Labels = HashMap<String, String>;

/// "Codex", or "Codex · Work" for an added account, using the user's own label.
pub fn name(q: &Quota, labels: &Labels) -> String {
    if q.account_id == "default" {
        return q.tool.display_name().into();
    }
    let key = format!("{}/{}", q.tool.raw(), q.account_id);
    format!("{} · {}", q.tool.display_name(), labels.get(&key).cloned().unwrap_or_else(|| q.account_id.clone()))
}

/// The window with the least room left.
pub fn tightest(quotas: &[Quota]) -> Option<&Quota> {
    quotas.iter().max_by(|a, b| a.used_percent.total_cmp(&b.used_percent))
}

pub fn pace(q: &Quota) -> PaceVerdict {
    let (Some(reset), Some(mins)) = (q.resets_at, q.window_minutes) else { return PaceVerdict::NotEnoughData };
    if mins <= 0 {
        return PaceVerdict::NotEnoughData;
    }
    let window = (mins * 60) as f64;
    let start = reset - Duration::seconds(mins * 60);
    let elapsed = (q.observed_at - start).num_milliseconds() as f64 / 1000.0;
    if elapsed < window * 0.05 || elapsed >= window || q.used_percent <= 0.0 {
        return PaceVerdict::NotEnoughData;
    }
    if q.used_percent >= 100.0 {
        return PaceVerdict::WillRunOut { at: q.observed_at };
    }
    let projected = q.used_percent / elapsed * window;
    if projected >= 100.0 {
        let secs = elapsed * 100.0 / q.used_percent;
        return PaceVerdict::WillRunOut { at: start + Duration::milliseconds((secs * 1000.0) as i64) };
    }
    if projected > 90.0 {
        PaceVerdict::Tight { projected_percent: projected }
    } else {
        PaceVerdict::OnCourse { projected_percent: projected }
    }
}

/// Pure. A note exists only when it is true.
pub fn notes(quotas: &[Quota], days: &[DayUsage], today: &str, now: DateTime<Utc>, labels: &Labels) -> Vec<Note> {
    let mut out = vec![];
    if let Some(t) = tightest(quotas).filter(|t| t.used_percent >= 50.0) {
        out.push(Note {
            kind: NoteKind::ClosestToCap,
            text: format!("{} {} is closest to its cap: {} used.", name(t, labels), t.label, percent(t.used_percent)),
        });
    }
    for q in quotas {
        if let PaceVerdict::WillRunOut { at } = pace(q) {
            out.push(Note {
                kind: NoteKind::Pace,
                text: format!(
                    "At today's pace {} {} runs out {}, before it resets.",
                    name(q, labels),
                    q.label,
                    when(at, now)
                ),
            });
        }
    }
    out.extend(out_of_pattern(days, today, 3.0, 50_000));
    out.extend(idle_quota(quotas, 80.0, 10.0));
    out
}

/// A project whose tokens today are several times its recent daily average.
pub fn out_of_pattern(days: &[DayUsage], today: &str, factor: f64, min_tokens: i64) -> Vec<Note> {
    let mut today_tokens: HashMap<(ToolId, String), i64> = HashMap::new();
    let mut prior: HashMap<(ToolId, String), HashMap<String, i64>> = HashMap::new();
    for d in days.iter().filter(|d| !d.project.is_empty()) {
        let k = (d.tool, d.project.clone());
        if d.day == today {
            *today_tokens.entry(k).or_default() += d.tokens();
        } else if d.day.as_str() < today {
            *prior.entry(k).or_default().entry(d.day.clone()).or_default() += d.tokens();
        }
    }
    let mut out: Vec<Note> = today_tokens
        .into_iter()
        .filter_map(|(k, t)| {
            let daily: Vec<i64> = prior.get(&k)?.values().copied().filter(|v| *v > 0).collect();
            if daily.len() < 3 || t < min_tokens {
                return None;
            }
            let mean = daily.iter().sum::<i64>() as f64 / daily.len() as f64;
            if (t as f64) < factor * mean {
                return None;
            }
            let project = crate::format::base_name(&k.1);
            Some(Note {
                kind: NoteKind::OutOfPattern,
                text: format!(
                    "{} in {} used about {:.0}× its recent daily average today.",
                    k.0.display_name(),
                    project,
                    t as f64 / mean
                ),
            })
        })
        .collect();
    out.sort_by(|a, b| a.text.cmp(&b.text));
    out
}

/// One tool barely used while another is near its cap.
pub fn idle_quota(quotas: &[Quota], near_cap: f64, barely: f64) -> Option<Note> {
    let mut worst: HashMap<ToolId, f64> = HashMap::new();
    for q in quotas {
        let e = worst.entry(q.tool).or_insert(0.0);
        *e = e.max(q.used_percent);
    }
    let hot = worst.iter().filter(|(_, v)| **v >= near_cap).max_by(|a, b| a.1.total_cmp(b.1))?;
    let idle = worst.iter().filter(|(k, v)| *k != hot.0 && **v <= barely).min_by(|a, b| a.1.total_cmp(b.1))?;
    Some(Note {
        kind: NoteKind::IdleQuota,
        text: format!(
            "{} is barely used ({}) while {} is at {}.",
            idle.0.display_name(),
            percent(*idle.1),
            hot.0.display_name(),
            percent(*hot.1)
        ),
    })
}

pub fn when(at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let u = until(at, now);
    if u == "now" {
        u
    } else {
        u.replacen("in ", "in about ", 1)
    }
}

/// What the minimal menu shows: at most `limit` lines, tightest first, plus a warning only when one is true.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MenuSummary {
    pub lines: Vec<String>,
    pub warning: Option<String>,
    pub tightest_percent: Option<f64>,
}

impl MenuSummary {
    pub fn build(quotas: &[Quota], notes: &[Note], limit: usize, labels: &Labels) -> Self {
        let mut sorted: Vec<&Quota> = quotas.iter().collect();
        sorted.sort_by(|a, b| b.used_percent.total_cmp(&a.used_percent));
        let mut seen = HashSet::new();
        let lines = sorted
            .iter()
            .filter(|q| seen.insert(format!("{}/{}/{}", q.tool.raw(), q.account_id, q.label)))
            .take(limit)
            .map(|q| format!("{} {}  {}", name(q, labels), q.label, percent(q.used_percent)))
            .collect();
        let warning = notes
            .iter()
            .find(|n| matches!(n.kind, NoteKind::Pace | NoteKind::OutOfPattern))
            .map(|n| n.text.clone());
        MenuSummary { lines, warning, tightest_percent: sorted.first().map(|q| q.used_percent) }
    }
}
