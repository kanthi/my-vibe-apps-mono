import Foundation
import SQLite3

/// Reads agy (Antigravity CLI) activity from `conversation_summaries.db`: steps per conversation,
/// workspace, and parent/child links. It never selects `title`, `preview` or `raw_summary`.
/// Token counts live in undocumented protobuf blobs and are not read yet; agy records no quota locally.
public struct AgyReader: UsageReader {
    public let tool = ToolID.agy
    let root: URL

    public init(root: URL = FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent(".gemini/antigravity-cli")) {
        self.root = root
    }

    var dbURL: URL { root.appendingPathComponent("conversation_summaries.db") }
    public var isInstalled: Bool { FileManager.default.fileExists(atPath: dbURL.path) }

    public func read(since: Date) throws -> ReaderOutput {
        guard isInstalled else { return ReaderOutput() }
        let snapshot = try SQLiteSnapshot(copying: dbURL)
        defer { snapshot.close() }
        let db = snapshot.db
        let sql = "SELECT conversation_id, step_count, last_modified_time, last_user_input_time, workspace_uris, parent_conversation_id FROM conversation_summaries"
        var stmt: OpaquePointer?
        guard sqlite3_prepare_v2(db, sql, -1, &stmt, nil) == SQLITE_OK else { throw CocoaError(.fileReadUnknown) }
        defer { sqlite3_finalize(stmt) }
        func text(_ i: Int32) -> String? { sqlite3_column_text(stmt, i).map { String(cString: $0) } }

        var all: [Session] = []
        var parentOf: [String: String] = [:]
        while sqlite3_step(stmt) == SQLITE_ROW {
            guard let id = text(0), let end = text(2).flatMap(Self.date) else { continue }
            let input = text(3).flatMap(Self.date)
            let start = (input.map { $0 > Date(timeIntervalSince1970: 0) } ?? false) ? input! : end
            if let p = text(5), !p.isEmpty { parentOf[id] = p }
            all.append(Session(tool: .agy, id: id, project: text(4).flatMap(Self.project),
                               start: min(start, end), end: end, isSubagent: parentOf[id] != nil,
                               turns: Int(sqlite3_column_int64(stmt, 1))))
        }
        // Fold child conversations into their parent so subagent work is counted once.
        var byID = Dictionary(uniqueKeysWithValues: all.map { ($0.id, $0) })
        for (child, parent) in parentOf {
            guard let c = byID[child], var p = byID[parent] else { continue }
            p.turns = (p.turns ?? 0) + (c.turns ?? 0)
            p.end = max(p.end, c.end)
            byID[parent] = p
            byID[child] = nil
        }
        return ReaderOutput(sessions: byID.values.filter { $0.end >= since }.sorted { $0.start < $1.start }, quotas: [])
    }

    /// "2026-10-20 16:31:28.552742+00:00" → Date
    static func date(_ s: String) -> Date? { Parse.date(s.replacingOccurrences(of: " ", with: "T")) }

    /// `["file:///Users/me/proj"]` → "/Users/me/proj"
    static func project(_ json: String) -> String? {
        guard let arr = try? JSONSerialization.jsonObject(with: Data(json.utf8)) as? [String], let first = arr.first,
              let url = URL(string: first) else { return nil }
        return url.path
    }
}
