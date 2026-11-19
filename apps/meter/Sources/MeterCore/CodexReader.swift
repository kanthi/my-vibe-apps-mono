import Foundation
import SQLite3

/// Reads Codex's local records. Selects only non-text columns from the thread table,
/// never `title` or `first_user_message`, and never opens credentials.
public struct CodexReader: UsageReader {
    public let tool = ToolID.codex
    let root: URL

    public init(root: URL = FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent(".codex")) {
        self.root = root
    }

    public var isInstalled: Bool { FileManager.default.fileExists(atPath: root.path) }

    public func read(since: Date) throws -> ReaderOutput {
        ReaderOutput(sessions: try sessions(since: since), quotas: latestQuotas())
    }

    func sessions(since: Date) throws -> [Session] {
        let dbURL = root.appendingPathComponent("state_5.sqlite")
        guard FileManager.default.fileExists(atPath: dbURL.path) else { return [] }
        let snapshot = try SQLiteSnapshot(copying: dbURL)
        defer { snapshot.close() }
        let db = snapshot.db
        let sql = "SELECT id, created_at, updated_at, source, cwd, model, tokens_used FROM threads WHERE updated_at >= ?"
        var stmt: OpaquePointer?
        guard sqlite3_prepare_v2(db, sql, -1, &stmt, nil) == SQLITE_OK else { throw CocoaError(.fileReadUnknown) }
        defer { sqlite3_finalize(stmt) }
        sqlite3_bind_int64(stmt, 1, Int64(since.timeIntervalSince1970))
        func text(_ i: Int32) -> String? { sqlite3_column_text(stmt, i).map { String(cString: $0) } }
        var out: [Session] = []
        while sqlite3_step(stmt) == SQLITE_ROW {
            let tokens = sqlite3_column_type(stmt, 6) == SQLITE_NULL ? nil : Int(sqlite3_column_int64(stmt, 6))
            out.append(Session(
                tool: .codex, id: text(0) ?? "",
                model: text(5), project: text(4),
                start: Date(timeIntervalSince1970: TimeInterval(sqlite3_column_int64(stmt, 1))),
                end: Date(timeIntervalSince1970: TimeInterval(sqlite3_column_int64(stmt, 2))),
                // `tokens_used` is one total; Codex does not split it here.
                inputTokens: tokens,
                isSubagent: (text(3) ?? "").contains("\"subagent\"")))
        }
        return out
    }

    /// The newest `rate_limits` seen in the newest rollout files.
    func latestQuotas() -> [Quota] {
        let base = root.appendingPathComponent("sessions")
        guard let walker = FileManager.default.enumerator(at: base, includingPropertiesForKeys: nil) else { return [] }
        let files = walker.compactMap { $0 as? URL }
            .filter { $0.lastPathComponent.hasPrefix("rollout-") && $0.pathExtension == "jsonl" }
            .sorted { $0.path > $1.path }
        for file in files.prefix(5) {
            var best: (Date, [String: Any])?
            for obj in Parse.jsonLines(in: file, containing: ["\"rate_limits\""]) {
                guard let payload = obj["payload"] as? [String: Any], payload["type"] as? String == "token_count",
                      let limits = payload["rate_limits"] as? [String: Any],
                      let ts = (obj["timestamp"] as? String).flatMap(Parse.date) else { continue }
                if best == nil || ts >= best!.0 { best = (ts, limits) }
            }
            guard let (observed, limits) = best else { continue }
            return ["primary", "secondary"].compactMap { key in
                guard let w = limits[key] as? [String: Any], let used = (w["used_percent"] as? NSNumber)?.doubleValue else { return nil }
                let minutes = Parse.int(w["window_minutes"])
                let reset = (w["resets_at"] as? NSNumber).map { Date(timeIntervalSince1970: $0.doubleValue) }
                return Quota(tool: .codex, label: Parse.windowLabel(minutes: minutes), usedPercent: used,
                             windowMinutes: minutes, resetsAt: reset, observedAt: observed)
            }
        }
        return []
    }
}
