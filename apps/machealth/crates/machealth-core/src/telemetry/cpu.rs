use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::thread;
use std::time::Duration;

const PROCESSOR_CPU_LOAD_INFO: libc::c_int = 2;
const CPU_STATE_USER: usize = 0;
const CPU_STATE_SYSTEM: usize = 1;
const CPU_STATE_IDLE: usize = 2;
const CPU_STATE_NICE: usize = 3;
const CPU_STATE_MAX: usize = 4;
const KERN_SUCCESS: libc::c_int = 0;

unsafe extern "C" {
    fn mach_host_self() -> u32;
    fn mach_task_self() -> u32;
    fn host_processor_info(
        host: u32,
        flavor: libc::c_int,
        out_processor_count: *mut u32,
        out_processor_info: *mut *mut libc::c_int,
        out_processor_info_cnt: *mut u32,
    ) -> libc::c_int;
    fn vm_deallocate(target_task: u32, address: usize, size: usize) -> libc::c_int;
}

/// Raw cumulative CPU ticks for a single logical core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CoreTicks {
    pub user: u64,
    pub system: u64,
    pub idle: u64,
    pub nice: u64,
}

impl CoreTicks {
    pub fn total(&self) -> u64 {
        self.user
            .saturating_add(self.system)
            .saturating_add(self.idle)
            .saturating_add(self.nice)
    }

