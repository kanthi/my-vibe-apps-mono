use chrono::{DateTime, Utc};

/// 1_234 -> "1.2k", 94_351 -> "94k", 8_616_973 -> "8.6M"
pub fn tokens(n: i64) -> String {
    match n {
        i64::MIN..=999 => n.to_string(),
        1_000..=9_999 => format!("{:.1}k", n as f64 / 1_000.0),
        10_000..=999_999 => format!("{:.0}k", n as f64 / 1_000.0),
        1_000_000..=9_999_999 => format!("{:.1}M", n as f64 / 1_000_000.0),
        _ => format!("{:.0}M", n as f64 / 1_000_000.0),
    }
}

/// "in 3 h", "in 2 d", "in 40 min", or "now"
pub fn until(date: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let s = (date - now).num_seconds() as f64;
    if s <= 60.0 {
        "now".into()
    } else if s < 3600.0 {
        format!("in {} min", (s / 60.0) as i64)
    } else if s < 86_400.0 {
        format!("in {} h", (s / 3600.0).round() as i64)
    } else {
        format!("in {} d", (s / 86_400.0).round() as i64)
    }
}

pub fn base_name(path: &str) -> String {
    path.rsplit('/').find(|s| !s.is_empty()).unwrap_or(path).to_string()
}

pub fn percent(v: f64) -> String {
    format!("{:.0}%", v)
}
