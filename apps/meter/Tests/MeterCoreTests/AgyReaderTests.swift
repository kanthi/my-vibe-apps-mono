import Foundation
import SQLite3
import Testing
@testable import MeterCore

@Suite struct AgyReaderTests {
    func makeDB(_ root: URL) throws {
        var db: OpaquePointer?
        precondition(sqlite3_open(root.appendingPathComponent("conversation_summaries.db").path, &db) == SQLITE_OK)
        defer { sqlite3_close(db) }
        let sql = """
        CREATE TABLE conversation_summaries (conversation_id TEXT, title TEXT, preview TEXT, step_count INTEGER,
          last_modified_time TEXT, workspace_uris TEXT, parent_conversation_id TEXT, last_user_input_time TEXT);
        INSERT INTO conversation_summaries VALUES ('p','SECRET','SECRET',80,'2026-10-20 16:31:28.552742+00:00','["file:///Users/me/My%20Proj"]','','2026-10-20 16:30:36.79965+00:00');
        INSERT INTO conversation_summaries VALUES ('c','SECRET','SECRET',20,'2026-10-20 16:40:00.000000+00:00','["file:///Users/me/My%20Proj"]','p','0001-01-01 00:00:00+00:00');
        INSERT INTO conversation_summaries VALUES ('old','x','x',5,'2026-09-01 00:00:00+00:00','[]','','0001-01-01 00:00:00+00:00');
        """
        precondition(sqlite3_exec(db, sql, nil, nil, nil) == SQLITE_OK)
    }

    @Test func foldsSubagentIntoParentAndDecodesProject() throws {
        let root = try Fixtures.tempDir(); try makeDB(root)
        let cutoff = try #require(Parse.date("2026-10-01T00:00:00Z"))
        let out = try AgyReader(root: root).read(since: cutoff)
        #expect(out.sessions.map(\.id) == ["p"])
        let p = out.sessions[0]
        #expect(p.turns == 100)
        #expect(p.project == "/Users/me/My Proj")
        #expect(p.end == Parse.date("2026-10-20T16:40:00Z"))
        #expect(p.inputTokens == nil)
        #expect(out.quotas.isEmpty)
    }

    @Test func sinceDropsOldConversations() throws {
        let root = try Fixtures.tempDir(); try makeDB(root)
        let out = try AgyReader(root: root).read(since: .distantPast)
        #expect(out.sessions.count == 2)
    }

    @Test func notInstalled() throws {
        #expect(!AgyReader(root: try Fixtures.tempDir().appendingPathComponent("none")).isInstalled)
    }
}
