mod agy;
mod claude_code;
mod codex;
mod cursor;
mod grok;

pub use agy::AgyReader;
pub use claude_code::ClaudeCodeReader;
pub use codex::CodexReader;
pub use cursor::CursorReader;
pub use grok::GrokReader;

use crate::model::{ToolId, UsageReader};
use crate::util::home_dir;
use std::path::PathBuf;

/// A reader for one account: `root` overrides the tool's default data folder.
pub fn make(tool: ToolId, root: Option<PathBuf>) -> Box<dyn UsageReader> {
    let h = home_dir();
    match tool {
        ToolId::Codex => Box::new(CodexReader::new(root.unwrap_or_else(|| h.join(".codex")))),
        ToolId::Grok => Box::new(GrokReader::new(root.unwrap_or_else(|| h.join(".grok")))),
        ToolId::ClaudeCode => Box::new(ClaudeCodeReader::new(root.unwrap_or_else(|| h.join(".claude")))),
        ToolId::Cursor => Box::new(CursorReader::new(
            root.unwrap_or_else(|| h.join("Library/Application Support/Cursor")),
        )),
        ToolId::Agy => Box::new(AgyReader::new(root.unwrap_or_else(|| h.join(".gemini/antigravity-cli")))),
    }
}

/// Window label from its length: a 5-hour window is a Session, 7 days Weekly, 30 days Monthly.
pub(crate) fn window_label(minutes: Option<i64>) -> String {
    match minutes {
        None => "Limit".into(),
        Some(m) if m <= 600 => "Session".into(),
        Some(m) if (9_000..=11_000).contains(&m) => "Weekly".into(),
        Some(m) if m >= 40_000 => "Monthly".into(),
        Some(m) => format!("{m} min"),
    }
}
