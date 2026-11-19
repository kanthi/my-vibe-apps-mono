use chrono::{DateTime, Utc};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Accepts "Z" or "+00:00" offsets, any fractional digits, and a space instead of "T".
pub fn parse_date(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim().replacen(' ', "T", 1);
    DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))
}

pub fn from_epoch(secs: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(secs, 0).unwrap_or(DateTime::UNIX_EPOCH)
}

pub fn from_epoch_millis(ms: i64) -> DateTime<Utc> {
    DateTime::from_timestamp_millis(ms).unwrap_or(DateTime::UNIX_EPOCH)
}

pub fn home_dir() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"))
}

pub fn as_i64(v: &Value) -> Option<i64> {
    v.as_i64().or_else(|| v.as_f64().map(|f| f as i64))
}

/// JSON objects for the lines of a JSONL file that contain any marker. Cheap substring test first,
/// so most lines are never parsed.
pub fn json_lines(path: &Path, markers: &[&str]) -> Vec<Value> {
    let Ok(bytes) = std::fs::read(path) else { return vec![] };
    let text = String::from_utf8_lossy(&bytes);
    text.lines()
        .filter(|l| markers.iter().any(|m| l.contains(m)))
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .collect()
}

/// Every file under `dir` (recursive) for which `keep` is true. Missing folders give an empty list.
pub fn walk_files(dir: &Path, keep: &dyn Fn(&Path) -> bool) -> Vec<PathBuf> {
    let mut out = vec![];
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if keep(&p) {
                out.push(p);
            }
        }
    }
    out
}

pub fn now() -> DateTime<Utc> {
    Utc::now()
}
