use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerSource {
    AcPower,
    BatteryPower,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatteryTelemetry {
    pub has_battery: bool,
    pub power_source: PowerSource,
    pub charge_percent: Option<u8>,
    pub health_percent: Option<u8>,
    pub cycle_count: Option<u32>,
    pub design_cycle_count: Option<u32>,
    pub design_capacity_mah: Option<u32>,
    pub nominal_max_capacity_mah: Option<u32>,
    pub raw_max_capacity_mah: Option<u32>,
    pub is_charging: bool,
    pub external_connected: bool,
    pub time_remaining_minutes: Option<u32>,
    pub temperature_celsius: Option<f32>,
    pub summary: String,
}

pub fn collect_battery() -> BatteryTelemetry {
    let ioreg_text = Command::new("/usr/sbin/ioreg")
        .args(["-r", "-c", "AppleSmartBattery"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();

    parse_ioreg_battery(&ioreg_text)
}

pub fn parse_ioreg_battery(text: &str) -> BatteryTelemetry {
    if text.trim().is_empty() || !text.contains("AppleSmartBattery") {
        return BatteryTelemetry {
            has_battery: false,
            power_source: PowerSource::AcPower,
            charge_percent: None,
            health_percent: None,
            cycle_count: None,
            design_cycle_count: None,
            design_capacity_mah: None,
            nominal_max_capacity_mah: None,
            raw_max_capacity_mah: None,
            is_charging: false,
            external_connected: true,
            time_remaining_minutes: None,
            temperature_celsius: None,
            summary: "Desktop Mac running on wall power".to_string(),
        };
    }

    let current_capacity = extract_top_level_u32(text, "CurrentCapacity");
    let max_capacity = extract_top_level_u32(text, "MaxCapacity");
    let cycle_count = extract_top_level_u32(text, "CycleCount");
    let design_cycle_count = extract_top_level_u32(text, "DesignCycleCount9C");
    let is_charging = extract_top_level_bool(text, "IsCharging").unwrap_or(false);
    let external_connected = extract_top_level_bool(text, "ExternalConnected").unwrap_or(false);
    let time_remaining_raw = extract_top_level_u32(text, "TimeRemaining");
    let time_remaining_minutes = time_remaining_raw.filter(|&m| m > 0 && m < 65_535);

    let design_capacity_mah = extract_top_level_u32(text, "DesignCapacity")
        .or_else(|| extract_nested_u32(text, "DesignCapacity"));
    let nominal_max_capacity_mah = extract_top_level_u32(text, "NominalChargeCapacity")
        .or_else(|| extract_nested_u32(text, "NominalChargeCapacity"));
    let raw_max_capacity_mah = extract_top_level_u32(text, "AppleRawMaxCapacity")
        .or_else(|| extract_nested_u32(text, "AppleRawMaxCapacity"));

    let temperature_celsius = extract_top_level_u32(text, "Temperature")
        .or_else(|| extract_nested_u32(text, "Temperature"))
        .and_then(decode_battery_temperature);

    // On Apple Silicon, top-level CurrentCapacity/MaxCapacity are percentages (e.g. 80 / 100),
    // while Intel Macs report mAh for both.
    let charge_percent = match (current_capacity, max_capacity) {
        (Some(cur), Some(max)) if max > 0 && max <= 100 => Some(cur.min(100) as u8),
        (Some(cur), Some(max)) if max > 100 => Some(((cur as f64 / max as f64) * 100.0).round().clamp(0.0, 100.0) as u8),
        (Some(cur), None) if cur <= 100 => Some(cur as u8),
        _ => None,
    };

    // Prefer macOS System Settings formula: NominalChargeCapacity / DesignCapacity.
    let health_numerator = nominal_max_capacity_mah
        .or(raw_max_capacity_mah)
        .or_else(|| max_capacity.filter(|&m| m > 100));
    let health_percent = match (health_numerator, design_capacity_mah) {
        (Some(max_mah), Some(design_mah)) if design_mah > 0 => {
            let pct = ((max_mah as f64 / design_mah as f64) * 100.0)
                .round()
                .clamp(0.0, 100.0) as u8;
            Some(pct)
        }
        _ => None,
    };

    let power_source = if external_connected {
        PowerSource::AcPower
    } else {
        PowerSource::BatteryPower
    };

    let summary = match (health_percent, cycle_count) {
        (Some(h), Some(c)) if h < 80 => {
            format!("Battery capacity is {h}% ({c} cycles) — service recommended")
        }
        (Some(h), Some(c)) => format!("Battery health is {h}% ({c} cycles)"),
        (Some(h), None) => format!("Battery health is {h}%"),
        (None, Some(c)) => format!("Battery has {c} charge cycles"),
        (None, None) => "Battery status available".to_string(),
    };

    BatteryTelemetry {
        has_battery: true,
        power_source,
        charge_percent,
        health_percent,
        cycle_count,
        design_cycle_count,
        design_capacity_mah,
        nominal_max_capacity_mah,
        raw_max_capacity_mah,
        is_charging,
        external_connected,
        time_remaining_minutes,
        temperature_celsius,
        summary,
    }
}

fn decode_battery_temperature(raw: u32) -> Option<f32> {
    // Some AppleSmartBattery entries report centi-Kelvin (~29500..36000),
    // others report centi-Celsius (~1500..6500).
    if (25_000..=38_000).contains(&raw) {
        Some((raw as f32 / 100.0) - 273.15)
    } else if (500..=8_500).contains(&raw) {
        Some(raw as f32 / 100.0)
    } else {
        None
    }
}

fn extract_top_level_u32(text: &str, key: &str) -> Option<u32> {
    let needle = format!("\"{key}\" = ");
    for line in text.lines() {
        if let Some(pos) = line.find(&needle) {
            let val_str = line[pos + needle.len()..].trim();
            if let Ok(v) = val_str.parse::<u32>() {
                return Some(v);
            }
        }
    }
    None
}

fn extract_top_level_bool(text: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{key}\" = ");
    for line in text.lines() {
        if let Some(pos) = line.find(&needle) {
            let val_str = line[pos + needle.len()..].trim();
            if val_str.eq_ignore_ascii_case("Yes") || val_str == "true" || val_str == "1" {
                return Some(true);
            }
            if val_str.eq_ignore_ascii_case("No") || val_str == "false" || val_str == "0" {
                return Some(false);
            }
        }
    }
    None
}

fn extract_nested_u32(text: &str, key: &str) -> Option<u32> {
    let needle = format!("\"{key}\"=");
    for line in text.lines() {
        if let Some(pos) = line.find(&needle) {
            let rest = &line[pos + needle.len()..];
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(v) = digits.parse::<u32>() {
                return Some(v);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_apple_silicon_battery() {
        let sample = r#"
+-o AppleSmartBattery  <class AppleSmartBattery, id 0x100>
  |   "TimeRemaining" = 65535
  |   "ExternalConnected" = Yes
  |   "MaxCapacity" = 100
  |   "CurrentCapacity" = 80
  |   "CycleCount" = 407
  |   "DesignCycleCount9C" = 1000
  |   "IsCharging" = No
  |   "Temperature" = 3015
  |   "BatteryData" = {"CurrentCapacity"=80,"AppleRawMaxCapacity"=4960,"DesignCapacity"=6075,"NominalChargeCapacity"=5110}
"#;
        let batt = parse_ioreg_battery(sample);
        assert!(batt.has_battery);
        assert_eq!(batt.power_source, PowerSource::AcPower);
        assert_eq!(batt.charge_percent, Some(80));
        assert_eq!(batt.health_percent, Some(84));
        assert_eq!(batt.cycle_count, Some(407));
        assert_eq!(batt.design_cycle_count, Some(1000));
        assert_eq!(batt.time_remaining_minutes, None);
        assert!((batt.temperature_celsius.unwrap() - 30.15).abs() < 0.05);
    }

    #[test]
    fn handles_desktop_mac_without_battery() {
        let batt = parse_ioreg_battery("");
        assert!(!batt.has_battery);
        assert_eq!(batt.power_source, PowerSource::AcPower);
        assert!(batt.external_connected);
    }

    #[test]
    fn reads_live_macos_battery() {
        let batt = collect_battery();
        assert!(!batt.summary.is_empty());
        if batt.has_battery {
            assert!(batt.charge_percent.is_some());
        }
    }
}
