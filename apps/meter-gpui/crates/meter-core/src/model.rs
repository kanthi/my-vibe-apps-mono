use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolId {
    Codex,
    Grok,
    ClaudeCode,
    Cursor,
    Agy,
}

impl ToolId {
    pub const ALL: [ToolId; 5] = [ToolId::Codex, ToolId::Grok, ToolId::ClaudeCode, ToolId::Cursor, ToolId::Agy];

    pub fn display_name(self) -> &'static str {
        match self {
            ToolId::Codex => "Codex",
            ToolId::Grok => "Grok",
            ToolId::ClaudeCode => "Claude Code",
            ToolId::Cursor => "Cursor",
            ToolId::Agy => "agy",
        }
    }

    /// Stable key used in account keys and stored files.
    pub fn raw(self) -> &'static str {
        match self {
            ToolId::Codex => "codex",
            ToolId::Grok => "grok",
            ToolId::ClaudeCode => "claudeCode",
            ToolId::Cursor => "cursor",
            ToolId::Agy => "agy",
        }
    }
}

/// One agent session. Never carries prompt or reply text.
#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub tool: ToolId,
    pub id: String,
    pub account_id: String,
    pub model: Option<String>,
    pub project: Option<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub input_tokens: Option<i64>,
    pub cached_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub is_subagent: bool,
    /// Model calls or turns, for tools that record activity but not tokens.
    pub turns: Option<i64>,
}

impl Session {
    pub fn new(tool: ToolId, id: impl Into<String>, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Session {
            tool,
            id: id.into(),
            account_id: "default".into(),
            model: None,
            project: None,
            start,
            end,
            input_tokens: None,
            cached_tokens: None,
            output_tokens: None,
            is_subagent: false,
            turns: None,
        }
    }
}

/// A quota window as the tool last recorded it. The percent is kept as reported.
#[derive(Clone, Debug, PartialEq)]
pub struct Quota {
    pub tool: ToolId,
    pub account_id: String,
    pub label: String,
    pub used_percent: f64,
    pub window_minutes: Option<i64>,
    pub resets_at: Option<DateTime<Utc>>,
    pub observed_at: DateTime<Utc>,
}

impl Quota {
    pub fn new(tool: ToolId, label: impl Into<String>, used_percent: f64, observed_at: DateTime<Utc>) -> Self {
        Quota {
            tool,
            account_id: "default".into(),
            label: label.into(),
            used_percent,
            window_minutes: None,
            resets_at: None,
            observed_at,
        }
    }

    /// A window whose reset time has passed describes the past; it is not shown as the present.
    pub fn is_current(&self, now: DateTime<Utc>) -> bool {
        self.resets_at.map_or(true, |r| r > now)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReaderOutput {
    pub sessions: Vec<Session>,
    pub quotas: Vec<Quota>,
}

impl ReaderOutput {
    /// Stamps every record with the account it was read for.
    pub fn tagged(mut self, account_id: &str) -> Self {
        for s in &mut self.sessions {
            s.account_id = account_id.into();
        }
        for q in &mut self.quotas {
            q.account_id = account_id.into();
        }
        self
    }
}

#[derive(Debug)]
pub struct ReadError(pub String);

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for ReadError {}

impl From<rusqlite::Error> for ReadError {
    fn from(e: rusqlite::Error) -> Self {
        ReadError(e.to_string())
    }
}
impl From<std::io::Error> for ReadError {
    fn from(e: std::io::Error) -> Self {
        ReadError(e.to_string())
    }
}

pub trait UsageReader: Send + Sync {
    fn tool(&self) -> ToolId;
    /// False when the tool's data folder does not exist. Not installed is not zero usage.
    fn is_installed(&self) -> bool;
    fn read(&self, since: DateTime<Utc>) -> Result<ReaderOutput, ReadError>;
}
