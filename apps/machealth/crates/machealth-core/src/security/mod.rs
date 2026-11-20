use crate::telemetry::health_inputs::{
    CheckState, StartupItemsTelemetry, collect_security_summary, collect_startup_items,
};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuditStatus {
    Pass,
    Unknown,
    Warn,
    Critical,
}

impl AuditStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pass => "Good",
            Self::Unknown => "Unknown",
            Self::Warn => "Attention",
            Self::Critical => "Action Needed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityCheckItem {
    pub id: String,
    pub title: String,
    pub status: AuditStatus,
    pub summary: String,
    pub detail: String,
    pub settings_label: String,
    pub settings_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityAuditReport {
    pub checks: Vec<SecurityCheckItem>,
    pub startup_items: StartupItemsTelemetry,
    pub latest_backup_path: Option<String>,
    pub pass_count: usize,
    pub attention_count: usize,
    pub overall_summary: String,
}

/// Runs all 6 non-privileged macOS Security & Maintenance audits with System Settings deep links.
pub fn run_security_audit() -> SecurityAuditReport {
    let summary = collect_security_summary();
    let startup_items = collect_startup_items();
    let latest_backup_path = if summary.time_machine_configured == Some(true) {
        read_latest_time_machine_backup()
    } else {
        None
    };

    build_security_audit_report(&summary, startup_items, latest_backup_path)
}

