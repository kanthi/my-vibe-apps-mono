use crate::model::*;
use crate::snapshot::SqliteSnapshot;
use crate::util::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;

/// Reads agy (Antigravity CLI) activity from `conversation_summaries.db`: steps per conversation,
/// workspace, and parent/child links. It never selects `title`, `preview` or `raw_summary`. Token
/// counts live in undocumented protobuf blobs and are not read; agy records no quota locally.
pub struct AgyReader {
    root: PathBuf,
}

impl AgyReader {
    pub fn new(root: PathBuf) -> Self {
        AgyReader { root }
    }
    fn db(&self) -> PathBuf {
        self.root.join("conversation_summaries.db")
    }

    /// `["file:///Users/me/proj"]` -> "/Users/me/proj"
    fn project(json: &str) -> Option<String> {
        let arr: Vec<String> = serde_json::from_str(json).ok()?;
        let first = arr.first()?;
        let path = first.strip_prefix("file://")?;
        Some(percent_encoding::percent_decode_str(path).decode_utf8_lossy().to_string())
    }
}

impl UsageReader for AgyReader {
    fn tool(&self) -> ToolId {
        ToolId::Agy
    }
    fn is_installed(&self) -> bool {
        self.db().exists()
    }

    fn read(&self, since: DateTime<Utc>) -> Result<ReaderOutput, ReadError> {
        if !self.is_installed() {
            return Ok(ReaderOutput::default());
        }
        let snap = SqliteSnapshot::open(&self.db()).map_err(|e| ReadError(e.to_string()))?;
        let mut stmt = snap.conn.prepare(
            "SELECT conversation_id, step_count, last_modified_time, last_user_input_time, workspace_uris, parent_conversation_id
             FROM conversation_summaries",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<i64>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
            ))
        })?;
        let mut by_id: HashMap<String, Session> = HashMap::new();
        let mut parent_of: HashMap<String, String> = HashMap::new();
        for (id, steps, modified, input, ws, parent) in rows.flatten() {
            let Some(end) = modified.as_deref().and_then(parse_date) else { continue };
            // A zero date means "no user input recorded"; fall back to the last-modified time.
            let start = input
                .as_deref()
                .and_then(parse_date)
                .filter(|d| *d > DateTime::UNIX_EPOCH)
                .unwrap_or(end)
                .min(end);
            let parent = parent.filter(|p| !p.is_empty());
            if let Some(p) = &parent {
                parent_of.insert(id.clone(), p.clone());
            }
            let mut s = Session::new(ToolId::Agy, id.clone(), start, end);
            s.project = ws.as_deref().and_then(Self::project);
            s.is_subagent = parent.is_some();
            s.turns = steps;
            by_id.insert(id, s);
        }
        // Fold child conversations into their parent so subagent work is counted once.
        for (child, parent) in &parent_of {
            if !by_id.contains_key(parent) {
                continue;
            }
            if let Some(c) = by_id.remove(child) {
                let p = by_id.get_mut(parent).unwrap();
                p.turns = Some(p.turns.unwrap_or(0) + c.turns.unwrap_or(0));
                p.end = p.end.max(c.end);
            }
        }
        let mut sessions: Vec<Session> = by_id.into_values().filter(|s| s.end >= since).collect();
        sessions.sort_by_key(|s| s.start);
        Ok(ReaderOutput { sessions, quotas: vec![] })
    }
}
