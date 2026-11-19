import Foundation
import Testing
@testable import MeterCore

let utc: Calendar = { var c = Calendar(identifier: .gregorian); c.timeZone = TimeZone(identifier: "UTC")!; return c }()
func d(_ s: String) -> Date { Parse.date(s)! }

@Suite struct RollupTests {
    @Test func groupsByDayProjectModelAndSkipsSubagents() {
        let s = [
            Session(tool: .codex, id: "a", model: "m", project: "/p", start: d("2026-10-24T10:00:00Z"), end: d("2026-10-24T11:00:00Z"), inputTokens: 100, outputTokens: 10, turns: 2),
            Session(tool: .codex, id: "b", model: "m", project: "/p", start: d("2026-10-24T12:00:00Z"), end: d("2026-10-24T13:00:00Z"), inputTokens: 50, outputTokens: 5, turns: 1),
            Session(tool: .codex, id: "c", model: "m", project: "/p", start: d("2026-10-24T12:00:00Z"), end: d("2026-10-24T13:00:00Z"), inputTokens: 999, isSubagent: true),
            Session(tool: .codex, id: "e", model: "m", project: "/p", start: d("2026-10-23T12:00:00Z"), end: d("2026-10-23T13:00:00Z"), inputTokens: 7)
        ]
        let days = Rollup.days(from: s, calendar: utc)
        #expect(days.count == 2)
        let today = days.first { $0.day == "2026-10-24" }!
        #expect(today.tokens == 165 && today.turns == 3 && today.sessions == 2)
    }

    @Test func storeKeepsOldDaysAndReplacesWindowWithoutDoubleCounting() throws {
        let url = try Fixtures.tempDir().appendingPathComponent("rollup.json")
        let store = RollupStore(url: url)
        func day(_ day: String, _ t: Int) -> DayUsage { DayUsage(day: day, tool: .claudeCode, accountID: "default", project: "/p", model: "m", inputTokens: t) }
        try store.merge([day("2026-10-01", 5), day("2026-10-24", 10)], tools: [.claudeCode], fromDay: "2026-10-01")
        // The tool later cleans up Oct 1; a re-read returns only Oct 24 (now larger).
        let all = try store.merge([day("2026-10-24", 30)], tools: [.claudeCode], fromDay: "2026-10-20")
        #expect(all.map(\.day) == ["2026-10-01", "2026-10-24"])
        #expect(all.last?.tokens == 30)
    }
}

@Suite struct AccountTests {
    @Test func addHideUnhideAndFilter() throws {
        let store = AccountStore(url: try Fixtures.tempDir().appendingPathComponent("accounts.json"))
        try store.add(Account(id: "work", tool: .codex, label: "Work"))
        try store.add(Account(id: "default", tool: .grok, label: "Personal"))
        try store.setHidden(true, tool: .codex, id: "work")
        #expect(store.hiddenKeys() == ["codex/work"])
        let out = ReaderOutput(
            sessions: [Session(tool: .codex, id: "1", accountID: "work", start: .now, end: .now),
                       Session(tool: .codex, id: "2", accountID: "default", start: .now, end: .now)],
            quotas: [Quota(tool: .grok, accountID: "default", label: "Weekly", usedPercent: 5, observedAt: .now)])
        let shown = out.excluding(hiddenKeys: store.hiddenKeys())
        #expect(shown.sessions.map(\.id) == ["2"])
        #expect(shown.quotas.count == 1)            // Grok's "default" is not hidden by Codex's "work"
        try store.setHidden(false, tool: .codex, id: "work")
        #expect(store.hiddenKeys().isEmpty && store.load().count == 2)
    }
}

@Suite struct InsightTests {
    let reset = d("2026-10-28T00:00:00Z")
    func weekly(_ used: Double, at: String, tool: ToolID = .codex) -> Quota {
        Quota(tool: tool, label: "Weekly", usedPercent: used, windowMinutes: 7 * 1440, resetsAt: reset, observedAt: d(at))
    }

    @Test func paceVerdicts() {
        // Day 3.5 of 7 = halfway.
        #expect(Insights.pace(weekly(40, at: "2026-10-24T12:00:00Z")) == .onCourse(projectedPercent: 80))
        if case .tight = Insights.pace(weekly(47, at: "2026-10-24T12:00:00Z")) {} else { Issue.record("expected tight") }
        // 70% used at the halfway point: 100% is reached at 5 of 7 days.
        #expect(Insights.pace(weekly(70, at: "2026-10-24T12:00:00Z")) == .willRunOut(at: d("2026-10-21T00:00:00Z").addingTimeInterval(3.5 * 86_400 / 0.7)))
        #expect(Insights.pace(weekly(0, at: "2026-10-24T12:00:00Z")) == .notEnoughData)
        #expect(Insights.pace(weekly(5, at: "2026-10-21T01:00:00Z")) == .notEnoughData)   // under 5% of the window elapsed
    }

