import Foundation
import SQLite3

/// Builds throwaway tool folders in a temp directory. Tests never read the real home folder.
enum Fixtures {
    static func tempDir() throws -> URL {
        let url = FileManager.default.temporaryDirectory.appendingPathComponent("meter-test-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }

    static func write(_ text: String, to url: URL) throws {
        try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        try text.write(to: url, atomically: true, encoding: .utf8)
    }

    static func codexDB(at url: URL, rows: [(id: String, created: Int, updated: Int, source: String, cwd: String, model: String, tokens: Int)]) throws {
        var db: OpaquePointer?
        precondition(sqlite3_open(url.path, &db) == SQLITE_OK)
        defer { sqlite3_close(db) }
        // Includes a text-bearing column to prove the reader never touches it.
        var sql = "CREATE TABLE threads (id TEXT, created_at INTEGER, updated_at INTEGER, source TEXT, cwd TEXT, model TEXT, tokens_used INTEGER, title TEXT);"
        for r in rows {
            sql += "INSERT INTO threads VALUES ('\(r.id)', \(r.created), \(r.updated), '\(r.source)', '\(r.cwd)', '\(r.model)', \(r.tokens), 'SECRET TITLE');"
        }
        precondition(sqlite3_exec(db, sql, nil, nil, nil) == SQLITE_OK)
    }
}

extension Fixtures {
    /// Cursor-style key/value table whose values are JSON that includes a decoy message text.
    static func cursorDB(at url: URL, values: [(key: String, json: String)]) throws {
        try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        var db: OpaquePointer?
        precondition(sqlite3_open(url.path, &db) == SQLITE_OK)
        defer { sqlite3_close(db) }
        precondition(sqlite3_exec(db, "CREATE TABLE cursorDiskKV (key TEXT, value BLOB);", nil, nil, nil) == SQLITE_OK)
        for v in values {
            var stmt: OpaquePointer?
            sqlite3_prepare_v2(db, "INSERT INTO cursorDiskKV VALUES (?, ?)", -1, &stmt, nil)
            sqlite3_bind_text(stmt, 1, v.key, -1, unsafeBitCast(-1, to: sqlite3_destructor_type.self))
            sqlite3_bind_text(stmt, 2, v.json, -1, unsafeBitCast(-1, to: sqlite3_destructor_type.self))
            precondition(sqlite3_step(stmt) == SQLITE_DONE)
            sqlite3_finalize(stmt)
        }
    }
}
