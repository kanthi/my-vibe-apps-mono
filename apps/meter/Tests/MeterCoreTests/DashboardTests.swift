import Foundation
import Testing
@testable import MeterCore

@Suite struct DashboardTests {
    let now = d("2026-10-24T12:00:00Z")
    func quota(_ tool: ToolID, _ used: Double, resets: String = "2026-10-28T00:00:00Z") -> Quota {
        Quota(tool: tool, label: "Weekly", usedPercent: used, windowMinutes: 7 * 1440, resetsAt: d(resets), observedAt: now)
    }

    @Test func buildsStatusAndSeparatesEndedWindowsAndHiddenAccounts() {
        let codex = Account(id: "default", tool: .codex, label: "Default")
        let grok = Account(id: "default", tool: .grok, label: "Default", isHidden: true)
        let runs: [String: AccountRun] = [
            "codex/default": AccountRun(output: ReaderOutput(
                sessions: [Session(tool: .codex, id: "s", model: "gpt", project: "/p", start: d("2026-10-24T09:00:00Z"), end: now, inputTokens: 900, outputTokens: 100)],
                quotas: [quota(.codex, 60), quota(.codex, 10, resets: "2026-10-23T00:00:00Z")]), lastSuccess: now),
            "grok/default": AccountRun(output: ReaderOutput(quotas: [quota(.grok, 99)]), lastSuccess: now)
        ]
        let dash = Dashboard.build(accounts: [codex, grok], runs: runs, days: [], calendar: utc, now: now)
        let c = dash.accounts.first { $0.account.tool == .codex }!
        #expect(c.tokensToday == 1000 && c.topModel == "gpt" && c.quotas.count == 1 && c.endedWindows.count == 1)
        #expect(dash.visibleQuotas.map(\.tool) == [.codex])          // hidden Grok (99%) is out of the menu
        #expect(dash.menu.lines == ["Codex Weekly  60%"])
        #expect(dash.accounts.first { $0.account.tool == .grok }?.account.isHidden == true)
    }

    @Test func staleTagAfterTenMinutesAndNotInstalled() {
        let a = Account(id: "default", tool: .codex, label: "Default")
        let old = AccountRun(output: ReaderOutput(), lastSuccess: now.addingTimeInterval(-700), error: "Could not read Codex data")
        let dash = Dashboard.build(accounts: [a, Account(id: "default", tool: .cursor, label: "Default")],
                                   runs: ["codex/default": old, "cursor/default": AccountRun(installed: false)], days: [], calendar: utc, now: now)
        #expect(dash.accounts[0].isStale && dash.accounts[0].error != nil)
        #expect(!dash.accounts[1].installed)
    }

    @Test func refresherKeepsLastGoodOnFailureAndMarksMissing() async throws {
        let missing = Account(id: "x", tool: .codex, label: "Gone", dataRoot: "/nonexistent/path/xyz")
        let runs = await Refresher.run(accounts: [missing], previous: [:], now: now, since: .distantPast)
        #expect(runs["codex/x"]?.installed == false)
    }

    @Test func alertPolicy() {
        var p = AlertPolicy()
        let alert = Alert(kind: .threshold, tool: .codex, accountID: "default", label: "Weekly", title: "t", body: "b")
        #expect(!p.allows(alert, at: now, calendar: utc))                     // all off by default
        p.enabledKinds = ["threshold"]
        #expect(p.allows(alert, at: now, calendar: utc))
        p.quietStartMinute = 22 * 60; p.quietEndMinute = 8 * 60               // 22:00 to 08:00 wraps midnight
        #expect(p.allows(alert, at: now, calendar: utc))                      // 12:00 is outside quiet hours
        #expect(!p.allows(alert, at: d("2026-10-24T23:30:00Z"), calendar: utc))
        #expect(!p.allows(alert, at: d("2026-10-24T07:00:00Z"), calendar: utc))
        p.quietStartMinute = nil; p.snoozeUntil = now.addingTimeInterval(3600)
        #expect(!p.allows(alert, at: now, calendar: utc))
    }

    @Test func effectiveAccountsAddsBuiltInDefaultsAndUpsertsHide() throws {
        let store = AccountStore(url: try Fixtures.tempDir().appendingPathComponent("a.json"))
        #expect(store.effective(tools: [.codex, .grok]).count == 2)
        try store.setHidden(true, tool: .grok, id: "default")
        #expect(store.hiddenKeys() == ["grok/default"])
        #expect(store.effective(tools: [.codex, .grok]).count == 2)
    }
}

@Suite struct FormatTests {
    @Test func formats() {
        #expect(Format.tokens(999) == "999" && Format.tokens(1_234) == "1.2k" && Format.tokens(94_351) == "94k")
        #expect(Format.tokens(8_616_973) == "8.6M")
        let now = d("2026-10-24T12:00:00Z")
        #expect(Format.until(now.addingTimeInterval(30), now: now) == "now")
        #expect(Format.until(now.addingTimeInterval(7200), now: now) == "in 2 h")
        #expect(Format.until(now.addingTimeInterval(3 * 86_400), now: now) == "in 3 d")
        #expect(Format.baseName("/Users/me/app") == "app")
    }

    @Test func menuFallsBackToTokensThenTurnsWhenNoQuota() {
        let now = d("2026-10-24T12:00:00Z")
        let a = Account(id: "default", tool: .claudeCode, label: "Default")
        let c = Account(id: "default", tool: .cursor, label: "Default")
        let runs: [String: AccountRun] = [
            "claudeCode/default": AccountRun(output: ReaderOutput(sessions: [Session(tool: .claudeCode, id: "1", start: d("2026-10-24T09:00:00Z"), end: now, inputTokens: 90_000, outputTokens: 4_000)]), lastSuccess: now),
            "cursor/default": AccountRun(output: ReaderOutput(sessions: [Session(tool: .cursor, id: "2", start: d("2026-10-24T09:00:00Z"), end: now, turns: 83)]), lastSuccess: now)
        ]
        let dash = Dashboard.build(accounts: [a, c], runs: runs, days: [], calendar: utc, now: now)
        #expect(dash.menu.lines == ["Claude Code  94k tokens today"])
        let only = Dashboard.build(accounts: [c], runs: runs, days: [], calendar: utc, now: now)
        #expect(only.menu.lines == ["Cursor  83 turns today"])
    }
}
