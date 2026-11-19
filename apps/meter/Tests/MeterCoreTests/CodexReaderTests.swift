import Foundation
import Testing
@testable import MeterCore

@Suite struct CodexReaderTests {
    @Test func readsThreadsAndFlagsSubagents() throws {
        let root = try Fixtures.tempDir()
        try Fixtures.codexDB(at: root.appendingPathComponent("state_5.sqlite"), rows: [
            ("a", 1_000, 2_000, "vscode", "/p/one", "gpt-x", 500),
            ("b", 1_000, 2_100, "{\"subagent\":{\"other\":\"guardian\"}}", "/p/one", "auto-review", 40),
            ("old", 10, 20, "vscode", "/p/old", "gpt-x", 9)
        ])
        let out = try CodexReader(root: root).read(since: Date(timeIntervalSince1970: 1_500))
        #expect(out.sessions.map(\.id).sorted() == ["a", "b"])
        let b = try #require(out.sessions.first { $0.id == "b" })
        #expect(b.isSubagent)
        #expect(out.sessions.first { $0.id == "a" }?.inputTokens == 500)
        #expect(out.sessions.first { $0.id == "a" }?.project == "/p/one")
    }

    @Test func picksNewestQuotaAndLabelsWindows() throws {
        let root = try Fixtures.tempDir()
        let f = root.appendingPathComponent("sessions/2026/10/03/rollout-x.jsonl")
        let older = #"{"timestamp":"2026-10-03T10:00:00.000Z","type":"event_msg","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":1.0,"window_minutes":300,"resets_at":1800000000}}}}"#
        let newer = #"{"timestamp":"2026-10-03T11:00:00.000Z","type":"event_msg","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":2.0,"window_minutes":300,"resets_at":1800000000},"secondary":{"used_percent":40.5,"window_minutes":10080,"resets_at":1800500000}}}}"#
        try Fixtures.write(newer + "\n" + older + "\nnot json\n", to: f)
        let q = try CodexReader(root: root).read(since: .distantPast).quotas
        #expect(q.map(\.label) == ["Session", "Weekly"])
        #expect(q.map(\.usedPercent) == [2.0, 40.5])
        #expect(q[1].resetsAt == Date(timeIntervalSince1970: 1_800_500_000))
    }

    @Test func missingFolderIsNotInstalledAndEmpty() throws {
        let reader = CodexReader(root: try Fixtures.tempDir().appendingPathComponent("nope"))
        #expect(!reader.isInstalled)
        #expect(try reader.read(since: .distantPast) == ReaderOutput())
    }
}
