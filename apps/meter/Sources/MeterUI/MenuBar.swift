import MeterCore
import SwiftUI

/// The status item label: a symbol, plus the tightest percent when the setting allows.
public struct MenuBarLabel: View {
    let model: AppModel
    @Environment(\.openWindow) private var openWindow
    public init(model: AppModel) { self.model = model }

    public var body: some View {
        let p = model.dashboard.menu.tightestPercent
        HStack(spacing: 3) {
            Image(systemName: symbol(p))
            if model.settings.showPercentInMenuBar, let p { Text("\(Int(p.rounded()))%").monospacedDigit() }
        }
        .task {
            model.openMainWindow = { openWindow(id: "main") }
            model.start()   // the status item lives for the whole run, so refreshing starts here too
        }
        .accessibilityLabel(p.map { "Meter, tightest limit \(Int($0.rounded())) percent used" } ?? "Meter")
    }

    // A change of symbol, not color, signals a filling limit; the menu bar draws template images.
    private func symbol(_ p: Double?) -> String {
        guard let p else { return "gauge.with.dots.needle.33percent" }
        if p >= 90 { return "exclamationmark.gauge" }
        return p >= 60 ? "gauge.with.dots.needle.67percent" : "gauge.with.dots.needle.33percent"
    }
}

/// Minimal status menu. Detail lives in the window.
public struct MenuContent: View {
    let model: AppModel
    @Environment(\.openWindow) private var openWindow
    public init(model: AppModel) { self.model = model }

    public var body: some View {
        let menu = model.dashboard.menu
        if menu.lines.isEmpty {
            Text("No usage found yet")
        } else {
            ForEach(menu.lines, id: \.self) { Text($0) }
        }
        if let w = menu.warning { Text("⚠︎ \(w)") }
        Divider()
        Button("Open Meter…") {
            openWindow(id: "main")
            NSApp.activate(ignoringOtherApps: true)
        }
        Button("Refresh Now") { Task { await model.refresh() } }.keyboardShortcut("r")
        SettingsLink { Text("Settings…") }.keyboardShortcut(",")
        Divider()
        Button("Quit Meter") { NSApp.terminate(nil) }.keyboardShortcut("q")
    }
}
