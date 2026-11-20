use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrashProcessCount {
    pub process_name: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StabilityTelemetry {
    pub recent_crash_count_7d: usize,
    pub top_crashing_processes: Vec<CrashProcessCount>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StartupItemsTelemetry {
    pub user_launch_agents: Vec<String>,
    pub total_count: usize,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CheckState {
    Enabled,
    Disabled,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SecuritySummaryTelemetry {
    pub filevault: CheckState,
    pub firewall: CheckState,
    pub gatekeeper: CheckState,
    pub time_machine_configured: Option<bool>,
    pub pending_macos_updates: Option<usize>,
}

pub fn collect_stability() -> StabilityTelemetry {
    let Some(home) = home_dir() else {
        return StabilityTelemetry {
            summary: "Crash reports folder unavailable".to_string(),
            ..Default::default()
        };
    };
    let dir = home.join("Library/Logs/DiagnosticReports");
    scan_diagnostic_reports(&dir, Duration::from_secs(7 * 24 * 3600))
}

pub fn scan_diagnostic_reports(dir: &Path, window: Duration) -> StabilityTelemetry {
    let cutoff = SystemTime::now().checked_sub(window).unwrap_or(SystemTime::UNIX_EPOCH);
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut total = 0usize;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "ips" | "crash" | "diag") {
                continue;
            }
            let modified = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            if modified < cutoff {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            let process = extract_process_from_report_name(stem);
            *counts.entry(process).or_insert(0) += 1;
            total += 1;
        }
    }

    let mut top: Vec<CrashProcessCount> = counts
        .into_iter()
        .map(|(process_name, count)| CrashProcessCount { process_name, count })
        .collect();
    top.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.process_name.cmp(&b.process_name)));
    top.truncate(5);

    let summary = if total == 0 {
        "No recent crashes in the last 7 days".to_string()
    } else if total == 1 {
        format!(
            "1 crash report in the last 7 days ({})",
            top.first().map(|c| c.process_name.as_str()).unwrap_or("app")
        )
    } else {
        format!(
            "{total} crash reports in the last 7 days (most: {})",
            top.first().map(|c| c.process_name.as_str()).unwrap_or("app")
        )
    };

    StabilityTelemetry {
        recent_crash_count_7d: total,
        top_crashing_processes: top,
        summary,
    }
}

fn extract_process_from_report_name(stem: &str) -> String {
    // Examples:
    // "ExcUserFault_textunderstandingd-2026-11-19-213909"
    // "Xcode-2026-11-19-102200"
    let without_prefix = stem
        .strip_prefix("ExcUserFault_")
        .or_else(|| stem.strip_prefix("ExcResource_"))
        .unwrap_or(stem);

    if let Some(idx) = without_prefix.find("-20") {
        without_prefix[..idx].to_string()
    } else {
        without_prefix.to_string()
    }
}

pub fn collect_startup_items() -> StartupItemsTelemetry {
    let Some(home) = home_dir() else {
        return StartupItemsTelemetry {
            summary: "Startup items unavailable".to_string(),
            ..Default::default()
        };
    };
    scan_launch_agents(&home.join("Library/LaunchAgents"))
}

pub fn scan_launch_agents(dir: &Path) -> StartupItemsTelemetry {
    let mut items = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("plist") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    items.push(stem.to_string());
                }
            }
        }
    }
    items.sort();
    let total_count = items.len();
    let summary = match total_count {
        0 => "No user background startup items".to_string(),
        1 => "1 background startup item".to_string(),
        n => format!("{n} background startup items"),
    };

    StartupItemsTelemetry {
        user_launch_agents: items,
        total_count,
        summary,
    }
}

pub fn collect_security_summary() -> SecuritySummaryTelemetry {
    let filevault = run_cmd("/usr/bin/fdesetup", &["status"])
        .map(|out| parse_filevault_status(&out))
        .unwrap_or(CheckState::Unknown);

    let firewall = run_cmd(
        "/usr/libexec/ApplicationFirewall/socketfilterfw",
        &["--getglobalstate"],
    )
    .map(|out| parse_firewall_status(&out))
    .unwrap_or(CheckState::Unknown);

    let gatekeeper = run_cmd("/usr/sbin/spctl", &["--status"])
        .map(|out| parse_gatekeeper_status(&out))
        .unwrap_or(CheckState::Unknown);

    let time_machine_configured = run_cmd("/usr/bin/tmutil", &["destinationinfo"])
        .map(|out| parse_time_machine_configured(&out));

    let pending_macos_updates = run_cmd(
        "/usr/bin/defaults",
        &[
            "read",
            "/Library/Preferences/com.apple.SoftwareUpdate",
            "RecommendedUpdates",
        ],
    )
    .map(|out| count_recommended_updates(&out));

    SecuritySummaryTelemetry {
        filevault,
        firewall,
        gatekeeper,
        time_machine_configured,
        pending_macos_updates,
    }
}

fn run_cmd(bin: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(bin).args(args).output().ok()?;
    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    if combined.trim().is_empty() {
        combined = String::from_utf8_lossy(&output.stderr).into_owned();
    }
    Some(combined)
}

pub fn parse_filevault_status(out: &str) -> CheckState {
    if out.contains("FileVault is On") {
        CheckState::Enabled
    } else if out.contains("FileVault is Off") {
        CheckState::Disabled
    } else {
        CheckState::Unknown
    }
}

pub fn parse_firewall_status(out: &str) -> CheckState {
    if out.contains("Firewall is enabled") || out.contains("State = 1") || out.contains("State = 2") {
        CheckState::Enabled
    } else if out.contains("Firewall is disabled") || out.contains("State = 0") {
        CheckState::Disabled
    } else {
        CheckState::Unknown
    }
}

pub fn parse_gatekeeper_status(out: &str) -> CheckState {
    if out.contains("assessments enabled") {
        CheckState::Enabled
    } else if out.contains("assessments disabled") {
        CheckState::Disabled
    } else {
        CheckState::Unknown
    }
}

pub fn parse_time_machine_configured(out: &str) -> bool {
    !out.contains("No destinations configured") && out.contains("Name")
}

pub fn count_recommended_updates(out: &str) -> usize {
    out.matches("Display Name").count()
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_crash_process_name() {
        assert_eq!(
            extract_process_from_report_name("ExcUserFault_textunderstandingd-2026-11-19-213909"),
            "textunderstandingd"
        );
        assert_eq!(
            extract_process_from_report_name("Xcode-2026-11-19-102200"),
            "Xcode"
        );
    }

    #[test]
    fn parses_security_tool_outputs() {
        assert_eq!(parse_filevault_status("FileVault is On."), CheckState::Enabled);
        assert_eq!(parse_filevault_status("FileVault is Off."), CheckState::Disabled);
        assert_eq!(
            parse_firewall_status("Firewall is enabled. (State = 1)"),
            CheckState::Enabled
        );
        assert_eq!(parse_gatekeeper_status("assessments enabled"), CheckState::Enabled);
        assert!(!parse_time_machine_configured(
            "tmutil: No destinations configured."
        ));
        assert!(parse_time_machine_configured(
            "====================================================\nName          : BackupDrive\n"
        ));
    }

    #[test]
    fn reads_live_macos_health_inputs() {
        let stability = collect_stability();
        let startup = collect_startup_items();
        let security = collect_security_summary();
        assert!(!stability.summary.is_empty());
        assert!(!startup.summary.is_empty());
        assert_ne!(security.gatekeeper, CheckState::Unknown);
    }
}
