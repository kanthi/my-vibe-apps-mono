use machealth_ui::RuntimeConfig;
use std::path::PathBuf;

/// Runtime flags controlling host isolation so dev/testing never registers or persists
/// anything onto the host macOS environment.
#[derive(Debug, Clone)]
pub struct LaunchOptions {
    pub smoke_test: bool,
    pub ephemeral: bool,
    pub enable_menubar: bool,
    pub enable_hotkey: bool,
    pub open_popover_on_start: bool,
    pub data_dir_override: Option<PathBuf>,
}

impl LaunchOptions {
    pub fn from_env_and_args() -> Self {
        let args: Vec<String> = std::env::args().skip(1).collect();
        Self::parse(&args, std::env::var_os("MACHEALTH_EPHEMERAL").is_some(), std::env::var_os("MACHEALTH_DATA_DIR").map(PathBuf::from))
    }

    pub fn parse(
        args: &[String],
        env_ephemeral: bool,
        env_data_dir: Option<PathBuf>,
    ) -> Self {
        let smoke_test = args.iter().any(|a| a == "--smoke-test");
        let ephemeral = smoke_test || env_ephemeral || args.iter().any(|a| a == "--ephemeral");
        let enable_menubar = !smoke_test && !args.iter().any(|a| a == "--no-menubar");
        let enable_hotkey = !smoke_test && !args.iter().any(|a| a == "--no-hotkey");
        let open_popover_on_start = args.iter().any(|a| a == "--open-popover");

        Self {
            smoke_test,
            ephemeral,
            enable_menubar,
            enable_hotkey,
            open_popover_on_start,
            data_dir_override: env_data_dir,
        }
    }

    pub fn to_runtime_config(&self) -> RuntimeConfig {
        if self.ephemeral {
            return RuntimeConfig {
                data_dir: None,
                allow_external_actions: !self.smoke_test,
            };
        }
        if let Some(ref custom) = self.data_dir_override {
            return RuntimeConfig {
                data_dir: Some(custom.clone()),
                allow_external_actions: true,
            };
        }
        RuntimeConfig::default_persistent()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_disables_all_host_registration_and_writes() {
        let opts = LaunchOptions::parse(&["--smoke-test".into()], false, None);
        assert!(opts.smoke_test);
        assert!(opts.ephemeral);
        assert!(!opts.enable_menubar);
        assert!(!opts.enable_hotkey);

        let cfg = opts.to_runtime_config();
        assert!(cfg.data_dir.is_none());
        assert!(!cfg.allow_external_actions);
    }

    #[test]
    fn custom_data_dir_redirects_storage_cleanly() {
        let dir = PathBuf::from("/tmp/machealth-sandbox");
        let opts = LaunchOptions::parse(&[], false, Some(dir.clone()));
        let cfg = opts.to_runtime_config();
        assert_eq!(cfg.data_dir, Some(dir));
    }
}
