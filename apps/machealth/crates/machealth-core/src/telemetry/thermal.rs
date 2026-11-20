use serde::{Deserialize, Serialize};
use std::ffi::{CString, c_void};
use std::process::Command;

#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {
    fn objc_getClass(name: *const libc::c_char) -> *mut c_void;
    fn sel_registerName(name: *const libc::c_char) -> *mut c_void;
    fn objc_msgSend();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ThermalLevel {
    Normal,
    Warm,
    Hot,
    VeryHot,
}

impl ThermalLevel {
    pub fn from_ns_thermal_state(state: isize) -> Self {
        match state {
            0 => Self::Normal,
            1 => Self::Warm,
            2 => Self::Hot,
            _ => Self::VeryHot,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Warm => "Warm",
            Self::Hot => "Hot",
            Self::VeryHot => "Very Hot",
        }
    }

    pub fn plain_english(&self) -> &'static str {
        match self {
            Self::Normal => "The chip is running cool",
            Self::Warm => "The chip is running warm",
            Self::Hot => "The chip is running hot — performance may slow down",
            Self::VeryHot => "The chip is very hot — macOS is throttling to cool down",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThermalTelemetry {
    pub level: ThermalLevel,
    pub label: String,
    pub summary: String,
    pub cpu_speed_limit_percent: Option<u32>,
    pub pmset_thermal_level: Option<u32>,
    pub battery_temp_celsius: Option<f32>,
}

pub fn collect_thermal(battery_temp_celsius: Option<f32>) -> ThermalTelemetry {
    let ns_level = read_ns_process_thermal_state().unwrap_or(ThermalLevel::Normal);
    let (cpu_speed_limit_percent, pmset_thermal_level) = read_pmset_therm().unwrap_or((None, None));

    // Escalate if pmset reports active CPU throttling even if thermalState hasn't moved yet.
    let mut level = ns_level;
    if let Some(limit) = cpu_speed_limit_percent {
        if limit < 50 {
            level = level.max(ThermalLevel::VeryHot);
        } else if limit < 80 {
            level = level.max(ThermalLevel::Hot);
        } else if limit < 95 {
            level = level.max(ThermalLevel::Warm);
        }
    }

    ThermalTelemetry {
        level,
        label: level.label().to_string(),
        summary: level.plain_english().to_string(),
        cpu_speed_limit_percent,
        pmset_thermal_level,
        battery_temp_celsius,
    }
}

fn read_ns_process_thermal_state() -> Option<ThermalLevel> {
    let cls_name = CString::new("NSProcessInfo").ok()?;
    let process_info_sel = CString::new("processInfo").ok()?;
    let thermal_state_sel = CString::new("thermalState").ok()?;

    unsafe {
        let cls = objc_getClass(cls_name.as_ptr());
        if cls.is_null() {
            return None;
        }
        let sel_process_info = sel_registerName(process_info_sel.as_ptr());
        let sel_thermal_state = sel_registerName(thermal_state_sel.as_ptr());

        let msg_send_obj: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
            std::mem::transmute(objc_msgSend as *const ());
        let process_info = msg_send_obj(cls, sel_process_info);
        if process_info.is_null() {
            return None;
        }

        let msg_send_isize: unsafe extern "C" fn(*mut c_void, *mut c_void) -> isize =
            std::mem::transmute(objc_msgSend as *const ());
        let raw_state = msg_send_isize(process_info, sel_thermal_state);
        Some(ThermalLevel::from_ns_thermal_state(raw_state))
    }
}

fn read_pmset_therm() -> Option<(Option<u32>, Option<u32>)> {
    let output = Command::new("/usr/bin/pmset")
        .args(["-g", "therm"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Some(parse_pmset_therm(&text))
}

pub fn parse_pmset_therm(text: &str) -> (Option<u32>, Option<u32>) {
    let mut cpu_speed_limit = None;
    let mut thermal_level = None;

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix("CPU_Speed_Limit") {
            cpu_speed_limit = val
                .trim_start_matches(['=', ' ', '\t'])
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u32>().ok());
        } else if let Some(val) = trimmed.strip_prefix("Thermal_Level") {
            thermal_level = val
                .trim_start_matches(['=', ' ', '\t'])
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u32>().ok());
        }
    }

    (cpu_speed_limit, thermal_level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pmset_therm_normal_and_throttled() {
        let normal = "Note: No thermal warning level has been recorded\nNote: No CPU power status has been recorded\n";
        assert_eq!(parse_pmset_therm(normal), (None, None));

        let throttled = "CPU_Scheduler_Limit = 100\nCPU_Available_CPUs = 10\nCPU_Speed_Limit = 72\nThermal_Level = 45\n";
        assert_eq!(parse_pmset_therm(throttled), (Some(72), Some(45)));
    }

    #[test]
    fn reads_live_macos_thermal_state() {
        let thermal = collect_thermal(None);
        assert!(!thermal.label.is_empty());
        assert!(!thermal.summary.is_empty());
    }
}