    pub fn active(&self) -> u64 {
        self.user
            .saturating_add(self.system)
            .saturating_add(self.nice)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreKind {
    Performance,
    Efficiency,
    Standard,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoreUsage {
    pub index: usize,
    pub kind: CoreKind,
    pub total_percent: f32,
    pub user_percent: f32,
    pub system_percent: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadLevel {
    Light,
    Moderate,
    Heavy,
}

impl LoadLevel {
    pub fn from_cpu_percent(percent: f32) -> Self {
        if percent >= 75.0 {
            Self::Heavy
        } else if percent >= 35.0 {
            Self::Moderate
        } else {
            Self::Light
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Moderate => "Moderate",
            Self::Heavy => "Heavy",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpuTopology {
    pub logical_cores: u32,
    pub physical_cores: u32,
    pub performance_cores: u32,
    pub efficiency_cores: u32,
    pub is_apple_silicon: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CpuTelemetry {
    pub total_percent: f32,
    pub user_percent: f32,
    pub system_percent: f32,
    pub idle_percent: f32,
    pub performance_percent: Option<f32>,
    pub efficiency_percent: Option<f32>,
    pub load_level: LoadLevel,
    pub summary: String,
    pub topology: CpuTopology,
    pub cores: Vec<CoreUsage>,
}

/// Stateful CPU sampler that computes live usage deltas between ticks.
#[derive(Debug, Clone)]
pub struct CpuSampler {
    topology: CpuTopology,
    previous: Option<Vec<CoreTicks>>,
}

impl Default for CpuSampler {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuSampler {
    pub fn new() -> Self {
        let topology = read_cpu_topology();
        let previous = read_core_ticks().ok();
        Self { topology, previous }
    }

    pub fn topology(&self) -> &CpuTopology {
        &self.topology
    }

    /// Reads current ticks and computes usage since the previous sample.
    /// If no previous sample exists, returns a zero-delta snapshot and primes state.
    pub fn sample(&mut self) -> Result<CpuTelemetry, String> {
        let current = read_core_ticks()?;
        let prev = self.previous.replace(current.clone());
        match prev {
            Some(prev_ticks) => Ok(compute_cpu_telemetry(&prev_ticks, &current, &self.topology)),
            None => Ok(compute_cpu_telemetry(&current, &current, &self.topology)),
        }
    }

    /// Takes a fresh measurement over `interval` when a one-shot reading is needed.
    pub fn sample_over(&mut self, interval: Duration) -> Result<CpuTelemetry, String> {
        let first = read_core_ticks()?;
        thread::sleep(interval);
        let second = read_core_ticks()?;
        self.previous = Some(second.clone());
        Ok(compute_cpu_telemetry(&first, &second, &self.topology))
    }
}

pub fn read_cpu_topology() -> CpuTopology {
    let logical_cores = sysctl_u32("hw.logicalcpu")
        .or_else(|| sysctl_u32("hw.ncpu"))
        .unwrap_or(1);
    let physical_cores = sysctl_u32("hw.physicalcpu").unwrap_or(logical_cores);
    let perf_levels = sysctl_u32("hw.nperflevels").unwrap_or(0);

    let mut performance_cores = 0;
    let mut efficiency_cores = 0;

    if perf_levels >= 2 {
        for idx in 0..perf_levels {
            let name_key = format!("hw.perflevel{idx}.name");
            let cpu_key = format!("hw.perflevel{idx}.logicalcpu");
            let count = sysctl_u32(&cpu_key).unwrap_or(0);
            let name = sysctl_string(&name_key).unwrap_or_default();
            if name.eq_ignore_ascii_case("Performance") {
                performance_cores += count;
            } else if name.eq_ignore_ascii_case("Efficiency") {
                efficiency_cores += count;
            }
        }
    }

    let is_apple_silicon = cfg!(target_arch = "aarch64") || (performance_cores > 0 && efficiency_cores > 0);

    CpuTopology {
        logical_cores,
        physical_cores,
        performance_cores,
        efficiency_cores,
        is_apple_silicon,
    }
}

pub fn read_core_ticks() -> Result<Vec<CoreTicks>, String> {
    let mut num_cpus: u32 = 0;
    let mut cpu_info: *mut libc::c_int = ptr::null_mut();
    let mut num_cpu_info: u32 = 0;

    let kr = unsafe {
        host_processor_info(
            mach_host_self(),
            PROCESSOR_CPU_LOAD_INFO,
            &mut num_cpus,
            &mut cpu_info,
            &mut num_cpu_info,
        )
    };

    if kr != KERN_SUCCESS || cpu_info.is_null() {
        return Err(format!("host_processor_info failed with kern_return_t={kr}"));
    }

    let count = num_cpus as usize;
    let slice_len = num_cpu_info as usize;
    let mut result = Vec::with_capacity(count);

    unsafe {
        let slice = std::slice::from_raw_parts(cpu_info, slice_len);
        for i in 0..count {
            let base = i * CPU_STATE_MAX;
            if base + CPU_STATE_MAX <= slice.len() {
                result.push(CoreTicks {
                    user: slice[base + CPU_STATE_USER] as u32 as u64,
                    system: slice[base + CPU_STATE_SYSTEM] as u32 as u64,
                    idle: slice[base + CPU_STATE_IDLE] as u32 as u64,
                    nice: slice[base + CPU_STATE_NICE] as u32 as u64,
                });
            }
        }
        let byte_size = slice_len * mem::size_of::<libc::c_int>();
        let _ = vm_deallocate(mach_task_self(), cpu_info as usize, byte_size);
    }

    Ok(result)
}

pub fn compute_cpu_telemetry(
    prev: &[CoreTicks],
    curr: &[CoreTicks],
    topology: &CpuTopology,
) -> CpuTelemetry {
    let n = prev.len().min(curr.len());
    let mut cores = Vec::with_capacity(n);

    let mut total_user: u64 = 0;
    let mut total_sys: u64 = 0;
    let mut total_idle: u64 = 0;
    let mut total_nice: u64 = 0;

    let e_count = topology.efficiency_cores as usize;
    let p_count = topology.performance_cores as usize;
    let has_hybrid = e_count > 0 && p_count > 0 && (e_count + p_count) == n;

    let mut p_active_sum = 0.0f32;
    let mut p_cores_seen = 0usize;
    let mut e_active_sum = 0.0f32;
    let mut e_cores_seen = 0usize;

    for i in 0..n {
        let d_user = (curr[i].user as u32).wrapping_sub(prev[i].user as u32) as u64;
        let d_sys = (curr[i].system as u32).wrapping_sub(prev[i].system as u32) as u64;
        let d_idle = (curr[i].idle as u32).wrapping_sub(prev[i].idle as u32) as u64;
        let d_nice = (curr[i].nice as u32).wrapping_sub(prev[i].nice as u32) as u64;

        total_user = total_user.saturating_add(d_user);
        total_sys = total_sys.saturating_add(d_sys);
        total_idle = total_idle.saturating_add(d_idle);
        total_nice = total_nice.saturating_add(d_nice);

        let d_total = d_user + d_sys + d_idle + d_nice;
        let (user_pct, sys_pct, tot_pct) = if d_total > 0 {
            let u = ((d_user + d_nice) as f64 / d_total as f64 * 100.0) as f32;
            let s = (d_sys as f64 / d_total as f64 * 100.0) as f32;
            (u, s, (u + s).clamp(0.0, 100.0))
        } else {
            (0.0, 0.0, 0.0)
        };

        // On Apple Silicon XNU, cluster 0 (indices 0..e_count) are Efficiency cores,
        // followed by Performance clusters (e_count..n).
        let kind = if has_hybrid {
            if i < e_count {
                e_active_sum += tot_pct;
                e_cores_seen += 1;
                CoreKind::Efficiency
            } else {
                p_active_sum += tot_pct;
                p_cores_seen += 1;
                CoreKind::Performance
            }
        } else {
            CoreKind::Standard
        };

        cores.push(CoreUsage {
            index: i,
            kind,
            total_percent: tot_pct,
            user_percent: user_pct,
            system_percent: sys_pct,
        });
    }

    let grand_total = total_user + total_sys + total_idle + total_nice;
    let (user_percent, system_percent, idle_percent, total_percent) = if grand_total > 0 {
        let u = ((total_user + total_nice) as f64 / grand_total as f64 * 100.0) as f32;
        let s = (total_sys as f64 / grand_total as f64 * 100.0) as f32;
        let idle = (total_idle as f64 / grand_total as f64 * 100.0) as f32;
        (u, s, idle, (u + s).clamp(0.0, 100.0))
    } else {
        (0.0, 0.0, 100.0, 0.0)
    };

    let performance_percent = (p_cores_seen > 0).then(|| p_active_sum / p_cores_seen as f32);
    let efficiency_percent = (e_cores_seen > 0).then(|| e_active_sum / e_cores_seen as f32);
    let load_level = LoadLevel::from_cpu_percent(total_percent);

    let summary = match load_level {
        LoadLevel::Light => "Processor is mostly idle".to_string(),
        LoadLevel::Moderate => "Processor is working moderately".to_string(),
        LoadLevel::Heavy => "Processor is under heavy load".to_string(),
    };

    CpuTelemetry {
        total_percent,
        user_percent,
        system_percent,
        idle_percent,
        performance_percent,
        efficiency_percent,
        load_level,
        summary,
        topology: topology.clone(),
        cores,
    }
}

pub(crate) fn sysctl_u32(name: &str) -> Option<u32> {
    let cname = CString::new(name).ok()?;
    let mut value: u32 = 0;
    let mut size = mem::size_of::<u32>();
    let ret = unsafe {
        libc::sysctlbyname(
            cname.as_ptr(),
            &mut value as *mut u32 as *mut libc::c_void,
            &mut size,
            ptr::null_mut(),
            0,
        )
    };
    (ret == 0).then_some(value)
}

pub(crate) fn sysctl_i32(name: &str) -> Option<i32> {
    let cname = CString::new(name).ok()?;
    let mut value: i32 = 0;
    let mut size = mem::size_of::<i32>();
    let ret = unsafe {
        libc::sysctlbyname(
            cname.as_ptr(),
            &mut value as *mut i32 as *mut libc::c_void,
            &mut size,
            ptr::null_mut(),
            0,
        )
    };
    (ret == 0).then_some(value)
}

pub(crate) fn sysctl_u64(name: &str) -> Option<u64> {
    let cname = CString::new(name).ok()?;
    let mut value: u64 = 0;
    let mut size = mem::size_of::<u64>();
    let ret = unsafe {
        libc::sysctlbyname(
            cname.as_ptr(),
            &mut value as *mut u64 as *mut libc::c_void,
            &mut size,
            ptr::null_mut(),
            0,
        )
    };
    (ret == 0).then_some(value)
}

pub(crate) fn sysctl_string(name: &str) -> Option<String> {
    let cname = CString::new(name).ok()?;
    let mut size: usize = 0;
    let ret = unsafe {
        libc::sysctlbyname(
            cname.as_ptr(),
            ptr::null_mut(),
            &mut size,
            ptr::null_mut(),
            0,
        )
    };
    if ret != 0 || size == 0 {
        return None;
    }
    let mut buf = vec![0u8; size];
    let ret = unsafe {
        libc::sysctlbyname(
            cname.as_ptr(),
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut size,
            ptr::null_mut(),
            0,
        )
    };
    if ret != 0 {
        return None;
    }
    while buf.last() == Some(&0) {
        buf.pop();
    }
    String::from_utf8(buf).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_hybrid_core_percentages() {
        let topo = CpuTopology {
            logical_cores: 4,
            physical_cores: 4,
            performance_cores: 2,
            efficiency_cores: 2,
            is_apple_silicon: true,
        };
        let prev = vec![
            CoreTicks { user: 100, system: 50, idle: 850, nice: 0 },
            CoreTicks { user: 100, system: 50, idle: 850, nice: 0 },
            CoreTicks { user: 100, system: 50, idle: 850, nice: 0 },
            CoreTicks { user: 100, system: 50, idle: 850, nice: 0 },
        ];
        // E-cores: 20% busy each; P-cores: 80% busy each -> overall 50%
        let curr = vec![
            CoreTicks { user: 115, system: 55, idle: 930, nice: 0 },
            CoreTicks { user: 115, system: 55, idle: 930, nice: 0 },
            CoreTicks { user: 160, system: 70, idle: 870, nice: 0 },
            CoreTicks { user: 160, system: 70, idle: 870, nice: 0 },
        ];

        let cpu = compute_cpu_telemetry(&prev, &curr, &topo);
        assert!((cpu.total_percent - 50.0).abs() < 0.1);
        assert!((cpu.efficiency_percent.unwrap() - 20.0).abs() < 0.1);
        assert!((cpu.performance_percent.unwrap() - 80.0).abs() < 0.1);
        assert_eq!(cpu.load_level, LoadLevel::Moderate);
        assert_eq!(cpu.cores[0].kind, CoreKind::Efficiency);
        assert_eq!(cpu.cores[2].kind, CoreKind::Performance);
    }

    #[test]
    fn reads_live_macos_cpu() {
        let mut sampler = CpuSampler::new();
        let cpu = sampler.sample_over(Duration::from_millis(80)).expect("live cpu sample");
        assert!(cpu.topology.logical_cores >= 1);
        assert_eq!(cpu.cores.len(), cpu.topology.logical_cores as usize);
        assert!((0.0..=100.0).contains(&cpu.total_percent));
    }
}
