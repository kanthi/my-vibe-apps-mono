use std::path::Path;

pub fn format_bytes_si(bytes: u64) -> String {
    const KB: f64 = 1_000.0;
    const MB: f64 = 1_000_000.0;
    const GB: f64 = 1_000_000_000.0;
    const TB: f64 = 1_000_000_000_000.0;
    let b = bytes as f64;
    if b >= TB {
        format!("{:.2} TB", b / TB)
    } else if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

pub fn format_memory_gib(bytes: u64) -> String {
    const MIB: f64 = 1_048_576.0;
    const GIB: f64 = 1_073_741_824.0;
    let b = bytes as f64;
    if b >= GIB {
        format!("{:.2} GB", b / GIB)
    } else if b >= MIB {
        format!("{:.0} MB", b / MIB)
    } else {
        format!("{:.0} KB", b / 1024.0)
    }
}

pub fn format_rate(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1_000_000.0 {
        format!("{:.1} MB/s", bytes_per_sec / 1_000_000.0)
    } else if bytes_per_sec >= 1_000.0 {
        format!("{:.0} KB/s", bytes_per_sec / 1_000.0)
    } else {
        format!("{:.0} B/s", bytes_per_sec.max(0.0))
    }
}

pub fn format_duration_short(seconds: u64) -> String {
    if seconds >= 3600 {
        let h = seconds / 3600;
        let m = (seconds % 3600) / 60;
        format!("{h}h {m}m")
    } else if seconds >= 60 {
        format!("{}m", seconds / 60)
    } else {
        format!("{seconds}s")
    }
}

/// Formats a path using `~` for the user's home folder and middle-truncates overly long strings
/// so paths never wrap across lines inside table rows.
pub fn shorten_home_path(path: &Path, max_chars: usize) -> String {
    let raw = path.to_string_lossy();
    let tilde = if let Ok(home) = std::env::var("HOME")
        && let Some(rest) = raw.strip_prefix(&home)
    {
        format!("~{rest}")
    } else {
        raw.into_owned()
    };

    let chars: Vec<char> = tilde.chars().collect();
    if chars.len() <= max_chars || max_chars < 16 {
        return tilde;
    }
    let head = max_chars / 2 - 2;
    let tail = max_chars - head - 1;
    let prefix: String = chars[..head].iter().collect();
    let suffix: String = chars[chars.len() - tail..].iter().collect();
    format!("{prefix}…{suffix}")
}
