//! Writes invented tool data into a folder so Meter can be tried without touching real records.
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use rusqlite::Connection;
use std::path::Path;

fn write(home: &Path, rel: &str, text: &str) -> std::io::Result<()> {
    let p = home.join(rel);
    std::fs::create_dir_all(p.parent().unwrap())?;
    std::fs::write(p, text)
}

fn db(home: &Path, rel: &str, sql: &str) -> Result<(), Box<dyn std::error::Error>> {
    let p = home.join(rel);
    std::fs::create_dir_all(p.parent().unwrap())?;
    // Fresh each time, so the generator can be re-run. Safe because `build` refuses real homes.
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", p.display()));
    }
    Connection::open(p)?.execute_batch(sql)?;
    Ok(())
}

/// True when writing invented data into `target` could overwrite a real user's tool records:
/// the target is this user's real home folder and this is not a virtual machine.
pub fn is_unsafe_target(target: &Path, real_home: &Path, in_vm: bool) -> bool {
    let same = match (std::fs::canonicalize(target), std::fs::canonicalize(real_home)) {
        (Ok(a), Ok(b)) => a == b,
        _ => target == real_home,
    };
    same && !in_vm
}

/// `sysctl kern.hv_vmm_present` is 1 inside a virtual machine.
pub fn running_in_vm() -> bool {
    std::process::Command::new("sysctl")
        .args(["-n", "kern.hv_vmm_present"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "1")
        .unwrap_or(false)
}

pub fn build(home: &Path, now: DateTime<Utc>) -> Result<(), Box<dyn std::error::Error>> {
    if is_unsafe_target(home, &crate::util::home_dir(), running_in_vm()) {
        return Err("refusing to write invented data into your real home folder outside a VM; pass another folder".into());
    }
    let ago = |s: i64| (now - Duration::seconds(s)).to_rfc3339_opts(SecondsFormat::Millis, true);
    let t = now.timestamp();

    // Codex: two threads, and a quota 62% used halfway through a weekly window.
    db(
        home,
        ".codex/state_5.sqlite",
        &format!(
            "CREATE TABLE threads (id TEXT, created_at INTEGER, updated_at INTEGER, source TEXT, cwd TEXT, model TEXT, tokens_used INTEGER);
             INSERT INTO threads VALUES ('c1', {}, {}, 'vscode', '/Users/demo/shop', 'gpt-demo', 1800000);
             INSERT INTO threads VALUES ('c2', {}, {}, 'vscode', '/Users/demo/api', 'gpt-demo', 900000);",
            t - 7200,
            t - 600,
            t - 90000,
            t - 86000
        ),
    )?;
    let week_start = now - Duration::seconds((3.5 * 86_400.0) as i64);
    write(
        home,
        ".codex/sessions/2026/01/01/rollout-demo.jsonl",
        &format!(
            r#"{{"timestamp":"{}","type":"event_msg","payload":{{"type":"token_count","rate_limits":{{"primary":{{"used_percent":34.0,"window_minutes":300,"resets_at":{}}},"secondary":{{"used_percent":62.0,"window_minutes":10080,"resets_at":{}}}}}}}}}
"#,
            ago(300),
            (now + Duration::hours(3)).timestamp(),
            (week_start + Duration::days(7)).timestamp()
        ),
    )?;

    // Grok: weekly credit percent and one inference.
    let start = (now - Duration::days(2)).to_rfc3339_opts(SecondsFormat::Millis, true);
    let end = (now + Duration::days(5)).to_rfc3339_opts(SecondsFormat::Millis, true);
    write(
        home,
        ".grok/logs/unified.jsonl",
        &format!(
            r#"{{"ts":"{}","msg":"billing: fetched credits config","ctx":{{"config":{{"creditUsagePercent":18.0,"currentPeriod":{{"type":"USAGE_PERIOD_TYPE_WEEKLY","start":"{}","end":"{}"}}}}}}}}
{{"ts":"{}","sid":"g1","msg":"shell.turn.inference_done","ctx":{{"prompt_tokens":120000,"cached_prompt_tokens":40000,"completion_tokens":2500,"reasoning_tokens":900}}}}
"#,
            ago(100),
            start,
            end,
            ago(3000)
        ),
    )?;
    write(home, ".grok/sessions/%2FUsers%2Fdemo%2Fweb/g1/events.jsonl", "{\"type\":\"turn_started\",\"model_id\":\"grok-demo\"}\n")?;

    // Claude Code: tokens only.
    write(
        home,
        ".claude/projects/-Users-demo-notes/k1.jsonl",
        &format!(
            r#"{{"type":"assistant","timestamp":"{}","sessionId":"k1","requestId":"r1","cwd":"/Users/demo/notes","message":{{"model":"claude-demo","usage":{{"input_tokens":50,"cache_creation_input_tokens":20000,"cache_read_input_tokens":300000,"output_tokens":4000}}}}}}
"#,
            ago(1800)
        ),
    )?;

    // Cursor: turns only.
    db(
        home,
        "Library/Application Support/Cursor/User/globalStorage/state.vscdb",
        &format!(
            "CREATE TABLE cursorDiskKV (key TEXT, value BLOB);
             INSERT INTO cursorDiskKV VALUES ('composerData:d1', '{{\"createdAt\":{},\"lastUpdatedAt\":{},\"modelConfig\":{{\"modelName\":\"grok-demo\"}},\"fullConversationHeadersOnly\":[{{}},{{}},{{}},{{}},{{}},{{}}]}}');",
            t * 1000 - 3_600_000,
            t * 1000 - 600_000
        ),
    )?;

    // agy: steps only.
    let ts = |s: i64| ago(s).replacen('T', " ", 1);
    db(
        home,
        ".gemini/antigravity-cli/conversation_summaries.db",
        &format!(
            "CREATE TABLE conversation_summaries (conversation_id TEXT, step_count INTEGER, last_modified_time TEXT, last_user_input_time TEXT, workspace_uris TEXT, parent_conversation_id TEXT);
             INSERT INTO conversation_summaries VALUES ('a1', 64, '{}', '{}', '[\"file:///Users/demo/shop\"]', '');",
            ts(900),
            ts(2400)
        ),
    )?;
    Ok(())
}
