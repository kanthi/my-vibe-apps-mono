import Foundation
import SQLite3

/// Reads Cursor's composer records from `state.vscdb`. Values hold the conversation, so the
/// query extracts only four scalar fields with `json_extract`; no message text is selected.
/// Cursor records no tokens or quota locally (token fields are zero), so only turns are reported.
/// Grok Bot shows up as the model name on these records.
public struct CursorReader: UsageReader {
    public let tool = ToolID.cursor
    let support: URL

    public init(support: URL = FileManager.default.homeDirectoryForCurrentUser
        .appendingPathComponent("Library/Application Support/Cursor")) {
        self.support = support
    }

    var dbURL: URL { support.appendingPathComponent("User/globalStorage/state.vscdb") }
    public var isInstalled: Bool { FileManager.default.fileExists(atPath: dbURL.path) }

    public func read(since: Date) throws -> ReaderOutput {
        guard isInstalled else { return ReaderOutput() }
        let snapshot = try SQLiteSnapshot(copying: dbURL)
        defer { snapshot.close() }
        let db = snapshot.db
        let sql = """
        SELECT key,
               json_extract(value, '$.createdAt'),
               json_extract(value, '$.lastUpdatedAt'),
               json_extract(value, '$.modelConfig.modelName'),
               json_array_length(value, '$.fullConversationHeadersOnly')
        FROM cursorDiskKV WHERE key LIKE 'composerData:%'
        """
        var stmt: OpaquePointer?
        guard sqlite3_prepare_v2(db, sql, -1, &stmt, nil) == SQLITE_OK else { throw CocoaError(.fileReadUnknown) }
        defer { sqlite3_finalize(stmt) }
        var out: [Session] = []
        while sqlite3_step(stmt) == SQLITE_ROW {
            guard sqlite3_column_type(stmt, 1) != SQLITE_NULL else { continue }
            let created = Date(timeIntervalSince1970: Double(sqlite3_column_int64(stmt, 1)) / 1000)
            let updated = sqlite3_column_type(stmt, 2) == SQLITE_NULL
                ? created : Date(timeIntervalSince1970: Double(sqlite3_column_int64(stmt, 2)) / 1000)
            let turns = Int(sqlite3_column_int64(stmt, 4))
            guard turns > 0, updated >= since else { continue }   // empty drafts are not usage
            let id = sqlite3_column_text(stmt, 0).map { String(cString: $0) }?
                .replacingOccurrences(of: "composerData:", with: "") ?? ""
            out.append(Session(tool: .cursor, id: id, model: sqlite3_column_text(stmt, 3).map { String(cString: $0) },
                               start: created, end: updated, turns: turns))
        }
        return ReaderOutput(sessions: out.sorted { $0.start < $1.start }, quotas: [])
    }
}
