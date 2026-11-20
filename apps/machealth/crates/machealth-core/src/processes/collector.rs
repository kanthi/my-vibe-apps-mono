use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::CStr;
use std::process::Command;
use std::time::Instant;

const RUSAGE_INFO_V4: libc::c_int = 4;
const PROC_PIDPATHINFO_MAXSIZE: u32 = 4096;

/// Matches `struct rusage_info_v4` from `<sys/resource.h>` (288 bytes, natural 8-byte alignment).
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct RusageInfoV4 {
    pub ri_uuid: [u8; 16],
    pub ri_user_time: u64,
    pub ri_system_time: u64,
    pub ri_pkg_idle_wkups: u64,
    pub ri_interrupt_wkups: u64,
    pub ri_pageins: u64,
    pub ri_wired_size: u64,
    pub ri_resident_size: u64,
    pub ri_phys_footprint: u64,
    pub ri_proc_start_abstime: u64,
    pub ri_proc_exit_abstime: u64,
    pub ri_child_user_time: u64,
    pub ri_child_system_time: u64,
    pub ri_child_pkg_idle_wkups: u64,
    pub ri_child_interrupt_wkups: u64,
    pub ri_child_pageins: u64,
    pub ri_child_elapsed_abstime: u64,
    pub ri_diskio_bytesread: u64,
    pub ri_diskio_byteswritten: u64,
    pub ri_cpu_time_qos_default: u64,
    pub ri_cpu_time_qos_maintenance: u64,
    pub ri_cpu_time_qos_background: u64,
    pub ri_cpu_time_qos_utility: u64,
    pub ri_cpu_time_qos_legacy: u64,
    pub ri_cpu_time_qos_user_initiated: u64,
    pub ri_cpu_time_qos_user_interactive: u64,
    pub ri_billed_system_time: u64,
    pub ri_serviced_system_time: u64,
    pub ri_logical_writes: u64,
    pub ri_lifetime_max_phys_footprint: u64,
    pub ri_instructions: u64,
    pub ri_cycles: u64,
    pub ri_billed_energy: u64,
    pub ri_serviced_energy: u64,
    pub ri_interval_max_phys_footprint: u64,
    pub ri_runnable_time: u64,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct MachTimebaseInfo {
    numer: u32,
    denom: u32,
}

unsafe extern "C" {
    fn proc_pid_rusage(pid: libc::c_int, flavor: libc::c_int, buffer: *mut libc::c_void) -> libc::c_int;
    fn proc_pidpath(pid: libc::c_int, buffer: *mut libc::c_void, buffersize: u32) -> libc::c_int;
    fn mach_timebase_info(info: *mut MachTimebaseInfo) -> libc::c_int;
}

#[derive(Debug, Clone)]
pub struct RawProcessRow {
    pub pid: u32,
    pub ppid: u32,
    pub uid: u32,
    pub ps_cpu_percent: f32,
    pub rss_bytes: u64,
    pub uptime_seconds: u64,
    pub command_line: String,
    pub executable_path: Option<String>,
    pub rusage: Option<ProcessRusage>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ProcessRusage {
    pub cpu_time_ns: u64,
    pub phys_footprint_bytes: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub wakeups: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessNetCounters {
    pub bytes_in: u64,
    pub bytes_out: u64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PrevProcSample {
    pub cpu_time_ns: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub wakeups: u64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
    pub captured_at: Instant,
}

pub fn read_pid_path(pid: u32) -> Option<String> {
    let mut buf = vec![0u8; PROC_PIDPATHINFO_MAXSIZE as usize];
    let ret = unsafe {
        proc_pidpath(
            pid as libc::c_int,
            buf.as_mut_ptr() as *mut libc::c_void,
            PROC_PIDPATHINFO_MAXSIZE,
        )
    };
    if ret <= 0 {
        return None;
    }
    let slice = &buf[..ret as usize];
    let cstr_len = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
    String::from_utf8(slice[..cstr_len].to_vec()).ok()
}

pub fn read_pid_rusage(pid: u32) -> Option<ProcessRusage> {
    let mut info = RusageInfoV4::default();
    let ret = unsafe {
        proc_pid_rusage(
            pid as libc::c_int,
            RUSAGE_INFO_V4,
            &mut info as *mut RusageInfoV4 as *mut libc::c_void,
        )
    };
    if ret != 0 {
        return None;
    }
    let (numer, denom) = timebase_ratio();
    let raw_cpu = info.ri_user_time.saturating_add(info.ri_system_time);
    let cpu_time_ns = ((raw_cpu as u128).saturating_mul(numer as u128) / (denom.max(1) as u128)) as u64;

    Some(ProcessRusage {
        cpu_time_ns,
        phys_footprint_bytes: info.ri_phys_footprint,
        disk_read_bytes: info.ri_diskio_bytesread,
        disk_write_bytes: info.ri_diskio_byteswritten,
        wakeups: info
            .ri_pkg_idle_wkups
            .saturating_add(info.ri_interrupt_wkups),
    })
}

fn timebase_ratio() -> (u32, u32) {
    let mut info = MachTimebaseInfo::default();
    let ret = unsafe { mach_timebase_info(&mut info) };
    if ret == 0 && info.denom > 0 {
        (info.numer, info.denom)
    } else {
        (1, 1)
    }
}

pub fn collect_raw_processes() -> Result<Vec<RawProcessRow>, String> {
    let output = Command::new("/bin/ps")
        .args(["-axo", "pid=,ppid=,uid=,%cpu=,rss=,etime=,args="])
        .output()
        .map_err(|e| format!("failed to run ps: {e}"))?;
    if !output.status.success() {
        return Err(format!("ps exited with status {}", output.status));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut rows = parse_ps_output(&text);

    for row in &mut rows {
        row.executable_path = read_pid_path(row.pid);
        row.rusage = read_pid_rusage(row.pid);
    }

    Ok(rows)
}

pub fn parse_ps_output(text: &str) -> Vec<RawProcessRow> {
    let mut rows = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Columns: pid, ppid, uid, %cpu, rss (KiB), etime, args...
        let mut parts = trimmed.split_whitespace();
        let Some(pid) = parts.next().and_then(|s| s.parse::<u32>().ok()) else {
            continue;
        };
        let Some(ppid) = parts.next().and_then(|s| s.parse::<u32>().ok()) else {
            continue;
        };
        let Some(uid) = parts.next().and_then(|s| s.parse::<u32>().ok()) else {
            continue;
        };
        let Some(ps_cpu_percent) = parts.next().and_then(|s| s.parse::<f32>().ok()) else {
            continue;
        };
        let Some(rss_kib) = parts.next().and_then(|s| s.parse::<u64>().ok()) else {
            continue;
        };
        let Some(etime_str) = parts.next() else {
            continue;
        };
        let uptime_seconds = parse_etime_seconds(etime_str).unwrap_or(0);

        // Preserve exact remaining command line after the 6th whitespace token.
        let command_line = extract_after_n_tokens(trimmed, 6).to_string();
        if command_line.is_empty() {
            continue;
        }

        rows.push(RawProcessRow {
            pid,
            ppid,
            uid,
            ps_cpu_percent,
            rss_bytes: rss_kib.saturating_mul(1024),
            uptime_seconds,
            command_line,
            executable_path: None,
            rusage: None,
        });
    }
    rows
}

fn extract_after_n_tokens(s: &str, n: usize) -> &str {
    let mut in_ws = true;
    let mut tokens = 0usize;
    for (idx, ch) in s.char_indices() {
        if ch.is_whitespace() {
            in_ws = true;
        } else if in_ws {
            in_ws = false;
            tokens += 1;
            if tokens == n + 1 {
                return &s[idx..];
            }
        }
    }
    ""
}

/// Parses `ps` `etime` formats: `MM:SS`, `HH:MM:SS`, or `DD-HH:MM:SS`.
pub fn parse_etime_seconds(etime: &str) -> Option<u64> {
    let (days, rest) = match etime.split_once('-') {
        Some((d, r)) => (d.parse::<u64>().ok()?, r),
        None => (0, etime),
    };
    let nums: Vec<u64> = rest
        .split(':')
        .map(|p| p.parse::<u64>().ok())
        .collect::<Option<Vec<_>>>()?;
    let secs = match nums.as_slice() {
        [m, s] => m * 60 + s,
        [h, m, s] => h * 3600 + m * 60 + s,
        _ => return None,
    };
    Some(days * 86_400 + secs)
}

/// Collects per-process cumulative network bytes using macOS `nettop` (unprivileged).
pub fn collect_process_network_bytes() -> HashMap<u32, ProcessNetCounters> {
    let Ok(output) = Command::new("/usr/bin/nettop")
        .args(["-P", "-L", "1", "-J", "bytes_in,bytes_out"])
        .output()
    else {
        return HashMap::new();
    };
    if !output.status.success() {
        return HashMap::new();
    }
    let text = String::from_utf8_lossy(&output.stdout);
    parse_nettop_output(&text)
}

pub fn parse_nettop_output(text: &str) -> HashMap<u32, ProcessNetCounters> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(',') {
            continue;
        }
        // Format: `mDNSResponder.481,12705933,8511458,`
        let mut cols = trimmed.split(',');
        let Some(name_pid) = cols.next() else { continue };
        let Some(bytes_in_str) = cols.next() else { continue };
        let Some(bytes_out_str) = cols.next() else { continue };

        let Some((_, pid_str)) = name_pid.rsplit_once('.') else {
            continue;
        };
        let Ok(pid) = pid_str.parse::<u32>() else {
            continue;
        };
        let bytes_in = bytes_in_str.trim().parse::<u64>().unwrap_or(0);
        let bytes_out = bytes_out_str.trim().parse::<u64>().unwrap_or(0);

        let entry = map.entry(pid).or_insert_with(ProcessNetCounters::default);
        entry.bytes_in = entry.bytes_in.saturating_add(bytes_in);
        entry.bytes_out = entry.bytes_out.saturating_add(bytes_out);
    }
    map
}

/// Returns the PID of the currently frontmost macOS application (if detectable without permissions).
pub fn frontmost_app_pid() -> Option<u32> {
    let output = Command::new("/usr/bin/lsappinfo")
        .args(["info", "-only", "pid", "front"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    // Format: `"pid"=12345`
    text.split('=')
        .nth(1)
        .and_then(|s| s.trim().trim_matches('"').parse::<u32>().ok())
}

pub fn current_uid() -> u32 {
    unsafe { libc::getuid() }
}

pub fn current_pid() -> u32 {
    std::process::id()
}

/// Helper to read the executable basename cleanly from path or command line.
pub fn extract_process_name(executable_path: Option<&str>, command_line: &str) -> String {
    if let Some(path) = executable_path {
        if let Some(base) = path.rsplit('/').next()
            && !base.is_empty()
        {
            return base.to_string();
        }
    }
    // Fall back to first argument basename.
    let first = command_line.split_whitespace().next().unwrap_or("unknown");
    let stripped = first.trim_start_matches('-');
    stripped
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("unknown")
        .to_string()
}

/// Keep CStr imported cleanly if needed in future BSD extensions.
#[allow(dead_code)]
fn _cstr_helper(ptr: *const libc::c_char) -> String {
    unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rusage_info_v4_abi_size() {
        assert_eq!(std::mem::size_of::<RusageInfoV4>(), 296);
    }

    #[test]
    fn parses_etime_formats() {
        assert_eq!(parse_etime_seconds("04:15"), Some(255));
        assert_eq!(parse_etime_seconds("02:10:05"), Some(7805));
        assert_eq!(parse_etime_seconds("02-04:09:20"), Some(2 * 86_400 + 4 * 3600 + 9 * 60 + 20));
    }

    #[test]
    fn parses_ps_and_nettop_output() {
        let ps = "\
    1     0     0   0.0  19056 02-04:09:20 /sbin/launchd
  481     1   501  12.4 204800    01:15:00 /Applications/Google Chrome.app/Contents/MacOS/Google Chrome --flag=1
";
        let rows = parse_ps_output(ps);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].pid, 481);
        assert_eq!(rows[1].uid, 501);
        assert!((rows[1].ps_cpu_percent - 12.4).abs() < 0.01);
        assert_eq!(rows[1].rss_bytes, 204_800 * 1024);
        assert!(rows[1].command_line.contains("--flag=1"));

        let nettop = "\
,bytes_in,bytes_out,
mDNSResponder.481,12705933,8511458,
apsd.379,164940,593515,
";
        let net = parse_nettop_output(nettop);
        assert_eq!(net[&481].bytes_in, 12_705_933);
        assert_eq!(net[&379].bytes_out, 593_515);
    }

    #[test]
    fn reads_current_process_rusage_and_path() {
        let pid = current_pid();
        let path = read_pid_path(pid);
        assert!(path.is_some());
        let rusage = read_pid_rusage(pid).expect("rusage for current pid");
        assert!(rusage.phys_footprint_bytes > 0);
    }
}
