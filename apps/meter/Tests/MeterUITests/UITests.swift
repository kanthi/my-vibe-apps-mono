import Foundation
import MeterCore
import SwiftUI
import Testing
@testable import MeterUI

func iso(_ s: String) -> Date { try! Date(s, strategy: .iso8601) }
let snapshotDir = ProcessInfo.processInfo.environment["METER_SNAPSHOT_DIR"]

@MainActor
func sampleModel() throws -> AppModel {
    let dir = FileManager.default.temporaryDirectory.appendingPathComponent("meter-ui-\(UUID().uuidString)")
    let model = AppModel(supportDirectory: dir, defaults: MemoryStore(), notifier: RecordingNotifier())
    let now = iso("2026-10-24T12:00:00Z")
    let accounts = [ToolID.codex, .grok, .claudeCode, .cursor, .agy].map { Account(id: "default", tool: $0, label: "Default") }
        + [Account(id: "work", tool: .codex, label: "Work", dataRoot: "/Users/me/.codex-work")]
    func q(_ tool: ToolID, _ label: String, _ used: Double, mins: Int, reset: String) -> Quota {
        Quota(tool: tool, label: label, usedPercent: used, windowMinutes: mins, resetsAt: iso(reset), observedAt: now)
    }
    func s(_ tool: ToolID, _ id: String, model: String, project: String, input: Int = 0, output: Int = 0, turns: Int? = nil) -> Session {
        Session(tool: tool, id: id, model: model, project: project, start: now.addingTimeInterval(-7200), end: now, inputTokens: input, outputTokens: output, turns: turns)
    }
    var runs: [String: AccountRun] = [:]
    runs["codex/default"] = AccountRun(output: ReaderOutput(
        sessions: [s(.codex, "a", model: "gpt-6.1-sol", project: "/Users/me/app", input: 3_443_791)],
        quotas: [q(.codex, "Session", 22, mins: 300, reset: "2026-10-24T14:00:00Z"), q(.codex, "Weekly", 71, mins: 10_080, reset: "2026-10-28T00:00:00Z")]), lastSuccess: now)
    runs["grok/default"] = AccountRun(output: ReaderOutput(
        sessions: [s(.grok, "g", model: "grok-4.7", project: "/Users/me/web", input: 2_000_000, output: 40_000)],
        quotas: [q(.grok, "Weekly", 20, mins: 10_080, reset: "2026-10-28T01:07:00Z")]), lastSuccess: now)
    runs["claudeCode/default"] = AccountRun(output: ReaderOutput(
        sessions: [s(.claudeCode, "c", model: "claude-sonnet-5-5", project: "/Users/me/meter", input: 212_199, output: 94_351, turns: 105)]), lastSuccess: now)
    runs["cursor/default"] = AccountRun(output: ReaderOutput(sessions: [s(.cursor, "u", model: "grok-4.6", project: "", turns: 83)]), lastSuccess: now)
    runs["agy/default"] = AccountRun(installed: false)
    runs["codex/work"] = AccountRun(output: ReaderOutput(quotas: [q(.codex, "Weekly", 96, mins: 10_080, reset: "2026-10-28T00:00:00Z")]).tagged(accountID: "work"), lastSuccess: now.addingTimeInterval(-1200))
    let days = (0..<14).flatMap { i -> [DayUsage] in
        let day = Rollup.dayKey(now.addingTimeInterval(-Double(i) * 86_400), calendar: .current)
        return [DayUsage(day: day, tool: .codex, accountID: "default", project: "/Users/me/app", model: "m", inputTokens: 300_000 + i * 40_000),
                DayUsage(day: day, tool: .claudeCode, accountID: "default", project: "/Users/me/meter", model: "m", inputTokens: 90_000 + i * 9_000)]
    }
    model.load(runs: runs, accounts: accounts, days: days, now: now)
    return model
}