pub fn build_security_audit_report(
    summary: &crate::telemetry::health_inputs::SecuritySummaryTelemetry,
    startup_items: StartupItemsTelemetry,
    latest_backup_path: Option<String>,
) -> SecurityAuditReport {
    let mut checks = Vec::with_capacity(6);

    // 1. FileVault
    checks.push(match summary.filevault {
        CheckState::Enabled => SecurityCheckItem {
            id: "filevault".into(),
            title: "FileVault Disk Encryption".into(),
            status: AuditStatus::Pass,
            summary: "FileVault is on".into(),
            detail: "Your startup disk is encrypted to protect your files if your Mac is lost or stolen.".into(),
            settings_label: "Open Privacy & Security".into(),
            settings_url: "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_FileVault".into(),
        },
        CheckState::Disabled => SecurityCheckItem {
            id: "filevault".into(),
            title: "FileVault Disk Encryption".into(),
            status: AuditStatus::Critical,
            summary: "FileVault is turned off".into(),
            detail: "Turning on FileVault encrypts your disk automatically with minimal performance impact.".into(),
            settings_label: "Turn On FileVault".into(),
            settings_url: "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_FileVault".into(),
        },
        CheckState::Unknown => SecurityCheckItem {
            id: "filevault".into(),
            title: "FileVault Disk Encryption".into(),
            status: AuditStatus::Unknown,
            summary: "Could not read FileVault status".into(),
            detail: "Check Privacy & Security in System Settings to verify disk encryption.".into(),
            settings_label: "Open Privacy & Security".into(),
            settings_url: "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_FileVault".into(),
        },
    });

    // 2. Firewall
    checks.push(match summary.firewall {
        CheckState::Enabled => SecurityCheckItem {
            id: "firewall".into(),
            title: "macOS Firewall".into(),
            status: AuditStatus::Pass,
            summary: "Firewall is enabled".into(),
            detail: "Built-in application firewall is blocking unwanted incoming network connections.".into(),
            settings_label: "Open Network Settings".into(),
            settings_url: "x-apple.systempreferences:com.apple.Network-Settings.extension".into(),
        },
        CheckState::Disabled => SecurityCheckItem {
            id: "firewall".into(),
            title: "macOS Firewall".into(),
            status: AuditStatus::Warn,
            summary: "Firewall is turned off".into(),
            detail: "Enabling the macOS Firewall helps protect your Mac on shared or public Wi-Fi networks.".into(),
            settings_label: "Open Firewall Settings".into(),
            settings_url: "x-apple.systempreferences:com.apple.Network-Settings.extension".into(),
        },
        CheckState::Unknown => SecurityCheckItem {
            id: "firewall".into(),
            title: "macOS Firewall".into(),
            status: AuditStatus::Unknown,
            summary: "Could not read Firewall status".into(),
            detail: "Inspect Firewall in System Settings -> Network.".into(),
            settings_label: "Open Network Settings".into(),
            settings_url: "x-apple.systempreferences:com.apple.Network-Settings.extension".into(),
        },
    });

    // 3. Gatekeeper
    checks.push(match summary.gatekeeper {
        CheckState::Enabled => SecurityCheckItem {
            id: "gatekeeper".into(),
            title: "Gatekeeper App Verification".into(),
            status: AuditStatus::Pass,
            summary: "Gatekeeper is active".into(),
            detail: "macOS verifies apps before opening them to guard against untrusted software.".into(),
            settings_label: "Open Privacy & Security".into(),
            settings_url: "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension".into(),
        },
        CheckState::Disabled => SecurityCheckItem {
            id: "gatekeeper".into(),
            title: "Gatekeeper App Verification".into(),
            status: AuditStatus::Critical,
            summary: "Gatekeeper is disabled".into(),
            detail: "Apps are not being checked before launch. Re-enable Gatekeeper in Privacy & Security.".into(),
            settings_label: "Open Privacy & Security".into(),
            settings_url: "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension".into(),
        },
        CheckState::Unknown => SecurityCheckItem {
            id: "gatekeeper".into(),
            title: "Gatekeeper App Verification".into(),
            status: AuditStatus::Unknown,
            summary: "Could not read Gatekeeper status".into(),
            detail: "Verify app security settings in Privacy & Security.".into(),
            settings_label: "Open Privacy & Security".into(),
            settings_url: "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension".into(),
        },
    });

    // 4. Time Machine
    checks.push(match summary.time_machine_configured {
        Some(true) => {
            let detail = match &latest_backup_path {
                Some(path) => format!("Time Machine is configured. Latest backup: {path}"),
                None => "Time Machine destination is configured. Connect your backup drive regularly to keep snapshots current.".into(),
            };
            SecurityCheckItem {
                id: "time-machine".into(),
                title: "Time Machine Backups".into(),
                status: AuditStatus::Pass,
                summary: "Time Machine is set up".into(),
                detail,
                settings_label: "Open Time Machine".into(),
                settings_url: "x-apple.systempreferences:com.apple.Time-Machine-Settings.extension".into(),
            }
        }
        Some(false) => SecurityCheckItem {
            id: "time-machine".into(),
            title: "Time Machine Backups".into(),
            status: AuditStatus::Warn,
            summary: "No Time Machine backup configured".into(),
            detail: "Set up an external drive or network share so you can recover from drive failure or accidental deletion.".into(),
            settings_label: "Set Up Time Machine".into(),
            settings_url: "x-apple.systempreferences:com.apple.Time-Machine-Settings.extension".into(),
        },
        None => SecurityCheckItem {
            id: "time-machine".into(),
            title: "Time Machine Backups".into(),
            status: AuditStatus::Unknown,
            summary: "Backup status unavailable".into(),
            detail: "Open Time Machine in System Settings to review your backup configuration.".into(),
            settings_label: "Open Time Machine".into(),
            settings_url: "x-apple.systempreferences:com.apple.Time-Machine-Settings.extension".into(),
        },
    });

    // 5. macOS Software Updates
    checks.push(match summary.pending_macos_updates {
        Some(0) => SecurityCheckItem {
            id: "software-updates".into(),
            title: "macOS Software Updates".into(),
            status: AuditStatus::Pass,
            summary: "Up to date".into(),
            detail: "No pending recommended macOS updates cached locally.".into(),
            settings_label: "Check Software Update".into(),
            settings_url: "x-apple.systempreferences:com.apple.Software-Update-Settings.extension".into(),
        },
        Some(n) => SecurityCheckItem {
            id: "software-updates".into(),
            title: "macOS Software Updates".into(),
            status: AuditStatus::Warn,
            summary: format!("{n} update{} available", if n == 1 { "" } else { "s" }),
            detail: "Installing macOS updates ensures you have the latest security and stability fixes.".into(),
            settings_label: "Open Software Update".into(),
            settings_url: "x-apple.systempreferences:com.apple.Software-Update-Settings.extension".into(),
        },
        None => SecurityCheckItem {
            id: "software-updates".into(),
            title: "macOS Software Updates".into(),
            status: AuditStatus::Unknown,
            summary: "Update status not cached".into(),
            detail: "Open Software Update in System Settings to check for new macOS releases.".into(),
            settings_label: "Open Software Update".into(),
            settings_url: "x-apple.systempreferences:com.apple.Software-Update-Settings.extension".into(),
        },
    });

    // 6. Login / Startup Items
    let startup_status = if startup_items.total_count > 12 {
        AuditStatus::Warn
    } else {
        AuditStatus::Pass
    };
    checks.push(SecurityCheckItem {
        id: "startup-items".into(),
        title: "Login & Startup Items".into(),
        status: startup_status,
        summary: startup_items.summary.clone(),
        detail: if startup_items.total_count == 0 {
            "No background user LaunchAgents found in ~/Library/LaunchAgents.".into()
        } else {
            format!(
                "Found {} user LaunchAgent{}. Disable unneeded items in Login Items to keep startup snappy.",
                startup_items.total_count,
                if startup_items.total_count == 1 { "" } else { "s" }
            )
        },
        settings_label: "Open Login Items".into(),
        settings_url: "x-apple.systempreferences:com.apple.LoginItems-Settings.extension".into(),
    });

    let pass_count = checks.iter().filter(|c| c.status == AuditStatus::Pass).count();
    let attention_count = checks
        .iter()
        .filter(|c| matches!(c.status, AuditStatus::Warn | AuditStatus::Critical))
        .count();

    let overall_summary = if attention_count == 0 {
        format!("All {pass_count} security & maintenance checks look good")
    } else {
        format!(
            "{pass_count} passed · {attention_count} {} attention",
            if attention_count == 1 { "needs" } else { "need" }
        )
    };

    SecurityAuditReport {
        checks,
        startup_items,
        latest_backup_path,
        pass_count,
        attention_count,
        overall_summary,
    }
}

