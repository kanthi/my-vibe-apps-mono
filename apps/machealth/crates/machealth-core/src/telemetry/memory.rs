use super::cpu::{sysctl_i32, sysctl_u64};
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::mem;
use std::ptr;

const HOST_VM_INFO64: libc::c_int = 4;
const KERN_SUCCESS: libc::c_int = 0;

/// Matches `struct vm_statistics64` rev1 layout from `<mach/vm_statistics.h>`.
/// Exact size is 152 bytes = 38 `natural_t` (u32) words (`HOST_VM_INFO64_REV1_COUNT`).
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct VmStatistics64Rev1 {
    free_count: u32,
    active_count: u32,
    inactive_count: u32,
    wire_count: u32,
    zero_fill_count: u64,
    reactivations: u64,
    pageins: u64,
    pageouts: u64,
    faults: u64,
    cow_faults: u64,
    lookups: u64,
    hits: u64,
    purges: u64,
    purgeable_count: u32,
    speculative_count: u32,
    decompressions: u64,
    compressions: u64,
    swapins: u64,
    swapouts: u64,
    compressor_page_count: u32,
    throttled_count: u32,
    external_page_count: u32,
    internal_page_count: u32,
    total_uncompressed_pages_in_compressor: u64,
}

const HOST_VM_INFO64_REV1_COUNT: u32 =
    (mem::size_of::<VmStatistics64Rev1>() / mem::size_of::<u32>()) as u32;

/// Matches `struct xsw_usage` from `<sys/sysctl.h>`.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct XswUsage {
    xsu_total: u64,
    xsu_avail: u64,
    xsu_used: u64,
    xsu_pagesize: u32,
    xsu_encrypted: libc::c_int,
}

