use super::battery::PowerSource;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SleepAssertion {
    pub pid: u32,
    pub process_name: String,
    pub assertion_type: String,
    pub reason: String,
    pub duration_seconds: u64,
    pub created_for_pid: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SleepTimers {
    pub display_sleep_minutes: Option<u32>,
    pub system_sleep_minutes: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SleepTelemetry {
    pub prevent_system_sleep: bool,
    pub prevent_display_sleep: bool,
    pub active_assertions: Vec<SleepAssertion>,
    pub battery_timers: SleepTimers,
    pub ac_timers: SleepTimers,
    pub active_timers: SleepTimers,
    pub summary: String,
}

pub fn collect_sleep(power_source: PowerSource) -> SleepTelemetry {
    let assertions_text = Command::new("/usr/bin/pmset")
        .args(["-g", "assertions"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();

    let custom_text = Command::new("/usr/bin/pmset")
        .args(["-g", "custom"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();

    parse_sleep_telemetry(&assertions_text, &custom_text, power_source)
}

pub fn parse_sleep_telemetry(
    assertions_text: &str,
    custom_text: &str,
    power_source: PowerSource,
) -> SleepTelemetry {
    let (prevent_system_sleep, prevent_display_sleep, active_assertions) =
        parse_pmset_assertions(assertions_text);
    let (battery_timers, ac_timers) = parse_pmset_custom(custom_text);
    let active_timers = match power_source {
        PowerSource::BatteryPower => battery_timers,
        PowerSource::AcPower => ac_timers,
    };

    let non_powerd: Vec<&SleepAssertion> = active_assertions
        .iter()
        .filter(|a| a.process_name != "powerd")
        .collect();

    let summary = if non_powerd.is_empty() {
        "Your Mac can sleep normally".to_string()
    } else if non_powerd.len() == 1 {
        format!("{} is keeping your Mac awake", non_powerd[0].process_name)
    } else {
        format!(
            "{} apps are keeping your Mac awake ({})",
            non_powerd.len(),
            non_powerd[0].process_name
        )
    };

    SleepTelemetry {
        prevent_system_sleep,
        prevent_display_sleep,
        active_assertions,
        battery_timers,
        ac_timers,
        active_timers,
        summary,
    }
}

pub fn parse_pmset_assertions(text: &str) -> (bool, bool, Vec<SleepAssertion>) {
    let mut prevent_system = false;
    let mut prevent_display = false;
    let mut assertions: Vec<SleepAssertion> = Vec::new();
    let mut in_process_section = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Listed by owning process:") {
            in_process_section = true;
            continue;
        }
        if trimmed.starts_with("Kernel Assertions:") {
            break;
        }

        if !in_process_section {
            if let Some(v) = trimmed.strip_prefix("PreventUserIdleSystemSleep") {
                prevent_system |= v.trim() == "1";
            } else if let Some(v) = trimmed.strip_prefix("PreventSystemSleep") {
                if !trimmed.starts_with("PreventSystemSleepSecurityIndicator") {
                    prevent_system |= v.trim() == "1";
                }
            } else if let Some(v) = trimmed.strip_prefix("PreventUserIdleDisplaySleep") {
                prevent_display |= v.trim() == "1";
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("Created for PID:") {
            if let Some(last) = assertions.last_mut() {
                last.created_for_pid = rest.trim().trim_end_matches('.').parse::<u32>().ok();
            }
            continue;
        }

        if trimmed.starts_with("pid ") {
            if let Some(parsed) = parse_assertion_line(trimmed) {
                if is_sleep_preventing_type(&parsed.assertion_type) {
                    assertions.push(parsed);
                }
            }
        }
    }

    (prevent_system, prevent_display, assertions)
}

fn is_sleep_preventing_type(assertion_type: &str) -> bool {
    matches!(
        assertion_type,
        "PreventUserIdleSystemSleep"
            | "PreventUserIdleDisplaySleep"
            | "PreventSystemSleep"
            | "NoIdleSleepAssertion"
            | "NoDisplaySleepAssertion"
    )
}

fn parse_assertion_line(line: &str) -> Option<SleepAssertion> {
    // Example:
    // pid 76616(caffeinate): [0x000140bc000185c0] 00:00:37 PreventUserIdleSystemSleep named: "caffeinate command-line tool"
    let rest = line.strip_prefix("pid ")?;
    let open_paren = rest.find('(')?;
    let close_paren = rest[open_paren + 1..].find(')')? + open_paren + 1;
    let pid = rest[..open_paren].trim().parse::<u32>().ok()?;
    let process_name = rest[open_paren + 1..close_paren].to_string();

    let after_bracket = rest.split(']').nth(1)?.trim();
    let mut parts = after_bracket.splitn(3, ' ');
    let duration_str = parts.next()?;
    let assertion_type = parts.next()?.to_string();
    let remainder = parts.next().unwrap_or("");

    let duration_seconds = parse_hms_duration(duration_str).unwrap_or(0);
    let reason = if let Some(start) = remainder.find('"') {
        let after_quote = &remainder[start + 1..];
        if let Some(end) = after_quote.rfind('"') {
            after_quote[..end].to_string()
        } else {
            after_quote.to_string()
        }
    } else {
        remainder.trim().to_string()
    };

    Some(SleepAssertion {
        pid,
        process_name,
        assertion_type,
        reason,
        duration_seconds,
        created_for_pid: None,
    })
}

fn parse_hms_duration(hms: &str) -> Option<u64> {
    let mut parts = hms.split(':');
    let h = parts.next()?.parse::<u64>().ok()?;
    let m = parts.next()?.parse::<u64>().ok()?;
    let s = parts.next()?.parse::<u64>().ok()?;
    Some(h * 3600 + m * 60 + s)
}

pub fn parse_pmset_custom(text: &str) -> (SleepTimers, SleepTimers) {
    #[derive(Clone, Copy)]
    enum Section {
        None,
        Battery,
        Ac,
    }

    let mut section = Section::None;
    let mut battery = SleepTimers::default();
    let mut ac = SleepTimers::default();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Battery Power:") {
            section = Section::Battery;
            continue;
        } else if trimmed.starts_with("AC Power:") {
            section = Section::Ac;
            continue;
        }

        let target = match section {
            Section::Battery => &mut battery,
            Section::Ac => &mut ac,
            Section::None => continue,
        };

        let mut parts = trimmed.split_whitespace();
        let Some(key) = parts.next() else { continue };
        let Some(val) = parts.next().and_then(|v| v.parse::<u32>().ok()) else {
            continue;
        };
        match key {
            "displaysleep" => target.display_sleep_minutes = Some(val),
            "sleep" => target.system_sleep_minutes = Some(val),
            _ => {}
        }
    }

    (battery, ac)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pmset_assertions_and_custom_timers() {
        let assertions = r#"
Assertion status system-wide:
   UserIsActive                   1
   PreventUserIdleDisplaySleep    0
   PreventSystemSleep             0
   PreventUserIdleSystemSleep     1
Listed by owning process:
   pid 408(WindowServer): [0x001] 00:00:02 UserIsActive named: "tickle"
   pid 76616(caffeinate): [0x002] 01:02:03 PreventUserIdleSystemSleep named: "caffeinate command-line tool"
	Created for PID: 51571.
Kernel Assertions: 0x100=MAGICWAKE
"#;
        let custom = r#"
Battery Power:
 displaysleep         2
 sleep                1
AC Power:
 displaysleep         10
 sleep                0
"#;
        let sleep = parse_sleep_telemetry(assertions, custom, PowerSource::AcPower);
        assert!(sleep.prevent_system_sleep);
        assert!(!sleep.prevent_display_sleep);
        assert_eq!(sleep.active_assertions.len(), 1);
        assert_eq!(sleep.active_assertions[0].pid, 76616);
        assert_eq!(sleep.active_assertions[0].process_name, "caffeinate");
        assert_eq!(sleep.active_assertions[0].duration_seconds, 3723);
        assert_eq!(sleep.active_assertions[0].created_for_pid, Some(51571));
        assert_eq!(sleep.battery_timers.display_sleep_minutes, Some(2));
        assert_eq!(sleep.ac_timers.display_sleep_minutes, Some(10));
        assert_eq!(sleep.ac_timers.system_sleep_minutes, Some(0));
    }

    #[test]
    fn reads_live_macos_sleep() {
        let sleep = collect_sleep(PowerSource::AcPower);
        assert!(!sleep.summary.is_empty());
    }
}
