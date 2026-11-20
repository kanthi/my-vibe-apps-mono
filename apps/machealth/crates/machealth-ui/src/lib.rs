pub mod components;
pub mod format;
pub mod state;
pub mod theme;
pub mod views;

pub use state::{
    AppState, FastRefreshData, NavPage, PendingModal, RuntimeConfig, SlowRefreshData,
    collect_fast_refresh, collect_slow_refresh,
};
pub use theme::Theme;
pub use views::{MenuBarPopoverView, RootView};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{
        format_bytes_si, format_duration_short, format_memory_gib, format_rate, shorten_home_path,
    };
    use std::path::Path;

    #[test]
    fn format_helpers_produce_tabular_strings() {
        assert_eq!(format_bytes_si(500), "500 B");
        assert_eq!(format_bytes_si(1_500_000_000), "1.5 GB");
        assert_eq!(format_memory_gib(2 * 1_073_741_824), "2.00 GB");
        assert_eq!(format_rate(2_500_000.0), "2.5 MB/s");
        assert_eq!(format_duration_short(3725), "1h 2m");
        let shortened = shorten_home_path(
            Path::new("/Users/someone/Library/Caches/com.very.long.bundle.identifier/sub/file.bin"),
            32,
        );
        assert!(shortened.chars().count() <= 32);
    }

    #[test]
    fn ephemeral_app_state_refreshes_without_touching_host_disk() {
        let mut state = AppState::new(RuntimeConfig::ephemeral());
        assert!(state.config.data_dir.is_none());
        state.refresh_blocking_all().expect("ephemeral refresh");
        assert!(state.snapshot.is_some());
        assert!(state.report.is_some());
        assert!(state.processes.is_some());
        assert!(state.agents.is_some());
        assert!(state.security.is_some());
        assert!(state.cleanup.is_some());
        assert_eq!(state.in_memory_history.len(), 1);
    }
}
