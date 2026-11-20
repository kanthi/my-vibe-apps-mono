use crate::telemetry::SystemSnapshot;
use crate::telemetry::health_inputs::CheckState;
use crate::telemetry::memory::MemoryPressure;
use crate::telemetry::storage::SmartStatus;
use crate::telemetry::thermal::ThermalLevel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AreaKind {
    ProcessorAndHeat,
    MemoryAndSwap,
    StorageAndDriveWear,
    Battery,
    SleepAndWake,
    Stability,
    StartupItems,
    SecurityUpdatesBackups,
}

impl AreaKind {
    pub fn title(&self) -> &'static str {
        match self {
            Self::ProcessorAndHeat => "Processor & Heat",
            Self::MemoryAndSwap => "Memory & Swap",
            Self::StorageAndDriveWear => "Storage & Drive Wear",
            Self::Battery => "Battery",
            Self::SleepAndWake => "Sleep & Wake",
            Self::Stability => "Stability",
            Self::StartupItems => "Startup Items",
            Self::SecurityUpdatesBackups => "Security, Updates & Backups",
        }
    }

    /// Weights sum to exactly 100.
    pub fn weight(&self) -> u8 {
        match self {
            Self::ProcessorAndHeat => 15,
            Self::MemoryAndSwap => 15,
            Self::StorageAndDriveWear => 15,
            Self::Battery => 10,
            Self::SleepAndWake => 10,
            Self::Stability => 10,
            Self::StartupItems => 10,
            Self::SecurityUpdatesBackups => 15,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Good,
    Attention,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthGrade {
    Great,
    Good,
    Fair,
    NeedsAttention,
}

impl HealthGrade {
    pub fn from_score(score: u8) -> Self {
        match score {
            85..=100 => Self::Great,
            70..=84 => Self::Good,
            50..=69 => Self::Fair,
            _ => Self::NeedsAttention,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Great => "Great",
            Self::Good => "Good",
            Self::Fair => "Fair",
            Self::NeedsAttention => "Needs Attention",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingAction {
    BalanceLoad,
    OpenActivityView,
    OpenCleanUpView,
    OpenSecurityView,
    OpenSystemSettings { label: String, url: String },
    InspectSleepAssertions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthFinding {
    pub id: String,
    pub area: AreaKind,
    pub severity: Severity,
    pub title: String,
    pub detail: String,
    pub action: FindingAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AreaScore {
    pub area: AreaKind,
    pub title: String,
    pub score: u8,
    pub weight: u8,
    pub status: Severity,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthReport {
    pub overall_score: u8,
    pub grade: HealthGrade,
    pub verdict: String,
    pub areas: Vec<AreaScore>,
    pub findings: Vec<HealthFinding>,
}

pub fn evaluate_health(snapshot: &SystemSnapshot) -> HealthReport {
    let mut findings = Vec::new();

    let areas = vec![
        score_processor_and_heat(snapshot, &mut findings),
        score_memory_and_swap(snapshot, &mut findings),
        score_storage(snapshot, &mut findings),
        score_battery(snapshot, &mut findings),
        score_sleep(snapshot, &mut findings),
        score_stability(snapshot, &mut findings),
        score_startup_items(snapshot, &mut findings),
        score_security_updates_backups(snapshot, &mut findings),
    ];

    // Sort findings: Critical first, then Attention.
    findings.sort_by(|a, b| b.severity.cmp(&a.severity).then_with(|| a.id.cmp(&b.id)));

    let weighted_sum: u32 = areas
        .iter()
        .map(|a| (a.score as u32) * (a.weight as u32))
        .sum();
    let total_weight: u32 = areas.iter().map(|a| a.weight as u32).sum();
    let overall_score = if total_weight > 0 {
        ((weighted_sum + total_weight / 2) / total_weight).min(100) as u8
    } else {
        100
    };

    let critical_count = findings
        .iter()
        .filter(|f| f.severity == Severity::Critical)
        .count();
    let raw_grade = HealthGrade::from_score(overall_score);
    let grade = match (raw_grade, critical_count) {
        (HealthGrade::Great, 1) => HealthGrade::Good,
        (HealthGrade::Great | HealthGrade::Good, 2..) => HealthGrade::Fair,
        (g, _) => g,
    };
    let attention_count = findings.len();
    let verdict = match attention_count {
        0 => format!("{} — everything looks healthy", grade.label()),
        1 => format!("{} — 1 thing needs your attention", grade.label()),
        n => format!("{} — {n} things need your attention", grade.label()),
    };

    HealthReport {
        overall_score,
        grade,
        verdict,
        areas,
        findings,
    }
}

fn score_processor_and_heat(
    snapshot: &SystemSnapshot,
    findings: &mut Vec<HealthFinding>,
) -> AreaScore {
    let mut score: i32 = 100;
    let cpu = snapshot.cpu.total_percent;

    if cpu >= 85.0 {
        score -= 35;
        findings.push(HealthFinding {
            id: "cpu-heavy-load".to_string(),
            area: AreaKind::ProcessorAndHeat,
            severity: Severity::Critical,
            title: format!("Processor is working very hard ({cpu:.0}%)"),
            detail: "Heavy background tasks can make apps feel sluggish. Balance the Load can shift background hogs to Efficiency cores.".to_string(),
            action: FindingAction::BalanceLoad,
        });
    } else if cpu >= 60.0 {
        score -= 15;
        findings.push(HealthFinding {
            id: "cpu-elevated-load".to_string(),
            area: AreaKind::ProcessorAndHeat,
            severity: Severity::Attention,
            title: format!("Processor usage is elevated ({cpu:.0}%)"),
            detail: "One or more apps are using substantial CPU right now.".to_string(),
            action: FindingAction::BalanceLoad,
        });
    }

    match snapshot.thermal.level {
        ThermalLevel::Normal => {}
        ThermalLevel::Warm => score -= 10,
        ThermalLevel::Hot => {
            score -= 30;
            findings.push(HealthFinding {
                id: "thermal-hot".to_string(),
                area: AreaKind::ProcessorAndHeat,
                severity: Severity::Attention,
                title: "Your Mac is running hot".to_string(),
                detail: snapshot.thermal.summary.clone(),
                action: FindingAction::BalanceLoad,
            });
        }
        ThermalLevel::VeryHot => {
            score -= 50;
            findings.push(HealthFinding {
                id: "thermal-very-hot".to_string(),
                area: AreaKind::ProcessorAndHeat,
                severity: Severity::Critical,
                title: "Your Mac is very hot and throttling".to_string(),
                detail: snapshot.thermal.summary.clone(),
                action: FindingAction::BalanceLoad,
            });
        }
    }

    let score = score.clamp(0, 100) as u8;
    let summary = format!("{} · {}", snapshot.cpu.summary, snapshot.thermal.summary);
    make_area(AreaKind::ProcessorAndHeat, score, summary)
}

fn score_memory_and_swap(
    snapshot: &SystemSnapshot,
    findings: &mut Vec<HealthFinding>,
) -> AreaScore {
    let mut score: i32 = 100;
    let mem = &snapshot.memory;

    match mem.pressure {
        MemoryPressure::Normal => {}
        MemoryPressure::Warning => {
            score -= 25;
            findings.push(HealthFinding {
                id: "memory-pressure-warning".to_string(),
                area: AreaKind::MemoryAndSwap,
                severity: Severity::Attention,
                title: "Memory is getting tight".to_string(),
                detail: mem.summary.clone(),
                action: FindingAction::OpenActivityView,
            });
        }
        MemoryPressure::Critical => {
            score -= 55;
            findings.push(HealthFinding {
                id: "memory-pressure-critical".to_string(),
                area: AreaKind::MemoryAndSwap,
                severity: Severity::Critical,
                title: "Memory pressure is critical".to_string(),
                detail: mem.summary.clone(),
                action: FindingAction::OpenActivityView,
            });
        }
    }

    let swap_gb = mem.swap_used_bytes as f64 / 1_073_741_824.0;
    if swap_gb >= 6.0 {
        score -= 25;
        findings.push(HealthFinding {
            id: "memory-heavy-swap".to_string(),
            area: AreaKind::MemoryAndSwap,
            severity: Severity::Attention,
            title: format!("Heavy swap usage ({swap_gb:.1} GB)"),
            detail: "macOS is spilling a lot of memory onto the SSD. Closing unused memory-heavy apps can speed things up.".to_string(),
            action: FindingAction::OpenActivityView,
        });
    } else if swap_gb >= 2.0 {
        score -= 10;
    }

    if mem.used_percent >= 92.0 {
        score -= 15;
    }

    let score = score.clamp(0, 100) as u8;
    make_area(AreaKind::MemoryAndSwap, score, mem.summary.clone())
}

fn score_storage(snapshot: &SystemSnapshot, findings: &mut Vec<HealthFinding>) -> AreaScore {
    let mut score: i32 = 100;
    let storage = &snapshot.storage;
    let free_gb = storage.available_bytes as f64 / 1_000_000_000.0;

    if storage.smart_status == SmartStatus::Failing {
        score -= 70;
        findings.push(HealthFinding {
            id: "storage-smart-failing".to_string(),
            area: AreaKind::StorageAndDriveWear,
            severity: Severity::Critical,
            title: "Drive SMART status is failing".to_string(),
            detail: "macOS reports hardware drive errors. Back up your Mac immediately.".to_string(),
            action: FindingAction::OpenSystemSettings {
                label: "Open Time Machine".to_string(),
                url: "x-apple.systempreferences:com.apple.Time-Machine-Settings.extension".to_string(),
            },
        });
    }

    if free_gb < 15.0 || storage.free_percent < 8.0 {
        score -= 45;
        findings.push(HealthFinding {
            id: "storage-critically-low".to_string(),
            area: AreaKind::StorageAndDriveWear,
            severity: Severity::Critical,
            title: format!("Very low storage ({free_gb:.1} GB available)"),
            detail: "macOS needs free space for updates, swap, and smooth operation. Review Safe Clean Up items.".to_string(),
            action: FindingAction::OpenCleanUpView,
        });
    } else if free_gb < 40.0 || storage.free_percent < 15.0 {
        score -= 20;
        findings.push(HealthFinding {
            id: "storage-getting-low".to_string(),
            area: AreaKind::StorageAndDriveWear,
            severity: Severity::Attention,
            title: format!("Storage is getting low ({free_gb:.1} GB available)"),
            detail: "Cleaning up old installers, downloads, and safe caches can free space.".to_string(),
            action: FindingAction::OpenCleanUpView,
        });
    }

    if let Some(wear) = storage.ssd_wear_percent {
        if wear >= 90 {
            score -= 30;
        } else if wear >= 75 {
            score -= 10;
        }
    }

    let score = score.clamp(0, 100) as u8;
    make_area(AreaKind::StorageAndDriveWear, score, storage.summary.clone())
}

fn score_battery(snapshot: &SystemSnapshot, findings: &mut Vec<HealthFinding>) -> AreaScore {
    let batt = &snapshot.battery;
    if !batt.has_battery {
        return make_area(AreaKind::Battery, 100, batt.summary.clone());
    }

    let mut score: i32 = 100;
    if let Some(health) = batt.health_percent {
        if health < 75 {
            score -= 45;
            findings.push(HealthFinding {
                id: "battery-service-recommended".to_string(),
                area: AreaKind::Battery,
                severity: Severity::Critical,
                title: format!("Battery capacity is low ({health}%)"),
                detail: "Your battery holds significantly less charge than when new. Battery service is recommended.".to_string(),
                action: FindingAction::OpenSystemSettings {
                    label: "Open Battery Settings".to_string(),
                    url: "x-apple.systempreferences:com.apple.Battery-Settings.extension".to_string(),
                },
            });
        } else if health < 80 {
            score -= 25;
            findings.push(HealthFinding {
                id: "battery-below-80".to_string(),
                area: AreaKind::Battery,
                severity: Severity::Attention,
                title: format!("Battery health is below 80% ({health}%)"),
                detail: "Consider checking Battery Health in System Settings.".to_string(),
                action: FindingAction::OpenSystemSettings {
                    label: "Open Battery Settings".to_string(),
                    url: "x-apple.systempreferences:com.apple.Battery-Settings.extension".to_string(),
                },
            });
        } else if health < 88 {
            score -= 10;
        }
    }

    if let Some(temp) = batt.temperature_celsius {
        if temp >= 42.0 {
            score -= 20;
            findings.push(HealthFinding {
                id: "battery-warm".to_string(),
                area: AreaKind::Battery,
                severity: Severity::Attention,
                title: format!("Battery temperature is elevated ({temp:.1}°C)"),
                detail: "High battery temperatures accelerate long-term battery wear.".to_string(),
                action: FindingAction::BalanceLoad,
            });
        }
    }

    let score = score.clamp(0, 100) as u8;
    make_area(AreaKind::Battery, score, batt.summary.clone())
}

fn score_sleep(snapshot: &SystemSnapshot, findings: &mut Vec<HealthFinding>) -> AreaScore {
    let mut score: i32 = 100;
    let sleep = &snapshot.sleep;

    // Exclude system powerd's "display is on" assertion from user-facing penalties.
    let user_blockers: Vec<_> = sleep
        .active_assertions
        .iter()
        .filter(|a| a.process_name != "powerd")
        .collect();

    if !user_blockers.is_empty() {
        let longest = user_blockers
            .iter()
            .map(|a| a.duration_seconds)
            .max()
            .unwrap_or(0);
        if longest >= 1800 || user_blockers.len() >= 3 {
            score -= 25;
            findings.push(HealthFinding {
                id: "sleep-prevented".to_string(),
                area: AreaKind::SleepAndWake,
                severity: Severity::Attention,
                title: format!("{} is preventing your Mac from sleeping", user_blockers[0].process_name),
                detail: format!(
                    "Reason: {} (active for {} min).",
                    user_blockers[0].reason,
                    longest / 60
                ),
                action: FindingAction::InspectSleepAssertions,
            });
        } else {
            score -= 10;
        }
    }

    if sleep.battery_timers.display_sleep_minutes == Some(0) {
        score -= 15;
        findings.push(HealthFinding {
            id: "sleep-display-never-battery".to_string(),
            area: AreaKind::SleepAndWake,
            severity: Severity::Attention,
            title: "Display never sleeps on battery".to_string(),
            detail: "Setting a display sleep timer helps preserve battery life.".to_string(),
            action: FindingAction::OpenSystemSettings {
                label: "Open Lock Screen Settings".to_string(),
                url: "x-apple.systempreferences:com.apple.Lock-Screen-Settings.extension".to_string(),
            },
        });
    }

    let score = score.clamp(0, 100) as u8;
    make_area(AreaKind::SleepAndWake, score, sleep.summary.clone())
}

fn score_stability(snapshot: &SystemSnapshot, findings: &mut Vec<HealthFinding>) -> AreaScore {
    let crashes = snapshot.stability.recent_crash_count_7d;
    let score: u8 = match crashes {
        0 => 100,
        1..=2 => 90,
        3..=5 => {
            findings.push(HealthFinding {
                id: "stability-moderate-crashes".to_string(),
                area: AreaKind::Stability,
                severity: Severity::Attention,
                title: format!("{crashes} crash reports in the last 7 days"),
                detail: snapshot.stability.summary.clone(),
                action: FindingAction::OpenActivityView,
            });
            75
        }
        _ => {
            findings.push(HealthFinding {
                id: "stability-frequent-crashes".to_string(),
                area: AreaKind::Stability,
                severity: Severity::Attention,
                title: format!("Frequent crashes detected ({crashes} in 7 days)"),
                detail: snapshot.stability.summary.clone(),
                action: FindingAction::OpenActivityView,
            });
            55
        }
    };

    make_area(AreaKind::Stability, score, snapshot.stability.summary.clone())
}

fn score_startup_items(snapshot: &SystemSnapshot, findings: &mut Vec<HealthFinding>) -> AreaScore {
    let count = snapshot.startup_items.total_count;
    let score: u8 = match count {
        0..=5 => 100,
        6..=12 => 85,
        13..=20 => {
            findings.push(HealthFinding {
                id: "startup-items-many".to_string(),
                area: AreaKind::StartupItems,
                severity: Severity::Attention,
                title: format!("{count} background startup items installed"),
                detail: "Trimming unused LaunchAgents can speed up login and reduce background RAM use.".to_string(),
                action: FindingAction::OpenSystemSettings {
                    label: "Open Login Items".to_string(),
                    url: "x-apple.systempreferences:com.apple.LoginItems-Settings.extension".to_string(),
                },
            });
            70
        }
        _ => {
            findings.push(HealthFinding {
                id: "startup-items-bloat".to_string(),
                area: AreaKind::StartupItems,
                severity: Severity::Attention,
                title: format!("Heavy startup item clutter ({count} LaunchAgents)"),
                detail: "Many background agents launch automatically at login.".to_string(),
                action: FindingAction::OpenSystemSettings {
                    label: "Open Login Items".to_string(),
                    url: "x-apple.systempreferences:com.apple.LoginItems-Settings.extension".to_string(),
                },
            });
            55
        }
    };

    make_area(
        AreaKind::StartupItems,
        score,
        snapshot.startup_items.summary.clone(),
    )
}

fn score_security_updates_backups(
    snapshot: &SystemSnapshot,
    findings: &mut Vec<HealthFinding>,
) -> AreaScore {
    let mut score: i32 = 100;
    let sec = &snapshot.security;
    let mut issues = Vec::new();

    if sec.filevault == CheckState::Disabled {
        score -= 30;
        issues.push("FileVault off");
        findings.push(HealthFinding {
            id: "security-filevault-off".to_string(),
            area: AreaKind::SecurityUpdatesBackups,
            severity: Severity::Critical,
            title: "FileVault disk encryption is turned off".to_string(),
            detail: "Turn on FileVault to protect your files if your Mac is lost or stolen.".to_string(),
            action: FindingAction::OpenSystemSettings {
                label: "Open Privacy & Security".to_string(),
                url: "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_FileVault".to_string(),
            },
        });
    }

    if sec.firewall == CheckState::Disabled {
        score -= 20;
        issues.push("Firewall off");
        findings.push(HealthFinding {
            id: "security-firewall-off".to_string(),
            area: AreaKind::SecurityUpdatesBackups,
            severity: Severity::Attention,
            title: "macOS Firewall is turned off".to_string(),
            detail: "Enabling the built-in firewall blocks unwanted incoming network connections.".to_string(),
            action: FindingAction::OpenSystemSettings {
                label: "Open Network Firewall".to_string(),
                url: "x-apple.systempreferences:com.apple.Network-Settings.extension".to_string(),
            },
        });
    }

    if sec.gatekeeper == CheckState::Disabled {
        score -= 25;
        issues.push("Gatekeeper off");
        findings.push(HealthFinding {
            id: "security-gatekeeper-off".to_string(),
            area: AreaKind::SecurityUpdatesBackups,
            severity: Severity::Critical,
            title: "Gatekeeper app protection is disabled".to_string(),
            detail: "Gatekeeper verifies apps before opening them to help block malware.".to_string(),
            action: FindingAction::OpenSecurityView,
        });
    }

    if sec.time_machine_configured == Some(false) {
        score -= 15;
        issues.push("No Time Machine backup");
        findings.push(HealthFinding {
            id: "backup-no-time-machine".to_string(),
            area: AreaKind::SecurityUpdatesBackups,
            severity: Severity::Attention,
            title: "Time Machine backup is not set up".to_string(),
            detail: "Connecting an external drive or NAS for Time Machine protects against drive loss.".to_string(),
            action: FindingAction::OpenSystemSettings {
                label: "Open Time Machine".to_string(),
                url: "x-apple.systempreferences:com.apple.Time-Machine-Settings.extension".to_string(),
            },
        });
    }

    if let Some(updates) = sec.pending_macos_updates
        && updates > 0
    {
        score -= 10;
        issues.push("Updates pending");
        findings.push(HealthFinding {
            id: "updates-pending".to_string(),
            area: AreaKind::SecurityUpdatesBackups,
            severity: Severity::Attention,
            title: format!("{updates} macOS software update(s) available"),
            detail: "Installing macOS updates keeps security patches current.".to_string(),
            action: FindingAction::OpenSystemSettings {
                label: "Open Software Update".to_string(),
                url: "x-apple.systempreferences:com.apple.Software-Update-Settings.extension".to_string(),
            },
        });
    }

    let score = score.clamp(0, 100) as u8;
    let summary = if issues.is_empty() {
        "Encryption, firewall, Gatekeeper, and backups look good".to_string()
    } else {
        issues.join(" · ")
    };

    make_area(AreaKind::SecurityUpdatesBackups, score, summary)
}

fn make_area(area: AreaKind, score: u8, summary: String) -> AreaScore {
    let status = match score {
        80..=100 => Severity::Good,
        55..=79 => Severity::Attention,
        _ => Severity::Critical,
    };
    AreaScore {
        area,
        title: area.title().to_string(),
        score,
        weight: area.weight(),
        status,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::battery::{BatteryTelemetry, PowerSource};
    use crate::telemetry::cpu::{CpuTelemetry, CpuTopology, LoadLevel};
    use crate::telemetry::health_inputs::{
        SecuritySummaryTelemetry, StabilityTelemetry, StartupItemsTelemetry,
    };
    use crate::telemetry::memory::MemoryTelemetry;
    use crate::telemetry::network::NetworkTelemetry;
    use crate::telemetry::sleep::{SleepAssertion, SleepTelemetry, SleepTimers};
    use crate::telemetry::storage::StorageTelemetry;
    use crate::telemetry::thermal::ThermalTelemetry;
    use chrono::Utc;

    fn healthy_snapshot() -> SystemSnapshot {
        SystemSnapshot {
            captured_at: Utc::now(),
            cpu: CpuTelemetry {
                total_percent: 12.0,
                user_percent: 8.0,
                system_percent: 4.0,
                idle_percent: 88.0,
                performance_percent: Some(10.0),
                efficiency_percent: Some(15.0),
                load_level: LoadLevel::Light,
                summary: "Processor is mostly idle".to_string(),
                topology: CpuTopology {
                    logical_cores: 10,
                    physical_cores: 10,
                    performance_cores: 6,
                    efficiency_cores: 4,
                    is_apple_silicon: true,
                },
                cores: vec![],
            },
            thermal: ThermalTelemetry {
                level: ThermalLevel::Normal,
                label: "Normal".to_string(),
                summary: "The chip is running cool".to_string(),
                cpu_speed_limit_percent: None,
                pmset_thermal_level: None,
                battery_temp_celsius: Some(30.0),
            },
            memory: MemoryTelemetry {
                total_bytes: 16 * 1_073_741_824,
                used_bytes: 8 * 1_073_741_824,
                app_bytes: 5 * 1_073_741_824,
                wired_bytes: 2 * 1_073_741_824,
                compressed_bytes: 1_073_741_824,
                cached_bytes: 4 * 1_073_741_824,
                free_bytes: 8 * 1_073_741_824,
                swap_total_bytes: 2 * 1_073_741_824,
                swap_used_bytes: 256 * 1_048_576,
                swap_encrypted: true,
                used_percent: 50.0,
                pressure: MemoryPressure::Normal,
                summary: "Apps have the memory they need".to_string(),
            },
            storage: StorageTelemetry {
                mount_point: "/".to_string(),
                total_bytes: 500_000_000_000,
                available_bytes: 200_000_000_000,
                raw_free_bytes: 180_000_000_000,
                purgeable_bytes: 20_000_000_000,
                used_bytes: 300_000_000_000,
                used_percent: 60.0,
                free_percent: 40.0,
                is_solid_state: Some(true),
                smart_status: SmartStatus::Verified,
                ssd_wear_percent: None,
                summary: "Plenty of space free (200.0 GB available)".to_string(),
            },
            network: NetworkTelemetry {
                rx_bytes_per_sec: 1024.0,
                tx_bytes_per_sec: 512.0,
                total_rx_bytes: 100_000,
                total_tx_bytes: 50_000,
                active_interfaces: vec!["en0".to_string()],
            },
            battery: BatteryTelemetry {
                has_battery: true,
                power_source: PowerSource::AcPower,
                charge_percent: Some(85),
                health_percent: Some(92),
                cycle_count: Some(120),
                design_cycle_count: Some(1000),
                design_capacity_mah: Some(6000),
                nominal_max_capacity_mah: Some(5520),
                raw_max_capacity_mah: Some(5480),
                is_charging: false,
                external_connected: true,
                time_remaining_minutes: None,
                temperature_celsius: Some(30.0),
                summary: "Battery health is 92% (120 cycles)".to_string(),
            },
            sleep: SleepTelemetry {
                prevent_system_sleep: false,
                prevent_display_sleep: false,
                active_assertions: vec![],
                battery_timers: SleepTimers {
                    display_sleep_minutes: Some(5),
                    system_sleep_minutes: Some(10),
                },
                ac_timers: SleepTimers {
                    display_sleep_minutes: Some(15),
                    system_sleep_minutes: Some(30),
                },
                active_timers: SleepTimers {
                    display_sleep_minutes: Some(15),
                    system_sleep_minutes: Some(30),
                },
                summary: "Your Mac can sleep normally".to_string(),
            },
            stability: StabilityTelemetry {
                recent_crash_count_7d: 0,
                top_crashing_processes: vec![],
                summary: "No recent crashes in the last 7 days".to_string(),
            },
            startup_items: StartupItemsTelemetry {
                user_launch_agents: vec!["com.example.sync".to_string()],
                total_count: 1,
                summary: "1 background startup item".to_string(),
            },
            security: SecuritySummaryTelemetry {
                filevault: CheckState::Enabled,
                firewall: CheckState::Enabled,
                gatekeeper: CheckState::Enabled,
                time_machine_configured: Some(true),
                pending_macos_updates: Some(0),
            },
        }
    }

    #[test]
    fn area_weights_sum_to_100() {
        let report = evaluate_health(&healthy_snapshot());
        let sum: u32 = report.areas.iter().map(|a| a.weight as u32).sum();
        assert_eq!(report.areas.len(), 8);
        assert_eq!(sum, 100);
        assert_eq!(report.overall_score, 100);
        assert_eq!(report.grade, HealthGrade::Great);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn flags_critical_and_attention_issues_in_priority_order() {
        let mut snap = healthy_snapshot();
        snap.cpu.total_percent = 92.0;
        snap.thermal.level = ThermalLevel::VeryHot;
        snap.memory.pressure = MemoryPressure::Critical;
        snap.storage.available_bytes = 8_000_000_000;
        snap.storage.free_percent = 2.0;
        snap.security.filevault = CheckState::Disabled;
        snap.sleep.active_assertions.push(SleepAssertion {
            pid: 999,
            process_name: "caffeinate".to_string(),
            assertion_type: "PreventUserIdleSystemSleep".to_string(),
            reason: "caffeinate command-line tool".to_string(),
            duration_seconds: 3600,
            created_for_pid: None,
        });

        let report = evaluate_health(&snap);
        assert!(report.overall_score < 70, "score was {}", report.overall_score);
        assert_eq!(report.grade, HealthGrade::Fair);
        assert!(!report.findings.is_empty());
        assert_eq!(report.findings[0].severity, Severity::Critical);
        assert!(report.verdict.contains("need your attention"));
    }

    #[test]
    fn severe_system_wide_issues_yield_needs_attention() {
        let mut snap = healthy_snapshot();
        snap.cpu.total_percent = 95.0;
        snap.thermal.level = ThermalLevel::VeryHot;
        snap.memory.pressure = MemoryPressure::Critical;
        snap.memory.swap_used_bytes = 8 * 1_073_741_824;
        snap.storage.smart_status = SmartStatus::Failing;
        snap.storage.available_bytes = 5_000_000_000;
        snap.storage.free_percent = 1.0;
        snap.battery.health_percent = Some(70);
        snap.stability.recent_crash_count_7d = 12;
        snap.startup_items.total_count = 25;
        snap.security.filevault = CheckState::Disabled;
        snap.security.firewall = CheckState::Disabled;
        snap.security.gatekeeper = CheckState::Disabled;

        let report = evaluate_health(&snap);
        assert!(report.overall_score < 50, "score was {}", report.overall_score);
        assert_eq!(report.grade, HealthGrade::NeedsAttention);
    }

    #[test]
    fn desktop_mac_gets_full_battery_area_score() {
        let mut snap = healthy_snapshot();
        snap.battery.has_battery = false;
        snap.battery.health_percent = None;
        snap.battery.summary = "Desktop Mac running on wall power".to_string();

        let report = evaluate_health(&snap);
        let battery_area = report
            .areas
            .iter()
            .find(|a| a.area == AreaKind::Battery)
            .unwrap();
        assert_eq!(battery_area.score, 100);
    }
}
