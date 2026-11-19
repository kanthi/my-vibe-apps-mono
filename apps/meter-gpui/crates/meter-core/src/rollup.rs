use crate::model::{Session, ToolId};
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// One day of usage for a tool, account, project and model. Survives the tools' own log cleanup.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayUsage {
    pub day: String, // "yyyy-MM-dd" in the user's time zone
    pub tool: ToolId,
    pub account_id: String,
    pub project: String,
    pub model: String,
    #[serde(default)]
    pub input_tokens: i64,
    #[serde(default)]
    pub cached_tokens: i64,
    #[serde(default)]
    pub output_tokens: i64,
    #[serde(default)]
    pub turns: i64,
    #[serde(default)]
    pub sessions: i64,
}

impl DayUsage {
    pub fn new(day: &str, tool: ToolId, account_id: &str, project: &str, model: &str) -> Self {
        DayUsage {
            day: day.into(),
            tool,
            account_id: account_id.into(),
            project: project.into(),
            model: model.into(),
            input_tokens: 0,
            cached_tokens: 0,
            output_tokens: 0,
            turns: 0,
            sessions: 0,
        }
    }
    pub fn key(&self) -> String {
        format!("{}|{}|{}|{}|{}", self.day, self.tool.raw(), self.account_id, self.project, self.model)
    }
    pub fn tokens(&self) -> i64 {
        self.input_tokens + self.output_tokens
    }
}

pub fn day_key(date: DateTime<Utc>, tz: FixedOffset) -> String {
    date.with_timezone(&tz).format("%Y-%m-%d").to_string()
}

/// A session counts wholly toward the day it started. Subagent sessions are skipped.
pub fn days_from(sessions: &[Session], tz: FixedOffset) -> Vec<DayUsage> {
    let mut map: BTreeMap<String, DayUsage> = BTreeMap::new();
    for s in sessions.iter().filter(|s| !s.is_subagent) {
        let d = DayUsage::new(
            &day_key(s.start, tz),
            s.tool,
            &s.account_id,
            s.project.as_deref().unwrap_or(""),
            s.model.as_deref().unwrap_or(""),
        );
        let cur = map.entry(d.key()).or_insert(d);
        cur.input_tokens += s.input_tokens.unwrap_or(0);
        cur.cached_tokens += s.cached_tokens.unwrap_or(0);
        cur.output_tokens += s.output_tokens.unwrap_or(0);
        cur.turns += s.turns.unwrap_or(0);
        cur.sessions += 1;
    }
    map.into_values().collect()
}

/// JSON file holding all days seen so far. The path is injected so tests never use the real one.
pub struct RollupStore {
    pub path: PathBuf,
}

impl RollupStore {
    pub fn new(path: PathBuf) -> Self {
        RollupStore { path }
    }

    pub fn load(&self) -> Vec<DayUsage> {
        std::fs::read(&self.path).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
    }

    /// Replaces stored days for `tools` from `from_day` onward with fresh data, so a re-read never
    /// double counts. Older days (already cleaned up by the tool) are kept.
    pub fn merge(&self, fresh: Vec<DayUsage>, tools: &[ToolId], from_day: &str) -> std::io::Result<Vec<DayUsage>> {
        let mut kept: Vec<DayUsage> =
            self.load().into_iter().filter(|d| !(tools.contains(&d.tool) && d.day.as_str() >= from_day)).collect();
        kept.extend(fresh);
        kept.sort_by_key(|d| d.key());
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec(&kept)?)?;
        std::fs::rename(tmp, &self.path)?;
        Ok(kept)
    }
}