    @Test func tightestAndNotes() {
        let q = [weekly(82, at: "2026-10-24T12:00:00Z"), weekly(3, at: "2026-10-24T12:00:00Z", tool: .grok)]
        #expect(Insights.tightest(q)?.tool == .codex)
        let notes = Insights.notes(quotas: q, days: [], today: "2026-10-24", now: d("2026-10-24T12:00:00Z"))
        #expect(notes.map(\.kind).contains(.closestToCap))
        #expect(notes.map(\.kind).contains(.pace))
        #expect(notes.map(\.kind).contains(.idleQuota))
        #expect(Insights.notes(quotas: [weekly(20, at: "2026-10-24T12:00:00Z")], days: [], today: "2026-10-24", now: .now).isEmpty)
    }

    @Test func addedAccountsAreNamedByTheirLabel() {
        var work = weekly(90, at: "2026-10-24T12:00:00Z"); work.accountID = "abc"
        let notes = Insights.notes(quotas: [work], days: [], today: "2026-10-24", now: d("2026-10-24T12:00:00Z"), labels: ["codex/abc": "Work"])
        #expect(notes.first?.text.hasPrefix("Codex · Work Weekly is closest") == true)
        #expect(MenuSummary.build(quotas: [work], notes: [], labels: ["codex/abc": "Work"]).lines == ["Codex · Work Weekly  90%"])
    }

    @Test func outOfPattern() {
        func u(_ day: String, _ t: Int) -> DayUsage { DayUsage(day: day, tool: .claudeCode, accountID: "default", project: "/w/app", model: "m", inputTokens: t) }
        let history = [u("2026-10-20", 100_000), u("2026-10-21", 120_000), u("2026-10-22", 80_000)]
        #expect(Insights.outOfPattern(days: history + [u("2026-10-24", 500_000)], today: "2026-10-24").count == 1)
        #expect(Insights.outOfPattern(days: history + [u("2026-10-24", 150_000)], today: "2026-10-24").isEmpty)
        #expect(Insights.outOfPattern(days: [u("2026-10-22", 1), u("2026-10-24", 900_000)], today: "2026-10-24").isEmpty)  // too little history
    }

    @Test func menuSummaryIsMinimal() {
        let q = [weekly(10, at: "2026-10-24T12:00:00Z", tool: .grok), weekly(82, at: "2026-10-24T12:00:00Z"),
                 Quota(tool: .codex, label: "Session", usedPercent: 40, observedAt: .now),
                 Quota(tool: .cursor, label: "Limit", usedPercent: 1, observedAt: .now)]
        let s = MenuSummary.build(quotas: q, notes: [], limit: 3)
        #expect(s.lines == ["Codex Weekly  82%", "Codex Session  40%", "Grok Weekly  10%"])
        #expect(s.warning == nil && s.tightestPercent == 82)
    }
}

@Suite struct AlertTests {
    func q(_ used: Double, reset: String = "2026-10-28T00:00:00Z", at: String = "2026-10-24T12:00:00Z") -> Quota {
        Quota(tool: .codex, label: "Weekly", usedPercent: used, windowMinutes: 7 * 1440, resetsAt: d(reset), observedAt: d(at))
    }

    @Test func baselineThenOncePerThreshold() {
        let e = AlertEngine()
        #expect(e.evaluate([q(85)]).isEmpty)                      // already high at launch: baseline only
        #expect(e.evaluate([q(86)]).isEmpty)
        #expect(e.evaluate([q(96)]).map(\.kind) == [.threshold])  // new crossing at 95
        #expect(e.evaluate([q(97)]).isEmpty)                      // no repeat
    }

    @Test func exhaustedThenRestoredOnNewCycleAndNoAlertForSmallResetShift() {
        let e = AlertEngine()
        _ = e.evaluate([q(50)])
        #expect(e.evaluate([q(100)]).map(\.kind) == [.exhausted])
        #expect(e.evaluate([q(100, reset: "2026-10-28T00:03:00Z")]).isEmpty)   // reset time jitter = same cycle
        let after = e.evaluate([q(2, reset: "2026-11-04T00:00:00Z", at: "2026-10-28T01:00:00Z")])
        #expect(after.map(\.kind) == [.restored])
    }

    @Test func paceWarningOnce() {
        let e = AlertEngine()
        _ = e.evaluate([q(10)])
        #expect(e.evaluate([q(70)]).map(\.kind).contains(.pace))
        #expect(!e.evaluate([q(71)]).map(\.kind).contains(.pace))
    }
}
