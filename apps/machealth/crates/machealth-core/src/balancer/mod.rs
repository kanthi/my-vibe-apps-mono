use crate::processes::actions::{GuardrailVerdict, check_process_safety, quit_process};
use crate::processes::{AppGroup, ProcessInfo};
use crate::telemetry::SystemSnapshot;
use crate::telemetry::memory::MemoryPressure;
use crate::telemetry::thermal::ThermalLevel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BalanceConfig {
    /// Minimum CPU % for a non-frontmost background process to be considered a CPU hog.
    pub background_cpu_threshold_percent: f32,
    /// Minimum memory footprint (bytes) for an idle GUI app to be suggested for optional quit.
    pub idle_app_memory_threshold_bytes: u64,
    /// Maximum CPU % for an app to be considered idle.
    pub idle_app_max_cpu_percent: f32,
    /// Minimum uptime (seconds) before an idle app is suggested.
    pub idle_app_min_uptime_seconds: u64,
}

impl Default for BalanceConfig {
    fn default() -> Self {
        Self {
            background_cpu_threshold_percent: 25.0,
            idle_app_memory_threshold_bytes: 512 * 1_048_576, // 512 MiB
            idle_app_max_cpu_percent: 1.5,
            idle_app_min_uptime_seconds: 1800, // 30 minutes
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BalanceActionKind {
    /// Shift process to Apple Silicon Efficiency cores via `taskpolicy -b -p <pid>`.
    MoveToEfficiencyCores,
    /// Optional graceful quit of a long-idle memory-heavy app.
    QuitIdleApp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceTarget {
    pub pid: u32,
    pub display_name: String,
    pub action: BalanceActionKind,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub reason: String,
    pub process: ProcessInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalancePlan {
    pub throttle_targets: Vec<BalanceTarget>,
    pub idle_quit_candidates: Vec<BalanceTarget>,
    pub summary: String,
}

impl BalancePlan {
    pub fn is_empty(&self) -> bool {
        self.throttle_targets.is_empty() && self.idle_quit_candidates.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricsReading {
    pub captured_at: DateTime<Utc>,
    pub cpu_percent: f32,
    pub thermal_level: ThermalLevel,
    pub memory_used_bytes: u64,
    pub memory_pressure: MemoryPressure,
}

impl MetricsReading {
    pub fn from_snapshot(snapshot: &SystemSnapshot) -> Self {
        Self {
            captured_at: snapshot.captured_at,
            cpu_percent: snapshot.cpu.total_percent,
            thermal_level: snapshot.thermal.level,
            memory_used_bytes: snapshot.memory.used_bytes,
            memory_pressure: snapshot.memory.pressure,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThrottledRecord {
    pub pid: u32,
    pub name: String,
    pub applied_at: DateTime<Utc>,
    pub batch_id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndoLedger {
    pub entries: Vec<ThrottledRecord>,
}

impl UndoLedger {
    pub fn default_path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        Some(home.join("Library/Application Support/MacHealth/balance_undo.json"))
    }

    pub fn load_from(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str::<UndoLedger>(&s).ok())
            .unwrap_or_default()
    }

    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create MacHealth support directory: {e}"))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to serialize undo ledger: {e}"))?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, json).map_err(|e| format!("failed to write undo ledger: {e}"))?;
        fs::rename(&tmp, path).map_err(|e| format!("failed to commit undo ledger: {e}"))?;
        Ok(())
    }

    pub fn is_throttled(&self, pid: u32) -> bool {
        self.entries.iter().any(|e| e.pid == pid)
    }

    pub fn has_pending_undo(&self) -> bool {
        !self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceResult {
    pub batch_id: String,
    pub before: MetricsReading,
    pub after: MetricsReading,
    pub cpu_delta_percent: f32,
    pub memory_delta_bytes: i64,
    pub throttled: Vec<ThrottledRecord>,
    pub quit_pids: Vec<u32>,
    pub failures: Vec<(u32, String)>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndoResult {
    pub restored_pids: Vec<u32>,
    pub failures: Vec<(u32, String)>,
    pub summary: String,
}

/// Abstraction over macOS `taskpolicy` so tests can verify the state machine without mutating
/// unrelated host processes, while production uses real `/usr/bin/taskpolicy`.
pub trait TaskPolicyExecutor {
    fn set_background(&self, pid: u32) -> Result<(), String>;
    fn clear_background(&self, pid: u32) -> Result<(), String>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemTaskPolicyExecutor;

impl TaskPolicyExecutor for SystemTaskPolicyExecutor {
    fn set_background(&self, pid: u32) -> Result<(), String> {
        run_taskpolicy("-b", pid)
    }

    fn clear_background(&self, pid: u32) -> Result<(), String> {
        run_taskpolicy("-B", pid)
    }
}

fn run_taskpolicy(flag: &str, pid: u32) -> Result<(), String> {
    let pid_str = pid.to_string();
    let output = Command::new("/usr/sbin/taskpolicy")
        .args([flag, "-p", &pid_str])
        .output()
        .map_err(|e| format!("failed to execute taskpolicy: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        // Treat already-exited PID as benign during restore.
        if flag == "-B" && (stderr.contains("No such process") || stderr.contains("ESRCH")) {
            return Ok(());
        }
        Err(if stderr.is_empty() {
            format!("taskpolicy {flag} -p {pid} exited with {}", output.status)
        } else {
            stderr
        })
    }
}

/// Scans running app groups for heavy background CPU hogs and long-idle memory-heavy apps.
pub fn scan_balance_candidates(
    groups: &[AppGroup],
    frontmost_pid: Option<u32>,
    ledger: &UndoLedger,
    config: &BalanceConfig,
    current_uid: u32,
    current_pid: u32,
) -> BalancePlan {
    let mut throttle_targets = Vec::new();
    let mut idle_quit_candidates = Vec::new();

    for group in groups {
        let is_frontmost = frontmost_pid
            .map(|fp| group.processes.iter().any(|p| p.pid == fp))
            .unwrap_or(false);
        if is_frontmost {
            continue;
        }

        // 1. Heavy background CPU hogs -> move to Efficiency cores via taskpolicy -b.
        for proc_info in &group.processes {
            if proc_info.cpu_percent < config.background_cpu_threshold_percent {
                continue;
            }
            if ledger.is_throttled(proc_info.pid) {
                continue;
            }
            if check_process_safety(proc_info, current_uid, current_pid) != GuardrailVerdict::Allowed
            {
                continue;
            }

            throttle_targets.push(BalanceTarget {
                pid: proc_info.pid,
                display_name: if group.display_name != proc_info.name {
                    format!("{} ({})", group.display_name, proc_info.name)
                } else {
                    proc_info.name.clone()
                },
                action: BalanceActionKind::MoveToEfficiencyCores,
                cpu_percent: proc_info.cpu_percent,
                memory_bytes: proc_info.memory_bytes,
                reason: format!(
                    "Using {:.0}% CPU in the background — can move to Efficiency cores",
                    proc_info.cpu_percent
                ),
                process: proc_info.clone(),
            });
        }

        // 2. Long-idle, memory-heavy GUI apps -> suggest optional graceful quit.
        if group.bundle_path.is_some()
            && group.memory_bytes >= config.idle_app_memory_threshold_bytes
            && group.cpu_percent <= config.idle_app_max_cpu_percent
        {
            if let Some(main_proc) = group.processes.first() {
                if main_proc.uptime_seconds >= config.idle_app_min_uptime_seconds
                    && check_process_safety(main_proc, current_uid, current_pid)
                        == GuardrailVerdict::Allowed
                {
                    let mem_gb = group.memory_bytes as f64 / 1_073_741_824.0;
                    idle_quit_candidates.push(BalanceTarget {
                        pid: main_proc.pid,
                        display_name: group.display_name.clone(),
                        action: BalanceActionKind::QuitIdleApp,
                        cpu_percent: group.cpu_percent,
                        memory_bytes: group.memory_bytes,
                        reason: format!(
                            "Idle in background while holding {mem_gb:.1} GB of memory"
                        ),
                        process: main_proc.clone(),
                    });
                }
            }
        }
    }

    throttle_targets.sort_by(|a, b| {
        b.cpu_percent
            .partial_cmp(&a.cpu_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    idle_quit_candidates.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));

    let summary = match (throttle_targets.len(), idle_quit_candidates.len()) {
        (0, 0) => "Everything looks balanced right now — no heavy background tasks found.".into(),
        (t, 0) => format!(
            "Move {t} heavy background {} to Efficiency cores.",
            if t == 1 { "task" } else { "tasks" }
        ),
        (0, q) => format!(
            "Optionally quit {q} idle {} holding significant memory.",
            if q == 1 { "app" } else { "apps" }
        ),
        (t, q) => format!(
            "Move {t} heavy background {} to Efficiency cores and optionally quit {q} idle {}.",
            if t == 1 { "task" } else { "tasks" },
            if q == 1 { "app" } else { "apps" }
        ),
    };

    BalancePlan {
        throttle_targets,
        idle_quit_candidates,
        summary,
    }
}

/// Applies confirmed targets from a `BalancePlan`, captures before/after metrics, and persists Undo.
#[allow(clippy::too_many_arguments)]
pub fn apply_balance_plan<E: TaskPolicyExecutor>(
    throttle_targets: &[BalanceTarget],
    selected_idle_quits: &[BalanceTarget],
    before: MetricsReading,
    after_reader: impl FnOnce() -> MetricsReading,
    executor: &E,
    ledger: &mut UndoLedger,
    ledger_path: Option<&Path>,
    current_uid: u32,
    current_pid: u32,
) -> BalanceResult {
    let now = Utc::now();
    let batch_id = format!("bal-{}", now.timestamp_millis());
    let mut throttled = Vec::new();
    let mut quit_pids = Vec::new();
    let mut failures = Vec::new();

    for target in throttle_targets {
        match check_process_safety(&target.process, current_uid, current_pid) {
            GuardrailVerdict::Refused { reason } => {
                failures.push((target.pid, reason));
                continue;
            }
            GuardrailVerdict::Allowed => {}
        }

        match executor.set_background(target.pid) {
            Ok(()) => {
                let record = ThrottledRecord {
                    pid: target.pid,
                    name: target.display_name.clone(),
                    applied_at: now,
                    batch_id: batch_id.clone(),
                };
                if !ledger.is_throttled(target.pid) {
                    ledger.entries.push(record.clone());
                }
                throttled.push(record);
            }
            Err(err) => failures.push((target.pid, err)),
        }
    }

    for target in selected_idle_quits {
        match quit_process(&target.process, false, current_uid, current_pid) {
            Ok(()) => quit_pids.push(target.pid),
            Err(err) => failures.push((target.pid, err)),
        }
    }

    if let Some(path) = ledger_path {
        if let Err(err) = ledger.save_to(path) {
            failures.push((0, err));
        }
    }

    let after = after_reader();
    let cpu_delta_percent = after.cpu_percent - before.cpu_percent;
    let memory_delta_bytes = after.memory_used_bytes as i64 - before.memory_used_bytes as i64;

    let summary = format!(
        "Moved {} {} to Efficiency cores, quit {} idle {} (CPU {:+.1}%).",
        throttled.len(),
        if throttled.len() == 1 { "process" } else { "processes" },
        quit_pids.len(),
        if quit_pids.len() == 1 { "app" } else { "apps" },
        cpu_delta_percent
    );

    BalanceResult {
        batch_id,
        before,
        after,
        cpu_delta_percent,
        memory_delta_bytes,
        throttled,
        quit_pids,
        failures,
        summary,
    }
}

/// Restores all throttled processes (or a specific batch) back to normal scheduling via `taskpolicy -B`.
pub fn undo_balance<E: TaskPolicyExecutor>(
    batch_id: Option<&str>,
    executor: &E,
    ledger: &mut UndoLedger,
    ledger_path: Option<&Path>,
) -> UndoResult {
    let mut restored_pids = Vec::new();
    let mut failures = Vec::new();
    let mut remaining = Vec::new();

    for entry in ledger.entries.drain(..) {
        let should_restore = batch_id.map(|id| entry.batch_id == id).unwrap_or(true);
        if !should_restore {
            remaining.push(entry);
            continue;
        }

        match executor.clear_background(entry.pid) {
            Ok(()) => restored_pids.push(entry.pid),
            Err(err) => {
                failures.push((entry.pid, err));
                remaining.push(entry);
            }
        }
    }

    ledger.entries = remaining;
    if let Some(path) = ledger_path {
        if let Err(err) = ledger.save_to(path) {
            failures.push((0, err));
        }
    }

    let summary = if restored_pids.is_empty() && failures.is_empty() {
        "No balanced processes needed restoring.".to_string()
    } else {
        format!(
            "Restored {} {} to normal scheduling.",
            restored_pids.len(),
            if restored_pids.len() == 1 { "process" } else { "processes" }
        )
    };

    UndoResult {
        restored_pids,
        failures,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processes::descriptions::ImpactLabel;
    use std::cell::RefCell;

    #[derive(Default)]
    struct MockPolicy {
        backgrounded: RefCell<Vec<u32>>,
        restored: RefCell<Vec<u32>>,
    }

    impl TaskPolicyExecutor for MockPolicy {
        fn set_background(&self, pid: u32) -> Result<(), String> {
            self.backgrounded.borrow_mut().push(pid);
            Ok(())
        }
        fn clear_background(&self, pid: u32) -> Result<(), String> {
            self.restored.borrow_mut().push(pid);
            Ok(())
        }
    }

    fn mk_group(
        pid: u32,
        name: &str,
        bundle: Option<&str>,
        cpu: f32,
        mem_mib: u64,
        uptime_secs: u64,
    ) -> AppGroup {
        let path = bundle
            .map(|b| format!("{b}/Contents/MacOS/{name}"))
            .unwrap_or_else(|| format!("/usr/local/bin/{name}"));
        let p = ProcessInfo {
            pid,
            ppid: 1,
            uid: 501,
            name: name.to_string(),
            executable_path: Some(path.clone()),
            command_line: path,
            app_bundle_name: bundle.map(|_| name.to_string()),
            app_bundle_path: bundle.map(str::to_string),
            is_helper: false,
            cpu_percent: cpu,
            memory_bytes: mem_mib * 1_048_576,
            energy_impact: cpu,
            disk_read_bytes: 0,
            disk_write_bytes: 0,
            disk_read_bytes_per_sec: 0.0,
            disk_write_bytes_per_sec: 0.0,
            net_rx_bytes: 0,
            net_tx_bytes: 0,
            net_rx_bytes_per_sec: 0.0,
            net_tx_bytes_per_sec: 0.0,
            uptime_seconds: uptime_secs,
            impact_label: ImpactLabel::classify(cpu, mem_mib * 1_048_576, cpu),
            description: String::new(),
        };
        AppGroup {
            id: format!("g:{pid}"),
            display_name: name.to_string(),
            primary_pid: pid,
            bundle_path: bundle.map(str::to_string),
            description: String::new(),
            impact_label: p.impact_label,
            process_count: 1,
            helper_count: 0,
            cpu_percent: cpu,
            memory_bytes: mem_mib * 1_048_576,
            energy_impact: cpu,
            disk_read_bytes: 0,
            disk_write_bytes: 0,
            disk_read_bytes_per_sec: 0.0,
            disk_write_bytes_per_sec: 0.0,
            net_rx_bytes: 0,
            net_tx_bytes: 0,
            net_rx_bytes_per_sec: 0.0,
            net_tx_bytes_per_sec: 0.0,
            processes: vec![p],
        }
    }

    #[test]
    fn scans_applies_persists_and_undoes_balance_plan() {
        let groups = vec![
            // Active frontmost app -> must NEVER be touched!
            mk_group(100, "Xcode", Some("/Applications/Xcode.app"), 85.0, 2048, 3600),
            // Heavy background CPU hog -> should be throttled
            mk_group(200, "ffmpeg", None, 65.0, 200, 120),
            // Long-idle memory-heavy GUI app -> optional quit candidate
            mk_group(300, "Slack", Some("/Applications/Slack.app"), 0.2, 1200, 7200),
            // Protected macOS process -> must NEVER be touched!
            mk_group(400, "WindowServer", None, 40.0, 800, 99999),
        ];

        let mut ledger = UndoLedger::default();
        let config = BalanceConfig::default();
        let plan = scan_balance_candidates(&groups, Some(100), &ledger, &config, 501, 9999);

        assert_eq!(plan.throttle_targets.len(), 1);
        assert_eq!(plan.throttle_targets[0].pid, 200);
        assert_eq!(plan.idle_quit_candidates.len(), 1);
        assert_eq!(plan.idle_quit_candidates[0].pid, 300);

        let tmp_ledger = std::env::temp_dir().join(format!(
            "machealth_balance_undo_test_{}.json",
            std::process::id()
        ));
        let mock = MockPolicy::default();
        let before = MetricsReading {
            captured_at: Utc::now(),
            cpu_percent: 78.0,
            thermal_level: ThermalLevel::Warm,
            memory_used_bytes: 12 * 1_073_741_824,
            memory_pressure: MemoryPressure::Normal,
        };
        let after_reading = MetricsReading {
            captured_at: Utc::now(),
            cpu_percent: 42.0,
            thermal_level: ThermalLevel::Normal,
            memory_used_bytes: 11 * 1_073_741_824,
            memory_pressure: MemoryPressure::Normal,
        };

        let result = apply_balance_plan(
            &plan.throttle_targets,
            &[],
            before,
            || after_reading.clone(),
            &mock,
            &mut ledger,
            Some(&tmp_ledger),
            501,
            9999,
        );

        assert_eq!(result.throttled.len(), 1);
        assert!(result.failures.is_empty());
        assert!((result.cpu_delta_percent - (-36.0)).abs() < 0.01);
        assert_eq!(*mock.backgrounded.borrow(), vec![200]);

        // Reload from disk to verify persistence.
        let mut reloaded = UndoLedger::load_from(&tmp_ledger);
        assert!(reloaded.is_throttled(200));

        let undo = undo_balance(Some(&result.batch_id), &mock, &mut reloaded, Some(&tmp_ledger));
        assert_eq!(undo.restored_pids, vec![200]);
        assert!(undo.failures.is_empty());
        assert!(!reloaded.has_pending_undo());

        let _ = fs::remove_file(&tmp_ledger);
    }

    #[test]
    fn live_taskpolicy_sets_and_restores_child_process() {
        let mut child = Command::new("/bin/sleep")
            .arg("20")
            .spawn()
            .expect("spawn sleep");
        let pid = child.id();

        let exec = SystemTaskPolicyExecutor;
        exec.set_background(pid).expect("taskpolicy -b on child");
        exec.clear_background(pid).expect("taskpolicy -B on child");

        let _ = child.kill();
        let _ = child.wait();
    }
}
