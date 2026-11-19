import Foundation
import Testing
@testable import MeterCore

@Suite struct ClaudeCodeReaderTests {
    func line(_ rid: String, _ out: Int, ts: String, session: String = "s1", sidechain: Bool = false, cwd: String = "/p/app") -> String {
        #"{"type":"assistant","timestamp":"\#(ts)","sessionId":"\#(session)","requestId":"\#(rid)","cwd":"\#(cwd)","isSidechain":\#(sidechain),"message":{"model":"claude-x","content":"SECRET TEXT","usage":{"input_tokens":10,"cache_creation_input_tokens":100,"cache_read_input_tokens":1000,"output_tokens":\#(out)}}}"#
    }

    @Test func dedupesStreamedRequestsAndSumsTokens() throws {
        let root = try Fixtures.tempDir()
        let f = root.appendingPathComponent("projects/-p-app/s1.jsonl")
        try Fixtures.write([
            line("r1", 5, ts: "2026-10-24T10:00:00.000Z"),
            line("r1", 50, ts: "2026-10-24T10:00:01.000Z"),   // fuller duplicate wins
            line("r2", 7, ts: "2026-10-24T10:05:00.000Z"),
            #"{"type":"user","timestamp":"2026-10-24T10:06:00.000Z","message":{"content":"hi"}}"#
        ].joined(separator: "\n"), to: f)
        let out = try ClaudeCodeReader(root: root).read(since: .distantPast)
        let s = try #require(out.sessions.first)
        #expect(out.sessions.count == 1)
        #expect(s.turns == 2)
        #expect(s.outputTokens == 57)
        #expect(s.inputTokens == 2 * 110)
        #expect(s.cachedTokens == 2000)
        #expect(s.model == "claude-x")
        #expect(s.project == "/p/app")
        #expect(out.quotas.isEmpty)
    }

    @Test func subagentWorkCountsUnderParentSession() throws {
        let root = try Fixtures.tempDir()
        try Fixtures.write(line("r1", 10, ts: "2026-10-24T10:00:00.000Z"), to: root.appendingPathComponent("projects/-p/s1.jsonl"))
        try Fixtures.write(line("r9", 20, ts: "2026-10-24T10:01:00.000Z", sidechain: true), to: root.appendingPathComponent("projects/-p/s1/subagents/agent-1.jsonl"))
        let out = try ClaudeCodeReader(root: root).read(since: .distantPast)
        #expect(out.sessions.count == 1)
        #expect(out.sessions[0].outputTokens == 30)
    }

    @Test func sinceFilters() throws {
        let root = try Fixtures.tempDir()
        try Fixtures.write(line("r1", 10, ts: "2026-10-01T10:00:00.000Z"), to: root.appendingPathComponent("projects/-p/s1.jsonl"))
        let cutoff = try #require(Parse.date("2026-10-10T00:00:00Z"))
        #expect(try ClaudeCodeReader(root: root).read(since: cutoff).sessions.isEmpty)
    }

    @Test func absentFolderIsNotInstalled() throws {
        #expect(!ClaudeCodeReader(root: try Fixtures.tempDir().appendingPathComponent("none")).isInstalled)
    }
}
