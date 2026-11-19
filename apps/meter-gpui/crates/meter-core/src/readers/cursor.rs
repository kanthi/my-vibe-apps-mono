use crate::model::*;
use crate::snapshot::SqliteSnapshot;
use crate::util::*;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Reads Cursor's composer records from `state.vscdb`. Values hold the conversation, so the query
/// extracts four scalar fields with `json_extract`; no message text is selected. Cursor records no
/// tokens or quota locally (its token fields are zero), so only turns are reported. Grok Bot shows
/// up as the model name on these records.
pub struct CursorReader {
    support: PathBuf,
}

impl CursorReader {
    pub fn new(support: PathBuf) -> Self {
        CursorReader { support }
    }
    fn db(&self) -> PathBuf {
        self.support.join("User/globalStorage/state.vscdb")
    }
}

impl UsageReader for CursorReader {
    fn tool(&self) -> ToolId {
        ToolId::Cursor
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
            "SELECT key,
                    json_extract(value, '$.createdAt'),
                    json_extract(value, '$.lastUpdatedAt'),
                    json_extract(value, '$.modelConfig.modelName'),
                    json_array_length(value, '$.fullConversationHeadersOnly')
             FROM cursorDiskKV WHERE key LIKE 'composerData:%'",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<i64>>(1)?,
                r.get::<_, Option<i64>>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<i64>>(4)?,
            ))
        })?;
        let mut sessions = vec![];
        for (key, created, updated, model, turns) in rows.flatten() {
            let Some(created) = created.map(from_epoch_millis) else { continue };
            let updated = updated.map(from_epoch_millis).unwrap_or(created);
            let turns = turns.unwrap_or(0);
            if turns == 0 || updated < since {
                continue; // empty drafts are not usage
            }
            let mut s = Session::new(ToolId::Cursor, key.replace("composerData:", ""), created, updated);
            s.model = model;
            s.turns = Some(turns);
            sessions.push(s);
        }
        sessions.sort_by_key(|s| s.start);
        Ok(ReaderOutput { sessions, quotas: vec![] })
    }
}
