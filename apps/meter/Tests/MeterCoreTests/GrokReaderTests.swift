import Foundation
import Testing
@testable import MeterCore

@Suite struct GrokReaderTests {
    func log(_ root: URL) throws {
        let lines = [
            #"{"ts":"2026-10-23T10:00:00.000Z","sid":"s1","msg":"shell.turn.inference_done","ctx":{"prompt_tokens":1000,"cached_prompt_tokens":200,"completion_tokens":50,"reasoning_tokens":30}}"#,
            #"{"ts":"2026-10-23T10:05:00.000Z","sid":"s1","msg":"shell.turn.inference_done","ctx":{"prompt_tokens":500,"cached_prompt_tokens":100,"completion_tokens":20,"reasoning_tokens":0}}"#,
            #"{"ts":"2026-10-23T11:00:00.000Z","msg":"billing: fetched credits config","ctx":{"config":{"creditUsagePercent":10.0,"currentPeriod":{"type":"USAGE_PERIOD_TYPE_WEEKLY","start":"2026-10-21T01:00:00+00:00","end":"2026-10-28T01:00:00+00:00"}}}}"#,
            #"{"ts":"2026-10-24T11:00:00.000Z","msg":"billing: fetched credits config","ctx":{"config":{"creditUsagePercent":20.0,"currentPeriod":{"type":"USAGE_PERIOD_TYPE_WEEKLY","start":"2026-10-21T01:00:00+00:00","end":"2026-10-28T01:00:00+00:00"}}}}"#,
            #"{"ts":"2026-10-24T11:00:00.000Z","msg":"startup timing","ctx":{}}"#
        ]
        try Fixtures.write(lines.joined(separator: "\n") + "\n", to: root.appendingPathComponent("logs/unified.jsonl"))
    }

    @Test func latestWeeklyQuota() throws {
        let root = try Fixtures.tempDir(); try log(root)
        let q = try #require(try GrokReader(root: root).read(since: .distantPast).quotas.first)
        #expect(q.label == "Weekly")
        #expect(q.usedPercent == 20.0)
        #expect(q.windowMinutes == 7 * 24 * 60)
        #expect(q.resetsAt == Parse.date("2026-10-28T01:00:00Z"))
    }

    @Test func sumsTokensPerSessionAndResolvesProject() throws {
        let root = try Fixtures.tempDir(); try log(root)
        try Fixtures.write(#"{"ts":"x","type":"turn_started","model_id":"grok-x"}"# + "\n",
                           to: root.appendingPathComponent("sessions/%2FUsers%2Fme%2Fapp/s1/events.jsonl"))
        let s = try #require(try GrokReader(root: root).read(since: .distantPast).sessions.first)
        #expect(s.id == "s1")
        #expect(s.inputTokens == 1500)
        #expect(s.cachedTokens == 300)
        #expect(s.outputTokens == 100)
        #expect(s.project == "/Users/me/app")
        #expect(s.model == "grok-x")
    }

    @Test func sinceFiltersOldInferences() throws {
        let root = try Fixtures.tempDir(); try log(root)
        let cutoff = try #require(Parse.date("2026-10-24T00:00:00Z"))
        #expect(try GrokReader(root: root).read(since: cutoff).sessions.isEmpty)
    }
}
