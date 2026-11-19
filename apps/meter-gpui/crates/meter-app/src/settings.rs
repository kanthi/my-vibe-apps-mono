use meter_core::AlertPolicy;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// User settings, kept as a JSON file in Meter's own folder (never in the host's preferences).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub show_percent_in_menu_bar: bool,
    pub refresh_seconds: u64,
    /// Off until the user turns it on.
    pub launch_at_login: bool,
    pub threshold_low: f64,
    pub threshold_high: f64,
    pub alert_policy: AlertPolicy,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            show_percent_in_menu_bar: true,
            refresh_seconds: 60,
            launch_at_login: false,
            threshold_low: 80.0,
            threshold_high: 95.0,
            alert_policy: AlertPolicy::default(),
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Settings {
        std::fs::read(path).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let tmp = path.with_extension("json.tmp");
        if let Ok(bytes) = serde_json::to_vec_pretty(self) {
            if std::fs::write(&tmp, bytes).is_ok() {
                let _ = std::fs::rename(tmp, path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_conservative() {
        let s = Settings::default();
        assert!(s.alert_policy.enabled_kinds.is_empty(), "all alerts start off");
        assert!(!s.launch_at_login);
        assert_eq!((s.refresh_seconds, s.threshold_low, s.threshold_high), (60, 80.0, 95.0));
    }

    #[test]
    fn round_trips_and_tolerates_missing_fields_and_garbage() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("sub/settings.json");
        let mut s = Settings::default();
        s.show_percent_in_menu_bar = false;
        s.alert_policy.enabled_kinds.insert("pace".into());
        s.save(&p);
        assert_eq!(Settings::load(&p), s);
        std::fs::write(&p, r#"{"refreshSeconds":300}"#).unwrap();
        let partial = Settings::load(&p);
        assert_eq!((partial.refresh_seconds, partial.show_percent_in_menu_bar), (300, true));
        std::fs::write(&p, "not json").unwrap();
        assert_eq!(Settings::load(&p), Settings::default());
    }
}
