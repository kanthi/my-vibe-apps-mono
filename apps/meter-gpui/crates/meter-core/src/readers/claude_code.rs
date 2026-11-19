use crate::model::*;
use crate::util::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;

/// Reads Claude Code session logs under `projects/`. Only usage numbers, model, cwd, session id
/// and timestamps are taken from each line; message text is never kept. Claude Code writes no
/// quota to disk, so this reader reports none.
pub struct ClaudeCodeReader {
    root: PathBuf,
}

#[derive(Clone)]
struct Call {
    session: String,
    model: Option<String>,
    cwd: Option<String>,
    at: DateTime<Utc>,
    input: i64,
    cache_read: i64,
    output: i64,
}

impl ClaudeCodeReader {
    pub fn new(root: PathBuf) -> Self {
        ClaudeCodeReader { root }
    }
}

impl UsageReader for ClaudeCodeReader {
    fn tool(&self) -> ToolId {
        ToolId::ClaudeCode
    }
    fn is_installed(&self) -> bool {
        self.root.join("projects").exists()
    }

    fn read(&self, since: DateTime<Utc>) -> Result<ReaderOutput, ReadError> {
        let files = walk_files(&self.root.join("projects"), &|p| p.extension().is_some_and(|e| e == "jsonl"));
        // A streamed response can be logged several times with the same requestId; keep the fullest.
        let mut by_request: HashMap<String, Call> = HashMap::new();
        let mut anonymous: Vec<Call> = vec![];
        for file in files {
            for obj in json_lines(&file, &["\"usage\""]) {
                if obj["type"] != "assistant" {
                    continue;
                }
                let msg = &obj["message"];
                let usage = &msg["usage"];
                let (Some(at), Some(session)) = (obj["timestamp"].as_str().and_then(parse_date), obj["sessionId"].as_str())
                else {
                    continue;
                };
                if usage.is_null() || at < since {
                    continue;
                }
                let n = |k: &str| as_i64(&usage[k]).unwrap_or(0);
                let call = Call {
                    session: session.to_string(),
                    model: msg["model"].as_str().map(String::from),
                    cwd: obj["cwd"].as_str().map(String::from),
                    at,
                    input: n("input_tokens") + n("cache_creation_input_tokens"),
                    cache_read: n("cache_read_input_tokens"),
                    output: n("output_tokens"),
                };
                match obj["requestId"].as_str() {
                    Some(rid) => {
                        if by_request.get(rid).is_some_and(|old| old.output > call.output) {
                            continue;
                        }
                        by_request.insert(rid.to_string(), call);
                    }
                    None => anonymous.push(call),
                }
            }
        }
        let mut by_session: HashMap<String, Vec<Call>> = HashMap::new();
        for c in by_request.into_values().chain(anonymous) {
            by_session.entry(c.session.clone()).or_default().push(c);
        }
        let mut sessions: Vec<Session> = by_session
            .into_iter()
            .map(|(id, mut calls)| {
                calls.sort_by_key(|c| c.at);
                let (first, last) = (calls.first().unwrap().clone(), calls.last().unwrap().clone());
                // Subagent work is counted once, under the parent session that spawned it.
                let mut s = Session::new(ToolId::ClaudeCode, id, first.at, last.at);
                s.model = last.model;
                s.project = last.cwd;
                s.input_tokens = Some(calls.iter().map(|c| c.input).sum());
                s.cached_tokens = Some(calls.iter().map(|c| c.cache_read).sum());
                s.output_tokens = Some(calls.iter().map(|c| c.output).sum());
                s.turns = Some(calls.len() as i64);
                s
            })
            .collect();
        sessions.sort_by_key(|s| s.start);
        Ok(ReaderOutput { sessions, quotas: vec![] })
    }
}
