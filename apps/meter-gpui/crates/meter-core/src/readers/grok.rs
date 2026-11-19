use crate::model::*;
use crate::readers::window_label;
use crate::util::*;
use chrono::{DateTime, Utc};
use percent_encoding::percent_decode_str;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Reads Grok's local log: weekly credit percent and per-inference token counts.
/// Never reads `auth.json` or chat history.
pub struct GrokReader {
    root: PathBuf,
}

impl GrokReader {
    pub fn new(root: PathBuf) -> Self {
        GrokReader { root }
    }

    fn quota(lines: &[Value]) -> Vec<Quota> {
        let mut latest: Option<(DateTime<Utc>, &Value)> = None;
        for l in lines.iter().filter(|l| l["msg"] == "billing: fetched credits config") {
            let Some(ts) = l["ts"].as_str().and_then(parse_date) else { continue };
            let cfg = &l["ctx"]["config"];
            if cfg.is_null() {
                continue;
            }
            if latest.map_or(true, |(t, _)| ts >= t) {
                latest = Some((ts, cfg));
            }
        }
        let Some((observed, cfg)) = latest else { return vec![] };
        let Some(used) = cfg["creditUsagePercent"].as_f64() else { return vec![] };
        let period = &cfg["currentPeriod"];
        let start = period["start"].as_str().and_then(parse_date);
        let end = period["end"].as_str().and_then(parse_date);
        let minutes = start.zip(end).map(|(s, e)| (e - s).num_minutes());
        let label = if period["type"].as_str().is_some_and(|t| t.contains("WEEKLY")) {
            "Weekly".to_string()
        } else {
            window_label(minutes)
        };
        let mut q = Quota::new(ToolId::Grok, label, used, observed);
        q.window_minutes = minutes;
        q.resets_at = end;
        vec![q]
    }

    fn sessions(&self, lines: &[Value], since: DateTime<Utc>) -> Vec<Session> {
        struct Acc {
            start: DateTime<Utc>,
            end: DateTime<Utc>,
            input: i64,
            cached: i64,
            output: i64,
        }
        let mut by_sid: HashMap<String, Acc> = HashMap::new();
        for l in lines.iter().filter(|l| l["msg"] == "shell.turn.inference_done") {
            let (Some(sid), Some(ts)) = (l["sid"].as_str(), l["ts"].as_str().and_then(parse_date)) else { continue };
            if ts < since {
                continue;
            }
            let ctx = &l["ctx"];
            let a = by_sid.entry(sid.to_string()).or_insert(Acc { start: ts, end: ts, input: 0, cached: 0, output: 0 });
            a.start = a.start.min(ts);
            a.end = a.end.max(ts);
            a.input += as_i64(&ctx["prompt_tokens"]).unwrap_or(0);
            a.cached += as_i64(&ctx["cached_prompt_tokens"]).unwrap_or(0);
            // Assumption: reasoning tokens are billed on top of completion tokens.
            a.output += as_i64(&ctx["completion_tokens"]).unwrap_or(0) + as_i64(&ctx["reasoning_tokens"]).unwrap_or(0);
        }
        let dirs = self.session_directories();
        let mut out: Vec<Session> = by_sid
            .into_iter()
            .map(|(sid, a)| {
                let dir = dirs.get(&sid);
                let mut s = Session::new(ToolId::Grok, sid.clone(), a.start, a.end);
                s.model = dir.and_then(|d| Self::model_in(d));
                s.project = dir.map(|d| Self::project_of(d));
                s.input_tokens = Some(a.input);
                s.cached_tokens = Some(a.cached);
                s.output_tokens = Some(a.output);
                s
            })
            .collect();
        out.sort_by_key(|s| s.start);
        out
    }

    /// session id -> its folder under sessions/<url-encoded cwd>/<id>
    fn session_directories(&self) -> HashMap<String, PathBuf> {
        let mut map = HashMap::new();
        let Ok(projects) = std::fs::read_dir(self.root.join("sessions")) else { return map };
        for proj in projects.flatten() {
            let Ok(sessions) = std::fs::read_dir(proj.path()) else { continue };
            for s in sessions.flatten() {
                map.insert(s.file_name().to_string_lossy().to_string(), s.path());
            }
        }
        map
    }

    fn project_of(dir: &Path) -> String {
        let name = dir.parent().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        percent_decode_str(&name).decode_utf8_lossy().to_string()
    }

    fn model_in(dir: &Path) -> Option<String> {
        json_lines(&dir.join("events.jsonl"), &["turn_started"])
            .into_iter()
            .find(|e| e["type"] == "turn_started")
            .and_then(|e| match &e["model_id"] {
                Value::String(s) => Some(s.clone()),
                Value::Null => None,
                v => Some(v.to_string()),
            })
    }
}

impl UsageReader for GrokReader {
    fn tool(&self) -> ToolId {
        ToolId::Grok
    }
    fn is_installed(&self) -> bool {
        self.root.exists()
    }
    fn read(&self, since: DateTime<Utc>) -> Result<ReaderOutput, ReadError> {
        let lines = json_lines(
            &self.root.join("logs/unified.jsonl"),
            &["billing: fetched credits config", "shell.turn.inference_done"],
        );
        Ok(ReaderOutput { sessions: self.sessions(&lines, since), quotas: Self::quota(&lines) })
    }
}
