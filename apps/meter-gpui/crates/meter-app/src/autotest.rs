//! Hooks for automated runs in a test VM, switched on only by environment variables. A normal
//! launch sets none of them and none of this runs.
//!   METER_START_PAGE=overview|usage|accounts|settings   open on that page
//!   METER_AUTOTEST_OPEN_MENU_MS=<n>                      open the menu bar menu after n ms
//!   METER_AUTOTEST_CLOSE_WINDOW_MS=<n>                   close the main window after n ms
//!   METER_AUTOTEST_LOG=<path>                            write a JSON summary after each refresh

use crate::state::{AppState, Page};
use std::time::Duration;

pub fn env_ms(name: &str) -> Option<Duration> {
    std::env::var(name).ok()?.parse::<u64>().ok().map(Duration::from_millis)
}

pub fn start_page() -> Option<Page> {
    match std::env::var("METER_START_PAGE").ok()?.to_lowercase().as_str() {
        "overview" => Some(Page::Overview),
        "usage" => Some(Page::Usage),
        "accounts" => Some(Page::Accounts),
        "settings" => Some(Page::Settings),
        _ => None,
    }
}

/// What a script can assert on without reading pixels.
pub fn write_summary(state: &AppState) {
    let Some(path) = std::env::var_os("METER_AUTOTEST_LOG") else { return };
    let d = &state.dashboard;
    let summary = serde_json::json!({
        "menuLines": d.menu.lines,
        "menuWarning": d.menu.warning,
        "tightestPercent": d.menu.tightest_percent,
        "notes": d.notes.iter().map(|n| n.text.clone()).collect::<Vec<_>>(),
        "accounts": d.accounts.iter().map(|a| serde_json::json!({
            "tool": a.account.tool.display_name(),
            "installed": a.installed,
            "hidden": a.account.is_hidden,
            "quotas": a.quotas.iter().map(|q| format!("{} {:.0}%", q.label, q.used_percent)).collect::<Vec<_>>(),
            "tokensToday": a.tokens_today,
            "turnsToday": a.turns_today,
        })).collect::<Vec<_>>(),
    });
    let _ = std::fs::write(path, serde_json::to_vec_pretty(&summary).unwrap_or_default());
}
