use crate::model::*;
use crate::readers::window_label;
use crate::snapshot::SqliteSnapshot;
use crate::util::*;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::path::PathBuf;

/// Reads Codex's local records. Selects only non-text columns from the thread table, never
/// `title` or `first_user_message`, and never opens credentials.
pub struct CodexReader {
    root: PathBuf,
}

impl CodexReader {
    pub fn new(root: PathBuf) -> Self {
        CodexReader { root }
    }

    fn sessions(&self, since: DateTime<Utc>) -> Result<Vec<Session>, ReadError> {
        let db = self.root.join("state_5.sqlite");
        if !db.exists() {
            return Ok(vec![]);
        }
        let snap = SqliteSnapshot::open(&db).map_err(|e| ReadError(e.to_string()))?;
        let mut stmt = snap.conn.prepare(
            "SELECT id, created_at, updated_at, source, cwd, model, tokens_used FROM threads WHERE updated_at >= ?1",
        )?;
        let rows = stmt.query_map([since.timestamp()], |r| {
            let source: Option<String> = r.get(3)?;
            let mut s = Session::new(
                ToolId::Codex,
                r.get::<_, String>(0)?,
                from_epoch(r.get(1)?),
                from_epoch(r.get(2)?),
            );
            s.project = r.get(4)?;
            s.model = r.get(5)?;
            // `tokens_used` is one total; Codex does not split it here.
            s.input_tokens = r.get::<_, Option<i64>>(6)?;
            s.is_subagent = source.unwrap_or_default().contains("\"subagent\"");
            Ok(s)
        })?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    /// The newest `rate_limits` seen in the newest rollout files.
    fn latest_quotas(&self) -> Vec<Quota> {
        let mut files = walk_files(&self.root.join("sessions"), &|p| {
            p.extension().is_some_and(|e| e == "jsonl")
                && p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("rollout-"))
        });
        files.sort_by(|a, b| b.cmp(a));
        for file in files.into_iter().take(5) {
            let mut best: Option<(DateTime<Utc>, Value)> = None;
            for obj in json_lines(&file, &["\"rate_limits\""]) {
                let payload = &obj["payload"];
                if payload["type"] != "token_count" {
                    continue;
                }
                let Some(ts) = obj["timestamp"].as_str().and_then(parse_date) else { continue };
                if best.as_ref().map_or(true, |(t, _)| ts >= *t) {
                    best = Some((ts, payload["rate_limits"].clone()));
                }
            }
            let Some((observed, limits)) = best else { continue };
            return ["primary", "secondary"]
                .iter()
                .filter_map(|k| {
                    let w = limits.get(*k)?;
                    let used = w["used_percent"].as_f64()?;
                    let minutes = w["window_minutes"].as_i64();
                    let mut q = Quota::new(ToolId::Codex, window_label(minutes), used, observed);
                    q.window_minutes = minutes;
                    q.resets_at = w["resets_at"].as_f64().map(|s| from_epoch(s as i64));
                    Some(q)
                })
                .collect();
        }
        vec![]
    }
}

impl UsageReader for CodexReader {
    fn tool(&self) -> ToolId {
        ToolId::Codex
    }
    fn is_installed(&self) -> bool {
        self.root.exists()
    }
    fn read(&self, since: DateTime<Utc>) -> Result<ReaderOutput, ReadError> {
        Ok(ReaderOutput { sessions: self.sessions(since)?, quotas: self.latest_quotas() })
    }
}