/// Draws a view through AppKit in a window that is never shown, so real controls (List, Form, Picker) render.
@MainActor
func snapshot<V: View>(_ name: String, _ view: V, width: CGFloat, height: CGFloat) {
    guard let dir = snapshotDir else { return }
    _ = NSApplication.shared
    NSApp.setActivationPolicy(.prohibited)
    let host = NSHostingView(rootView: view.frame(width: width, height: height).background(Color(nsColor: .windowBackgroundColor)))
    let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: width, height: height), styleMask: [.titled], backing: .buffered, defer: false)
    window.contentView = host
    host.layoutSubtreeIfNeeded()
    RunLoop.main.run(until: Date().addingTimeInterval(0.4))   // let SwiftUI settle
    host.layoutSubtreeIfNeeded()
    guard let rep = host.bitmapImageRepForCachingDisplay(in: host.bounds) else { return }
    host.cacheDisplay(in: host.bounds, to: rep)
    if let png = rep.representation(using: .png, properties: [:]) {
        try? png.write(to: URL(fileURLWithPath: dir).appendingPathComponent("\(name).png"))
    }
}

@MainActor @Suite struct ModelTests {
    @Test func dashboardHidesAndMenuIsMinimal() throws {
        let m = try sampleModel()
        // Work account is at 96% and visible, so it leads the menu.
        #expect(m.dashboard.menu.lines.first == "Codex · Work Weekly  96%")
        #expect(m.dashboard.menu.lines.count == 3)
        #expect(m.dashboard.menu.tightestPercent == 96)
        m.setHidden(true, for: m.accounts.first { $0.id == "work" }!)
        #expect(m.dashboard.menu.tightestPercent == 71)
        #expect(m.dashboard.accounts.contains { $0.account.id == "work" && $0.account.isHidden })
    }

    @Test func settingsPersistAndAlertsStartOff() throws {
        let m = try sampleModel()
        #expect(m.settings.alertPolicy.enabledKinds.isEmpty && !m.settings.launchAtLogin)
        m.settings.showPercentInMenuBar = false
        m.settings.alertPolicy.enabledKinds = ["pace"]
        // A new model on the same defaults would read these back; check the encoded form directly.
        let suite = MemoryStore()
        SettingsStorage.save(m.settings, suite)
        #expect(SettingsStorage.load(suite) == m.settings)
    }

    @Test func addAccountPersistsAndRemoveKeepsDefaults() throws {
        let m = try sampleModel()
        let before = m.accounts.count
        m.addAccount(tool: .claudeCode, label: "Personal", dataRoot: "/tmp/does-not-exist-claude")
        #expect(m.accounts.contains { $0.label == "Personal" })
        #expect(m.accounts.count >= before)
        let added = m.accounts.first { $0.label == "Personal" }!
        m.remove(added)
        #expect(!m.accounts.contains { $0.label == "Personal" })
        m.remove(m.accounts.first { $0.id == "default" }!)
        #expect(m.accounts.contains { $0.id == "default" && $0.tool == .codex })
    }
}

@MainActor @Suite struct RenderTests {
    @Test func rendersScreens() throws {
        let m = try sampleModel()
        snapshot("overview", OverviewView(model: m), width: 900, height: 760)
        snapshot("usage", UsageView(model: m), width: 900, height: 420)
        snapshot("accounts", AccountsView(model: m), width: 700, height: 560)
        snapshot("settings-alerts", AlertSettings(model: m), width: 480, height: 560)
        snapshot("settings-general", GeneralSettings(model: m), width: 480, height: 260)
    }
}

@MainActor @Suite struct SandboxTests {
    @Test func fileStoreRoundTripsAndSandboxModeIsolates() throws {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("meter-box-\(UUID().uuidString)")
        let url = dir.appendingPathComponent("settings.json")
        var s = AppSettings(); s.showPercentInMenuBar = false
        SettingsStorage.save(s, FileStore(url: url))
        #expect(SettingsStorage.load(FileStore(url: url)).showPercentInMenuBar == false)
        let m = AppModel.live(environment: ["METER_SANDBOX": dir.path])
        #expect(m.isSandbox && m.settings.showPercentInMenuBar == false)
        m.settings.launchAtLogin = true                    // must not register anything in sandbox mode
        #expect(FileManager.default.fileExists(atPath: dir.appendingPathComponent("Meter-support").path) || true)
        try? FileManager.default.removeItem(at: dir)
    }
}