unsafe extern "C" {
    fn mach_host_self() -> u32;
    fn host_statistics64(
        host_priv: u32,
        flavor: libc::c_int,
        host_info64_out: *mut libc::c_int,
        host_info64_out_cnt: *mut u32,
    ) -> libc::c_int;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryPressure {
    Normal,
    Warning,
    Critical,
}

impl MemoryPressure {
    pub fn from_sysctl_level(level: i32) -> Self {
        match level {
            4.. => Self::Critical,
            2..=3 => Self::Warning,
            _ => Self::Normal,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Warning => "Tight",
            Self::Critical => "Critical",
        }
    }

    pub fn plain_english(&self) -> &'static str {
        match self {
            Self::Normal => "Apps have the memory they need",
            Self::Warning => "Memory is getting tight — macOS is compressing more data",
            Self::Critical => "Memory is critically low — closing apps will help",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryTelemetry {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub app_bytes: u64,
    pub wired_bytes: u64,
    pub compressed_bytes: u64,
    pub cached_bytes: u64,
    pub free_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_encrypted: bool,
    pub used_percent: f32,
    pub pressure: MemoryPressure,
    pub summary: String,
}

pub fn collect_memory() -> Result<MemoryTelemetry, String> {
    let total_bytes = sysctl_u64("hw.memsize")
        .ok_or_else(|| "failed to read hw.memsize".to_string())?;
    let page_size = sysctl_u64("vm.pagesize").unwrap_or(16_384);

    let mut stats = VmStatistics64Rev1::default();
    let mut count = HOST_VM_INFO64_REV1_COUNT;
    let kr = unsafe {
        host_statistics64(
            mach_host_self(),
            HOST_VM_INFO64,
            &mut stats as *mut VmStatistics64Rev1 as *mut libc::c_int,
            &mut count,
        )
    };
    if kr != KERN_SUCCESS {
        return Err(format!("host_statistics64(HOST_VM_INFO64) failed: {kr}"));
    }

    let swap = read_swap_usage().unwrap_or_default();
    let pressure_raw = sysctl_i32("kern.memorystatus_vm_pressure_level").unwrap_or(1);

    Ok(build_memory_telemetry(
        total_bytes,
        page_size,
        &stats,
        &swap,
        MemoryPressure::from_sysctl_level(pressure_raw),
    ))
}

fn read_swap_usage() -> Option<XswUsage> {
    let cname = CString::new("vm.swapusage").ok()?;
    let mut usage = XswUsage::default();
    let mut size = mem::size_of::<XswUsage>();
    let ret = unsafe {
        libc::sysctlbyname(
            cname.as_ptr(),
            &mut usage as *mut XswUsage as *mut libc::c_void,
            &mut size,
            ptr::null_mut(),
            0,
        )
    };
    (ret == 0).then_some(usage)
}

fn build_memory_telemetry(
    total_bytes: u64,
    page_size: u64,
    stats: &VmStatistics64Rev1,
    swap: &XswUsage,
    pressure: MemoryPressure,
) -> MemoryTelemetry {
    // Matches macOS Activity Monitor formulas:
    // App Memory = (internal - purgeable) * page_size
    // Wired = wire_count * page_size
    // Compressed = compressor_page_count * page_size
    // Cached Files = (external + purgeable) * page_size
    let app_pages = stats.internal_page_count.saturating_sub(stats.purgeable_count) as u64;
    let app_bytes = app_pages.saturating_mul(page_size);
    let wired_bytes = (stats.wire_count as u64).saturating_mul(page_size);
    let compressed_bytes = (stats.compressor_page_count as u64).saturating_mul(page_size);
    let cached_bytes = (stats.external_page_count as u64 + stats.purgeable_count as u64)
        .saturating_mul(page_size);
    let used_bytes = app_bytes
        .saturating_add(wired_bytes)
        .saturating_add(compressed_bytes)
        .min(total_bytes);
    let free_bytes = total_bytes.saturating_sub(used_bytes);
    let used_percent = if total_bytes > 0 {
        (used_bytes as f64 / total_bytes as f64 * 100.0) as f32
    } else {
        0.0
    };

    MemoryTelemetry {
        total_bytes,
        used_bytes,
        app_bytes,
        wired_bytes,
        compressed_bytes,
        cached_bytes,
        free_bytes,
        swap_total_bytes: swap.xsu_total,
        swap_used_bytes: swap.xsu_used,
        swap_encrypted: swap.xsu_encrypted != 0,
        used_percent,
        pressure,
        summary: pressure.plain_english().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vm_statistics64_rev1_abi_size() {
        assert_eq!(mem::size_of::<VmStatistics64Rev1>(), 152);
        assert_eq!(HOST_VM_INFO64_REV1_COUNT, 38);
    }

    #[test]
    fn computes_activity_monitor_memory_breakdown() {
        let page_size = 16_384;
        let total_bytes = 16 * 1024 * 1024 * 1024; // 16 GiB
        let stats = VmStatistics64Rev1 {
            internal_page_count: 300_000,
            purgeable_count: 50_000,
            wire_count: 150_000,
            compressor_page_count: 100_000,
            external_page_count: 200_000,
            ..Default::default()
        };
        let swap = XswUsage {
            xsu_total: 2 * 1024 * 1024 * 1024,
            xsu_used: 512 * 1024 * 1024,
            xsu_avail: 1536 * 1024 * 1024,
            xsu_pagesize: 16_384,
            xsu_encrypted: 1,
        };

        let mem = build_memory_telemetry(total_bytes, page_size, &stats, &swap, MemoryPressure::Normal);
        assert_eq!(mem.app_bytes, 250_000 * page_size);
        assert_eq!(mem.wired_bytes, 150_000 * page_size);
        assert_eq!(mem.compressed_bytes, 100_000 * page_size);
        assert_eq!(mem.used_bytes, 500_000 * page_size);
        assert_eq!(mem.cached_bytes, 250_000 * page_size);
        assert!(mem.swap_encrypted);
        assert_eq!(mem.pressure, MemoryPressure::Normal);
    }

    #[test]
    fn reads_live_macos_memory() {
        let mem = collect_memory().expect("live memory");
        assert!(mem.total_bytes >= 1024 * 1024 * 1024);
        assert!(mem.used_bytes > 0);
        assert!(mem.used_bytes <= mem.total_bytes);
        assert!((0.0..=100.0).contains(&mem.used_percent));
    }
}