fn read_latest_time_machine_backup() -> Option<String> {
    let output = Command::new("/usr/bin/tmutil")
        .arg("latestbackup")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!s.is_empty()).then_some(s)
}

/// Opens a macOS System Settings pane via its `x-apple.systempreferences:...` URL.
pub fn open_system_settings(url: &str) -> Result<(), String> {
    if !url.starts_with("x-apple.systempreferences:") {
        return Err("Refused non-System-Settings URL".to_string());
    }
    let status = Command::new("/usr/bin/open")
        .arg(url)
        .status()
        .map_err(|e| format!("failed to open System Settings: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("open exited with status {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::health_inputs::SecuritySummaryTelemetry;

    #[test]
    fn builds_complete_six_item_security_audit() {
        let sec = SecuritySummaryTelemetry {
            filevault: CheckState::Enabled,
            firewall: CheckState::Disabled,
            gatekeeper: CheckState::Enabled,
            time_machine_configured: Some(false),
            pending_macos_updates: Some(1),
        };
        let startup = StartupItemsTelemetry {
            user_launch_agents: vec!["com.example.a".into(), "com.example.b".into()],
            total_count: 2,
            summary: "2 background startup items".into(),
        };

        let report = build_security_audit_report(&sec, startup, None);
        assert_eq!(report.checks.len(), 6);
        assert_eq!(report.pass_count, 3);
        assert_eq!(report.attention_count, 3);
        assert!(report.checks.iter().all(|c| c.settings_url.starts_with("x-apple.systempreferences:")));
    }

    #[test]
    fn refuses_non_system_settings_urls() {
        assert!(open_system_settings("https://example.com").is_err());
    }

    #[test]
    fn runs_live_macos_security_audit() {
        let report = run_security_audit();
        assert_eq!(report.checks.len(), 6);
        assert!(!report.overall_summary.is_empty());
    }
}
