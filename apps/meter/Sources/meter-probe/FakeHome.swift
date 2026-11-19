import Foundation
import SQLite3

/// Writes invented tool data into a folder so Meter can be tried without touching real records.
enum FakeHome {
    static func build(at home: URL) throws {
        let fm = FileManager.default
        func write(_ text: String, _ rel: String) throws {
            let url = home.appendingPathComponent(rel)
            try fm.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
            try text.write(to: url, atomically: true, encoding: .utf8)
        }
        func db(_ rel: String, _ sql: String) throws {
            let url = home.appendingPathComponent(rel)
            try fm.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
            var h: OpaquePointer?
            guard sqlite3_open(url.path, &h) == SQLITE_OK, sqlite3_exec(h, sql, nil, nil, nil) == SQLITE_OK else { sqlite3_close(h); throw CocoaError(.fileWriteUnknown) }
            sqlite3_close(h)
        }
        let now = Date()
        let iso = ISO8601DateFormatter(); iso.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        func ago(_ s: TimeInterval) -> String { iso.string(from: now.addingTimeInterval(-s)) }
        let t = Int(now.timeIntervalSince1970)

        // Codex: two threads and a quota that is 62% through a weekly window.
        try db(".codex/state_5.sqlite", """
        CREATE TABLE threads (id TEXT, created_at INTEGER, updated_at INTEGER, source TEXT, cwd TEXT, model TEXT, tokens_used INTEGER);
        INSERT INTO threads VALUES ('c1', \(t - 7200), \(t - 600), 'vscode', '/Users/demo/shop', 'gpt-demo', 1800000);
        INSERT INTO threads VALUES ('c2', \(t - 90000), \(t - 86000), 'vscode', '/Users/demo/api', 'gpt-demo', 900000);
        """)
        let weekStart = now.addingTimeInterval(-3.5 * 86_400)
        try write("""
        {"timestamp":"\(ago(300))","type":"event_msg","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":34.0,"window_minutes":300,"resets_at":\(Int(now.addingTimeInterval(3 * 3600).timeIntervalSince1970))},"secondary":{"used_percent":62.0,"window_minutes":10080,"resets_at":\(Int(weekStart.addingTimeInterval(7 * 86_400).timeIntervalSince1970))}}}}
        """, ".codex/sessions/2026/01/01/rollout-demo.jsonl")

        // Grok: weekly credit percent and a few inferences.
        let start = iso.string(from: now.addingTimeInterval(-2 * 86_400)), end = iso.string(from: now.addingTimeInterval(5 * 86_400))
        try write([
            #"{"ts":"\#(ago(100))","msg":"billing: fetched credits config","ctx":{"config":{"creditUsagePercent":18.0,"currentPeriod":{"type":"USAGE_PERIOD_TYPE_WEEKLY","start":"\#(start)","end":"\#(end)"}}}}"#,
            #"{"ts":"\#(ago(3000))","sid":"g1","msg":"shell.turn.inference_done","ctx":{"prompt_tokens":120000,"cached_prompt_tokens":40000,"completion_tokens":2500,"reasoning_tokens":900}}"#
        ].joined(separator: "\n") + "\n", ".grok/logs/unified.jsonl")
        try write(#"{"type":"turn_started","model_id":"grok-demo"}"# + "\n", ".grok/sessions/%2FUsers%2Fdemo%2Fweb/g1/events.jsonl")

        // Claude Code: no quota, just tokens.
        try write(#"{"type":"assistant","timestamp":"\#(ago(1800))","sessionId":"k1","requestId":"r1","cwd":"/Users/demo/notes","message":{"model":"claude-demo","usage":{"input_tokens":50,"cache_creation_input_tokens":20000,"cache_read_input_tokens":300000,"output_tokens":4000}}}"# + "\n",
                  ".claude/projects/-Users-demo-notes/k1.jsonl")

        // Cursor: turns only.
        try db("Library/Application Support/Cursor/User/globalStorage/state.vscdb", """
        CREATE TABLE cursorDiskKV (key TEXT, value BLOB);
        INSERT INTO cursorDiskKV VALUES ('composerData:d1', '{"createdAt":\(t * 1000 - 3_600_000),"lastUpdatedAt":\(t * 1000 - 600_000),"modelConfig":{"modelName":"grok-demo"},"fullConversationHeadersOnly":[{},{},{},{},{},{}]}');
        """)

        // agy: steps only.
        let ts = { (s: TimeInterval) in iso.string(from: now.addingTimeInterval(-s)).replacingOccurrences(of: "T", with: " ") }
        try db(".gemini/antigravity-cli/conversation_summaries.db", """
        CREATE TABLE conversation_summaries (conversation_id TEXT, step_count INTEGER, last_modified_time TEXT, last_user_input_time TEXT, workspace_uris TEXT, parent_conversation_id TEXT);
        INSERT INTO conversation_summaries VALUES ('a1', 64, '\(ts(900))', '\(ts(2400))', '["file:///Users/demo/shop"]', '');
        """)
    }
}
