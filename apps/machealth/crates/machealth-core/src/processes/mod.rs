pub mod actions;
pub mod collector;
pub mod descriptions;
pub mod grouping;

use collector::{
    PrevProcSample, ProcessNetCounters, collect_process_network_bytes, collect_raw_processes,
    extract_process_name,
};
use descriptions::{ImpactLabel, describe_process};
use grouping::{extract_outer_app_bundle, group_processes, looks_like_helper_name, sort_groups};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

pub use actions::{
    GuardrailVerdict, QuitConfirmation, QuitOutcome, build_quit_confirmation, check_process_safety,
    quit_app_group, quit_process,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityTab {
    Cpu,
    Memory,
    Energy,
    Disk,
    Network,
}

impl ActivityTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Memory => "Memory",
            Self::Energy => "Energy",
            Self::Disk => "Disk",
            Self::Network => "Network",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub uid: u32,
    pub name: String,
    pub executable_path: Option<String>,
    pub command_line: String,
    pub app_bundle_name: Option<String>,
    pub app_bundle_path: Option<String>,
    pub is_helper: bool,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub energy_impact: f32,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub disk_read_bytes_per_sec: f64,
    pub disk_write_bytes_per_sec: f64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
    pub net_rx_bytes_per_sec: f64,
    pub net_tx_bytes_per_sec: f64,
    pub uptime_seconds: u64,
    pub impact_label: ImpactLabel,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppGroup {
    pub id: String,
    pub display_name: String,
    pub primary_pid: u32,
    pub bundle_path: Option<String>,
    pub description: String,
    pub impact_label: ImpactLabel,
    pub process_count: usize,
    pub helper_count: usize,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub energy_impact: f32,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub disk_read_bytes_per_sec: f64,
    pub disk_write_bytes_per_sec: f64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
    pub net_rx_bytes_per_sec: f64,
    pub net_tx_bytes_per_sec: f64,
    pub processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    pub processes: Vec<ProcessInfo>,
    pub groups: Vec<AppGroup>,
}

impl ProcessSnapshot {
    pub fn groups_for_tab(&self, tab: ActivityTab) -> Vec<AppGroup> {
        let mut cloned = self.groups.clone();
        sort_groups(&mut cloned, tab);
        cloned
    }

    pub fn top_groups(&self, tab: ActivityTab, limit: usize) -> Vec<AppGroup> {
        let mut sorted = self.groups_for_tab(tab);
        sorted.truncate(limit);
        sorted
    }
}

/// Stateful sampler that tracks per-process CPU, Disk I/O, Wakeup/Energy, and Network deltas.
#[derive(Debug, Default)]
pub struct ProcessSampler {
    previous: HashMap<u32, PrevProcSample>,
}

impl ProcessSampler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Collects all running processes and rolls them into parent+helper `AppGroup`s.
    /// Pass `include_network = true` when sampling the Network tab (invokes `nettop -P -L 1`).
    pub fn sample(&mut self, include_network: bool) -> Result<ProcessSnapshot, String> {
        let now = Instant::now();
        let raw_rows = collect_raw_processes()?;
        let net_map: HashMap<u32, ProcessNetCounters> = if include_network {
            collect_process_network_bytes()
        } else {
            HashMap::new()
        };

        let mut next_prev = HashMap::with_capacity(raw_rows.len());
        let mut processes = Vec::with_capacity(raw_rows.len());

        for row in raw_rows {
            let name = extract_process_name(row.executable_path.as_deref(), &row.command_line);
            let (app_bundle_name, app_bundle_path, nested_helper) = row
                .executable_path
                .as_deref()
                .map(extract_outer_app_bundle)
                .unwrap_or((None, None, false));
            let is_helper = nested_helper || looks_like_helper_name(&name);

            let memory_bytes = row
                .rusage
                .map(|r| r.phys_footprint_bytes)
                .filter(|&b| b > 0)
                .unwrap_or(row.rss_bytes);

            let disk_read_bytes = row.rusage.map(|r| r.disk_read_bytes).unwrap_or(0);
            let disk_write_bytes = row.rusage.map(|r| r.disk_write_bytes).unwrap_or(0);
            let cpu_time_ns = row.rusage.map(|r| r.cpu_time_ns).unwrap_or(0);
            let wakeups = row.rusage.map(|r| r.wakeups).unwrap_or(0);

            let prev_entry = self.previous.get(&row.pid).copied();
            let net_counters = net_map.get(&row.pid).copied().unwrap_or_else(|| {
                prev_entry
                    .map(|p| ProcessNetCounters {
                        bytes_in: p.net_rx_bytes,
                        bytes_out: p.net_tx_bytes,
                    })
                    .unwrap_or_default()
            });

            let (cpu_percent, disk_r_rate, disk_w_rate, net_rx_rate, net_tx_rate, wakeups_per_sec) =
                if let Some(prev) = prev_entry {
                    let elapsed = now.duration_since(prev.captured_at).as_secs_f64();
                    if elapsed > 0.05 {
                        let cpu_pct = if row.rusage.is_some() && cpu_time_ns >= prev.cpu_time_ns {
                            let delta_ns = cpu_time_ns.saturating_sub(prev.cpu_time_ns) as f64;
                            (delta_ns / (elapsed * 1_000_000_000.0) * 100.0) as f32
                        } else {
                            row.ps_cpu_percent
                        };
                        let dr = disk_read_bytes.saturating_sub(prev.disk_read_bytes) as f64 / elapsed;
                        let dw = disk_write_bytes.saturating_sub(prev.disk_write_bytes) as f64 / elapsed;
                        let nr = net_counters.bytes_in.saturating_sub(prev.net_rx_bytes) as f64 / elapsed;
                        let nt = net_counters.bytes_out.saturating_sub(prev.net_tx_bytes) as f64 / elapsed;
                        let wk = wakeups.saturating_sub(prev.wakeups) as f64 / elapsed;
                        (cpu_pct, dr, dw, nr, nt, wk)
                    } else {
                        (row.ps_cpu_percent, 0.0, 0.0, 0.0, 0.0, 0.0)
                    }
                } else {
                    (row.ps_cpu_percent, 0.0, 0.0, 0.0, 0.0, 0.0)
                };

            // Composite Energy Impact inspired by macOS Activity Monitor:
            // weighted CPU% + idle wakeup pressure + active disk/network I/O.
            let io_mb_per_sec = (disk_r_rate + disk_w_rate + net_rx_rate + net_tx_rate) / 1_048_576.0;
            let energy_impact = (cpu_percent as f64
                + (wakeups_per_sec * 0.02)
                + (io_mb_per_sec * 0.5))
                .max(0.0) as f32;

            let impact_label = ImpactLabel::classify(cpu_percent, memory_bytes, energy_impact);
            let description = describe_process(&name, app_bundle_name.as_deref(), &row.command_line);

            next_prev.insert(
                row.pid,
                PrevProcSample {
                    cpu_time_ns,
                    disk_read_bytes,
                    disk_write_bytes,
                    wakeups,
                    net_rx_bytes: net_counters.bytes_in,
                    net_tx_bytes: net_counters.bytes_out,
                    captured_at: now,
                },
            );

            processes.push(ProcessInfo {
                pid: row.pid,
                ppid: row.ppid,
                uid: row.uid,
                name,
                executable_path: row.executable_path,
                command_line: row.command_line,
                app_bundle_name,
                app_bundle_path,
                is_helper,
                cpu_percent,
                memory_bytes,
                energy_impact,
                disk_read_bytes,
                disk_write_bytes,
                disk_read_bytes_per_sec: disk_r_rate,
                disk_write_bytes_per_sec: disk_w_rate,
                net_rx_bytes: net_counters.bytes_in,
                net_tx_bytes: net_counters.bytes_out,
                net_rx_bytes_per_sec: net_rx_rate,
                net_tx_bytes_per_sec: net_tx_rate,
                uptime_seconds: row.uptime_seconds,
                impact_label,
                description,
            });
        }

        self.previous = next_prev;
        let groups = group_processes(&processes);

        Ok(ProcessSnapshot { processes, groups })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_live_process_groups_across_all_5_tabs() {
        let mut sampler = ProcessSampler::new();
        let snap = sampler.sample(false).expect("live process sample");
        assert!(snap.processes.len() > 20);
        assert!(!snap.groups.is_empty());

        for tab in [
            ActivityTab::Cpu,
            ActivityTab::Memory,
            ActivityTab::Energy,
            ActivityTab::Disk,
            ActivityTab::Network,
        ] {
            let top = snap.top_groups(tab, 5);
            assert!(!top.is_empty());
        }
    }
}
