pub mod battery;
pub mod cpu;
pub mod health_inputs;
pub mod memory;
pub mod network;
pub mod sleep;
pub mod storage;
pub mod thermal;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use battery::{BatteryTelemetry, collect_battery};
use cpu::{CpuSampler, CpuTelemetry};
use health_inputs::{
    SecuritySummaryTelemetry, StabilityTelemetry, StartupItemsTelemetry, collect_security_summary,
    collect_stability, collect_startup_items,
};
use memory::{MemoryTelemetry, collect_memory};
use network::{NetworkSampler, NetworkTelemetry};
use sleep::{SleepTelemetry, collect_sleep};
use storage::{StorageTelemetry, collect_storage};
use thermal::{ThermalTelemetry, collect_thermal};

/// Complete point-in-time telemetry snapshot used by both the live UI and the 8-area scorer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub captured_at: DateTime<Utc>,
    pub cpu: CpuTelemetry,
    pub thermal: ThermalTelemetry,
    pub memory: MemoryTelemetry,
    pub storage: StorageTelemetry,
    pub network: NetworkTelemetry,
    pub battery: BatteryTelemetry,
    pub sleep: SleepTelemetry,
    pub stability: StabilityTelemetry,
    pub startup_items: StartupItemsTelemetry,
    pub security: SecuritySummaryTelemetry,
}

/// Stateful collector meant to be polled every 1–2 seconds for live dashboard updates.
#[derive(Debug, Clone)]
pub struct TelemetryCollector {
    cpu: CpuSampler,
    network: NetworkSampler,
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryCollector {
    pub fn new() -> Self {
        Self {
            cpu: CpuSampler::new(),
            network: NetworkSampler::new(),
        }
    }

    /// Collects a snapshot using deltas since the previous `collect()` call.
    pub fn collect(&mut self) -> Result<SystemSnapshot, String> {
        let cpu = self.cpu.sample()?;
        self.collect_with_cpu(cpu)
    }

    /// Collects a one-shot snapshot with a short CPU/network sampling window.
    pub fn collect_once(&mut self, sample_window: Duration) -> Result<SystemSnapshot, String> {
        let _ = self.network.sample();
        let cpu = self.cpu.sample_over(sample_window)?;
        self.collect_with_cpu(cpu)
    }

    fn collect_with_cpu(&mut self, cpu: CpuTelemetry) -> Result<SystemSnapshot, String> {
        let battery = collect_battery();
        let thermal = collect_thermal(battery.temperature_celsius);
        let memory = collect_memory()?;
        let storage = collect_storage()?;
        let network = self.network.sample()?;
        let sleep = collect_sleep(battery.power_source);
        let stability = collect_stability();
        let startup_items = collect_startup_items();
        let security = collect_security_summary();

        Ok(SystemSnapshot {
            captured_at: Utc::now(),
            cpu,
            thermal,
            memory,
            storage,
            network,
            battery,
            sleep,
            stability,
            startup_items,
            security,
        })
    }
}
