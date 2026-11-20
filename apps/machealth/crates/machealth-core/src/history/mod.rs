use crate::telemetry::SystemSnapshot;
use crate::telemetry::thermal::ThermalLevel;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

const MAX_RETENTION_DAYS: i64 = 30;
const TARGET_CHART_POINTS: usize = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryRange {
    Hour,
    Day,
    Week,
    Month,
}

impl HistoryRange {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Hour => "Hour",
            Self::Day => "Day",
            Self::Week => "Week",
            Self::Month => "Month",
        }
    }

    pub fn duration(&self) -> Duration {
        match self {
            Self::Hour => Duration::hours(1),
            Self::Day => Duration::days(1),
            Self::Week => Duration::days(7),
            Self::Month => Duration::days(30),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistorySample {
    pub timestamp: DateTime<Utc>,
    pub cpu_percent: f32,
    pub memory_used_percent: f32,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub thermal_level: ThermalLevel,
    pub health_score: u8,
    pub busiest_app: Option<String>,
}

impl HistorySample {
    pub fn from_snapshot(
        snapshot: &SystemSnapshot,
        health_score: u8,
        busiest_app: Option<String>,
    ) -> Self {
        Self {
            timestamp: snapshot.captured_at,
            cpu_percent: snapshot.cpu.total_percent,
            memory_used_percent: snapshot.memory.used_percent,
            memory_used_bytes: snapshot.memory.used_bytes,
            memory_total_bytes: snapshot.memory.total_bytes,
            thermal_level: snapshot.thermal.level,
            health_score,
            busiest_app,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryPoint {
    pub timestamp: DateTime<Utc>,
    pub cpu_percent: f32,
    pub memory_used_percent: f32,
    pub health_score: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistorySeries {
    pub range: HistoryRange,
    pub points: Vec<HistoryPoint>,
    pub sample_count: usize,
    pub avg_cpu_percent: f32,
    pub peak_cpu_percent: f32,
    pub avg_memory_percent: f32,
    pub peak_memory_percent: f32,
    pub busiest_apps: Vec<String>,
    pub summary: String,
}

/// Local-first JSONL telemetry history store under `~/Library/Application Support/MacHealth/history.jsonl`.
#[derive(Debug, Clone)]
pub struct HistoryStore {
    path: PathBuf,
}

impl HistoryStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default_store() -> Option<Self> {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        Some(Self {
            path: home.join("Library/Application Support/MacHealth/history.jsonl"),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(&self, sample: &HistorySample) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create MacHealth history directory: {e}"))?;
        }
        let line = serde_json::to_string(sample)
            .map_err(|e| format!("failed to serialize history sample: {e}"))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| format!("failed to open history store: {e}"))?;
        writeln!(file, "{line}").map_err(|e| format!("failed to write history sample: {e}"))?;
        Ok(())
    }

    pub fn load_all(&self) -> Vec<HistorySample> {
        let Ok(file) = fs::File::open(&self.path) else {
            return Vec::new();
        };
        let reader = BufReader::new(file);
        reader
            .lines()
            .map_while(Result::ok)
            .filter_map(|line| serde_json::from_str::<HistorySample>(&line).ok())
            .collect()
    }

    /// Compacts the JSONL file to keep only samples within the 30-day retention window.
    pub fn prune_older_than_retention(&self, now: DateTime<Utc>) -> Result<usize, String> {
        let cutoff = now - Duration::days(MAX_RETENTION_DAYS);
        let samples = self.load_all();
        let before = samples.len();
        let kept: Vec<HistorySample> = samples
            .into_iter()
            .filter(|s| s.timestamp >= cutoff)
            .collect();
        let removed = before.saturating_sub(kept.len());

        if removed == 0 {
            return Ok(0);
        }

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create history directory: {e}"))?;
        }
        let tmp = self.path.with_extension("jsonl.tmp");
        {
            let mut f = fs::File::create(&tmp)
                .map_err(|e| format!("failed to create temp history file: {e}"))?;
            for s in &kept {
                if let Ok(line) = serde_json::to_string(s) {
                    let _ = writeln!(f, "{line}");
                }
            }
        }
        fs::rename(&tmp, &self.path)
            .map_err(|e| format!("failed to compact history store: {e}"))?;
        Ok(removed)
    }

    pub fn query(&self, range: HistoryRange, now: DateTime<Utc>) -> HistorySeries {
        let samples = self.load_all();
        summarize_history(&samples, range, now)
    }
}

pub fn summarize_history(
    samples: &[HistorySample],
    range: HistoryRange,
    now: DateTime<Utc>,
) -> HistorySeries {
    let cutoff = now - range.duration();
    let mut filtered: Vec<&HistorySample> = samples
        .iter()
        .filter(|s| s.timestamp >= cutoff && s.timestamp <= now + Duration::seconds(5))
        .collect();
    filtered.sort_by_key(|s| s.timestamp);

    if filtered.is_empty() {
        return HistorySeries {
            range,
            points: Vec::new(),
            sample_count: 0,
            avg_cpu_percent: 0.0,
            peak_cpu_percent: 0.0,
            avg_memory_percent: 0.0,
            peak_memory_percent: 0.0,
            busiest_apps: Vec::new(),
            summary: format!(
                "No activity history recorded for the last {} yet.",
                range.label().to_ascii_lowercase()
            ),
        };
    }

    let sample_count = filtered.len();
    let avg_cpu_percent =
        filtered.iter().map(|s| s.cpu_percent).sum::<f32>() / sample_count as f32;
    let peak_cpu_percent = filtered
        .iter()
        .map(|s| s.cpu_percent)
        .fold(0.0f32, f32::max);
    let avg_memory_percent =
        filtered.iter().map(|s| s.memory_used_percent).sum::<f32>() / sample_count as f32;
    let peak_memory_percent = filtered
        .iter()
        .map(|s| s.memory_used_percent)
        .fold(0.0f32, f32::max);

    let mut app_freq: HashMap<&str, usize> = HashMap::new();
    for s in &filtered {
        if let Some(ref app) = s.busiest_app
            && !app.is_empty()
        {
            *app_freq.entry(app.as_str()).or_insert(0) += 1;
        }
    }
    let mut busiest_pairs: Vec<(&str, usize)> = app_freq.into_iter().collect();
    busiest_pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    let busiest_apps: Vec<String> = busiest_pairs
        .into_iter()
        .take(3)
        .map(|(name, _)| name.to_string())
        .collect();

    let points = downsample_points(&filtered, TARGET_CHART_POINTS);

    let busiest_clause = if busiest_apps.is_empty() {
        String::new()
    } else {
        format!(" Busiest: {}.", busiest_apps.join(", "))
    };
    let summary = format!(
        "Processor {:.0}% on average, briefly up to {:.0}%. Memory averaged {:.0}%.{}",
        avg_cpu_percent, peak_cpu_percent, avg_memory_percent, busiest_clause
    );

    HistorySeries {
        range,
        points,
        sample_count,
        avg_cpu_percent,
        peak_cpu_percent,
        avg_memory_percent,
        peak_memory_percent,
        busiest_apps,
        summary,
    }
}

fn downsample_points(samples: &[&HistorySample], max_points: usize) -> Vec<HistoryPoint> {
    if samples.len() <= max_points {
        return samples
            .iter()
            .map(|s| HistoryPoint {
                timestamp: s.timestamp,
                cpu_percent: s.cpu_percent,
                memory_used_percent: s.memory_used_percent,
                health_score: s.health_score,
            })
            .collect();
    }

    let chunk_size = samples.len().div_ceil(max_points);
    samples
        .chunks(chunk_size)
        .map(|chunk| {
            let n = chunk.len() as f32;
            let mid = chunk[chunk.len() / 2].timestamp;
            let cpu = chunk.iter().map(|s| s.cpu_percent).sum::<f32>() / n;
            let mem = chunk.iter().map(|s| s.memory_used_percent).sum::<f32>() / n;
            let score = (chunk.iter().map(|s| s.health_score as f32).sum::<f32>() / n)
                .round()
                .clamp(0.0, 100.0) as u8;
            HistoryPoint {
                timestamp: mid,
                cpu_percent: cpu,
                memory_used_percent: mem,
                health_score: score,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_queries_downsamples_and_prunes_history() {
        let path = std::env::temp_dir().join(format!(
            "machealth_history_test_{}.jsonl",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        let store = HistoryStore::new(path.clone());
        let now = Utc::now();

        // One ancient sample (>30d old) that should be pruned.
        store
            .append(&HistorySample {
                timestamp: now - Duration::days(45),
                cpu_percent: 5.0,
                memory_used_percent: 40.0,
                memory_used_bytes: 4_000_000_000,
                memory_total_bytes: 16_000_000_000,
                thermal_level: ThermalLevel::Normal,
                health_score: 95,
                busiest_app: Some("Finder".into()),
            })
            .unwrap();

        // 80 samples within the last hour (tests downsampling to <= 60 points).
        for i in 0..80 {
            let cpu = if i == 40 { 96.0 } else { 10.0 };
            let app = if i % 3 == 0 { "Xcode" } else { "Chrome" };
            store
                .append(&HistorySample {
                    timestamp: now - Duration::seconds((80 - i) * 30),
                    cpu_percent: cpu,
                    memory_used_percent: 62.0,
                    memory_used_bytes: 10_000_000_000,
                    memory_total_bytes: 16_000_000_000,
                    thermal_level: ThermalLevel::Normal,
                    health_score: 90,
                    busiest_app: Some(app.into()),
                })
                .unwrap();
        }

        let series = store.query(HistoryRange::Hour, now);
        assert_eq!(series.sample_count, 80);
        assert!(series.points.len() <= TARGET_CHART_POINTS);
        assert!((series.peak_cpu_percent - 96.0).abs() < 0.01);
        assert!(series.summary.contains("briefly up to 96%"));
        assert!(series.summary.contains("Busiest: Chrome, Xcode"));

        let removed = store.prune_older_than_retention(now).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(store.load_all().len(), 80);

        let _ = fs::remove_file(&path);
    }
}
