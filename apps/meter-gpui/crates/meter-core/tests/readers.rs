mod common;
use common::*;
use meter_core::readers::*;
use meter_core::{ReaderOutput, ToolId, UsageReader};
use tempfile::tempdir;

#[test]
fn codex_reads_threads_and_flags_subagents_and_never_touches_title() {
    let root = tempdir().unwrap();
    sqlite(root.path(), "state_5.sqlite", "
        CREATE TABLE threads (id TEXT, created_at INTEGER, updated_at INTEGER, source TEXT, cwd TEXT, model TEXT, tokens_used INTEGER, title TEXT);
        INSERT INTO threads VALUES ('a', 1000, 2000, 'vscode', '/p/one', 'gpt-x', 500, 'SECRET TITLE');
        INSERT INTO threads VALUES ('b', 1000, 2100, '{\"subagent\":{\"other\":\"guardian\"}}', '/p/one', 'auto-review', 40, 'SECRET TITLE');
        INSERT INTO threads VALUES ('old', 10, 20, 'vscode', '/p/old', 'gpt-x', 9, 'x');");
    let out = CodexReader::new(root.path().into()).read(meter_core::util::from_epoch(1500)).unwrap();
    let mut ids: Vec<_> = out.sessions.iter().map(|s| s.id.clone()).collect();
    ids.sort();
    assert_eq!(ids, ["a", "b"]);
    assert!(out.sessions.iter().find(|s| s.id == "b").unwrap().is_subagent);
    let a = out.sessions.iter().find(|s| s.id == "a").unwrap();
    assert_eq!(a.input_tokens, Some(500));
    assert_eq!(a.project.as_deref(), Some("/p/one"));
}

#[test]
fn codex_picks_newest_quota_and_labels_windows() {
    let root = tempdir().unwrap();
    let older = r#"{"timestamp":"2026-10-03T10:00:00.000Z","type":"event_msg","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":1.0,"window_minutes":300,"resets_at":1800000000}}}}"#;
    let newer = r#"{"timestamp":"2026-10-03T11:00:00.000Z","type":"event_msg","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":2.0,"window_minutes":300,"resets_at":1800000000},"secondary":{"used_percent":40.5,"window_minutes":10080,"resets_at":1800500000}}}}"#;
    write(root.path(), "sessions/2026/10/03/rollout-x.jsonl", &format!("{newer}\n{older}\nnot json\n"));
    let q = CodexReader::new(root.path().into()).read(chrono::DateTime::UNIX_EPOCH).unwrap().quotas;
    assert_eq!(q.iter().map(|q| q.label.as_str()).collect::<Vec<_>>(), ["Session", "Weekly"]);
    assert_eq!(q.iter().map(|q| q.used_percent).collect::<Vec<_>>(), [2.0, 40.5]);
    assert_eq!(q[1].resets_at, Some(meter_core::util::from_epoch(1_800_500_000)));
}

#[test]
fn missing_folders_are_not_installed_and_empty() {
    let root = tempdir().unwrap();
    let none = root.path().join("nope");
    for tool in ToolId::ALL {
        let r = meter_core::readers::make(tool, Some(none.clone()));
        assert!(!r.is_installed(), "{tool:?}");
        assert_eq!(r.read(chrono::DateTime::UNIX_EPOCH).unwrap_or_default(), ReaderOutput::default());
    }
}

fn grok_log(root: &std::path::Path) {
    let lines = [
        r#"{"ts":"2026-10-23T10:00:00.000Z","sid":"s1","msg":"shell.turn.inference_done","ctx":{"prompt_tokens":1000,"cached_prompt_tokens":200,"completion_tokens":50,"reasoning_tokens":30}}"#,
        r#"{"ts":"2026-10-23T10:05:00.000Z","sid":"s1","msg":"shell.turn.inference_done","ctx":{"prompt_tokens":500,"cached_prompt_tokens":100,"completion_tokens":20,"reasoning_tokens":0}}"#,
        r#"{"ts":"2026-10-23T11:00:00.000Z","msg":"billing: fetched credits config","ctx":{"config":{"creditUsagePercent":10.0,"currentPeriod":{"type":"USAGE_PERIOD_TYPE_WEEKLY","start":"2026-10-21T01:00:00+00:00","end":"2026-10-28T01:00:00+00:00"}}}}"#,
        r#"{"ts":"2026-10-24T11:00:00.000Z","msg":"billing: fetched credits config","ctx":{"config":{"creditUsagePercent":20.0,"currentPeriod":{"type":"USAGE_PERIOD_TYPE_WEEKLY","start":"2026-10-21T01:00:00+00:00","end":"2026-10-28T01:00:00+00:00"}}}}"#,
        r#"{"ts":"2026-10-24T11:00:00.000Z","msg":"startup timing","ctx":{}}"#,
    ];
    write(root, "logs/unified.jsonl", &(lines.join("\n") + "\n"));
}

#[test]
fn grok_latest_weekly_quota() {
    let root = tempdir().unwrap();
    grok_log(root.path());
    let q = GrokReader::new(root.path().into()).read(chrono::DateTime::UNIX_EPOCH).unwrap().quotas;
    assert_eq!(q.len(), 1);
    assert_eq!((q[0].label.as_str(), q[0].used_percent, q[0].window_minutes), ("Weekly", 20.0, Some(7 * 24 * 60)));
    assert_eq!(q[0].resets_at, Some(d("2026-10-28T01:00:00Z")));
}

#[test]
fn grok_sums_tokens_per_session_and_resolves_project_and_model() {
    let root = tempdir().unwrap();
    grok_log(root.path());
    write(root.path(), "sessions/%2FUsers%2Fme%2Fapp/s1/events.jsonl", "{\"ts\":\"x\",\"type\":\"turn_started\",\"model_id\":\"grok-x\"}\n");
    let s = GrokReader::new(root.path().into()).read(chrono::DateTime::UNIX_EPOCH).unwrap().sessions;
    assert_eq!(s.len(), 1);
    assert_eq!((s[0].input_tokens, s[0].cached_tokens, s[0].output_tokens), (Some(1500), Some(300), Some(100)));
    assert_eq!(s[0].project.as_deref(), Some("/Users/me/app"));
    assert_eq!(s[0].model.as_deref(), Some("grok-x"));
}

#[test]
fn grok_since_filters_old_inferences() {
    let root = tempdir().unwrap();
    grok_log(root.path());
    assert!(GrokReader::new(root.path().into()).read(d("2026-10-24T00:00:00Z")).unwrap().sessions.is_empty());
}

fn claude_line(rid: &str, out: i64, ts: &str, session: &str, sidechain: bool) -> String {
    format!(r#"{{"type":"assistant","timestamp":"{ts}","sessionId":"{session}","requestId":"{rid}","cwd":"/p/app","isSidechain":{sidechain},"message":{{"model":"claude-x","content":"SECRET TEXT","usage":{{"input_tokens":10,"cache_creation_input_tokens":100,"cache_read_input_tokens":1000,"output_tokens":{out}}}}}}}"#)
}

#[test]
fn claude_dedupes_streamed_requests_and_sums_tokens() {
    let root = tempdir().unwrap();
    let body = [
        claude_line("r1", 5, "2026-10-24T10:00:00.000Z", "s1", false),
        claude_line("r1", 50, "2026-10-24T10:00:01.000Z", "s1", false), // fuller duplicate wins
        claude_line("r2", 7, "2026-10-24T10:05:00.000Z", "s1", false),
        r#"{"type":"user","timestamp":"2026-10-24T10:06:00.000Z","message":{"content":"hi"}}"#.to_string(),
    ]
    .join("\n");
    write(root.path(), "projects/-p-app/s1.jsonl", &body);
    let out = ClaudeCodeReader::new(root.path().into()).read(chrono::DateTime::UNIX_EPOCH).unwrap();
    assert_eq!(out.sessions.len(), 1);
    let s = &out.sessions[0];
    assert_eq!((s.turns, s.output_tokens, s.input_tokens, s.cached_tokens), (Some(2), Some(57), Some(220), Some(2000)));
    assert_eq!((s.model.as_deref(), s.project.as_deref()), (Some("claude-x"), Some("/p/app")));
    assert!(out.quotas.is_empty());
}

#[test]
fn claude_subagent_work_counts_under_parent_session() {
    let root = tempdir().unwrap();
    write(root.path(), "projects/-p/s1.jsonl", &claude_line("r1", 10, "2026-10-24T10:00:00.000Z", "s1", false));
    write(root.path(), "projects/-p/s1/subagents/agent-1.jsonl", &claude_line("r9", 20, "2026-10-24T10:01:00.000Z", "s1", true));
    let out = ClaudeCodeReader::new(root.path().into()).read(chrono::DateTime::UNIX_EPOCH).unwrap();
    assert_eq!(out.sessions.len(), 1);
    assert_eq!(out.sessions[0].output_tokens, Some(30));
}

#[test]
fn claude_since_filters() {
    let root = tempdir().unwrap();
    write(root.path(), "projects/-p/s1.jsonl", &claude_line("r1", 10, "2026-10-01T10:00:00.000Z", "s1", false));
    assert!(ClaudeCodeReader::new(root.path().into()).read(d("2026-10-10T00:00:00Z")).unwrap().sessions.is_empty());
}

#[test]
fn cursor_reads_turns_and_model_and_skips_drafts() {
    let root = tempdir().unwrap();
    let used = r#"{"createdAt":1791092136848,"lastUpdatedAt":1791092166622,"text":"SECRET","modelConfig":{"modelName":"grok-4.6"},"fullConversationHeadersOnly":[{},{},{}]}"#;
    let draft = r#"{"createdAt":1791092136848,"modelConfig":{"modelName":"grok-4.6"},"fullConversationHeadersOnly":[]}"#;
    let no_date = r#"{"fullConversationHeadersOnly":[{}]}"#;
    sqlite(root.path(), "User/globalStorage/state.vscdb", &format!("
        CREATE TABLE cursorDiskKV (key TEXT, value BLOB);
        INSERT INTO cursorDiskKV VALUES ('composerData:abc', '{used}');
        INSERT INTO cursorDiskKV VALUES ('composerData:empty-state-draft', '{draft}');
        INSERT INTO cursorDiskKV VALUES ('composerData:nodate', '{no_date}');
        INSERT INTO cursorDiskKV VALUES ('bubbleId:abc:1', '{{\"text\":\"SECRET\"}}');"));
    let out = CursorReader::new(root.path().into()).read(chrono::DateTime::UNIX_EPOCH).unwrap();
    assert_eq!(out.sessions.len(), 1);
    let s = &out.sessions[0];
    assert_eq!((s.id.as_str(), s.model.as_deref(), s.turns), ("abc", Some("grok-4.6"), Some(3)));
    assert!(s.input_tokens.is_none() && s.output_tokens.is_none());
    assert_eq!(s.end, meter_core::util::from_epoch_millis(1_791_092_166_622));
}

fn agy_db(root: &std::path::Path) {
    sqlite(root, "conversation_summaries.db", "
        CREATE TABLE conversation_summaries (conversation_id TEXT, title TEXT, preview TEXT, step_count INTEGER,
          last_modified_time TEXT, workspace_uris TEXT, parent_conversation_id TEXT, last_user_input_time TEXT);
        INSERT INTO conversation_summaries VALUES ('p','SECRET','SECRET',80,'2026-10-20 16:31:28.552742+00:00','[\"file:///Users/me/My%20Proj\"]','','2026-10-20 16:30:36.79965+00:00');
        INSERT INTO conversation_summaries VALUES ('c','SECRET','SECRET',20,'2026-10-20 16:40:00.000000+00:00','[\"file:///Users/me/My%20Proj\"]','p','0001-01-01 00:00:00+00:00');
        INSERT INTO conversation_summaries VALUES ('old','x','x',5,'2026-09-01 00:00:00+00:00','[]','','0001-01-01 00:00:00+00:00');");
}

#[test]
fn agy_folds_subagent_into_parent_and_decodes_project() {
    let root = tempdir().unwrap();
    agy_db(root.path());
    let out = AgyReader::new(root.path().into()).read(d("2026-10-01T00:00:00Z")).unwrap();
    assert_eq!(out.sessions.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), ["p"]);
    let p = &out.sessions[0];
    assert_eq!((p.turns, p.project.as_deref()), (Some(100), Some("/Users/me/My Proj")));
    assert_eq!(p.end, d("2026-10-20T16:40:00Z"));
    assert!(p.input_tokens.is_none() && out.quotas.is_empty());
}

#[test]
fn agy_since_drops_old_conversations_only() {
    let root = tempdir().unwrap();
    agy_db(root.path());
    assert_eq!(AgyReader::new(root.path().into()).read(chrono::DateTime::UNIX_EPOCH).unwrap().sessions.len(), 2);
}

#[test]
fn readers_work_on_a_copy_and_leave_the_original_untouched() {
    let root = tempdir().unwrap();
    agy_db(root.path());
    let db = root.path().join("conversation_summaries.db");
    let before = std::fs::read(&db).unwrap();
    AgyReader::new(root.path().into()).read(chrono::DateTime::UNIX_EPOCH).unwrap();
    assert_eq!(std::fs::read(&db).unwrap(), before);
    let names: Vec<_> = std::fs::read_dir(root.path()).unwrap().flatten().map(|e| e.file_name()).collect();
    assert_eq!(names.len(), 1, "no -wal/-shm/journal files left next to the original: {names:?}");
}

#[test]
fn fake_home_can_be_rebuilt_in_the_same_folder() {
    let root = tempdir().unwrap();
    meter_core::fakehome::build(root.path(), chrono::Utc::now()).unwrap();
    meter_core::fakehome::build(root.path(), chrono::Utc::now()).unwrap(); // used to fail: table already exists
    let out = CodexReader::new(root.path().join(".codex")).read(chrono::DateTime::UNIX_EPOCH).unwrap();
    assert_eq!(out.sessions.len(), 2);
}

#[test]
fn fake_home_refuses_the_real_home_outside_a_vm_but_allows_it_inside_one() {
    use meter_core::fakehome::is_unsafe_target;
    let home = tempdir().unwrap();
    let other = tempdir().unwrap();
    assert!(is_unsafe_target(home.path(), home.path(), false), "real home on a real Mac");
    assert!(!is_unsafe_target(home.path(), home.path(), true), "real home inside a VM is the point");
    assert!(!is_unsafe_target(other.path(), home.path(), false), "any other folder is fine");
}
