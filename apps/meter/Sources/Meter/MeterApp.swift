import MeterUI
import SwiftUI

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    // Closing the window leaves Meter running in the menu bar.
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { false }

    func applicationDockMenu(_ sender: NSApplication) -> NSMenu? {
        let menu = NSMenu()
        for line in MeterApp.model.dashboard.menu.lines {
            let item = NSMenuItem(title: line, action: nil, keyEquivalent: ""); item.isEnabled = false; menu.addItem(item)
        }
        if !menu.items.isEmpty { menu.addItem(.separator()) }
        menu.addItem(NSMenuItem(title: "Open Meter", action: #selector(openMain), keyEquivalent: ""))
        menu.addItem(NSMenuItem(title: "Refresh Now", action: #selector(refresh), keyEquivalent: ""))
        menu.items.forEach { $0.target = self }
        return menu
    }

    @objc private func openMain() { MeterApp.model.openMainWindow?(); NSApp.activate(ignoringOtherApps: true) }
    @objc private func refresh() { Task { await MeterApp.model.refresh() } }
}

@main
struct MeterApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @MainActor static let model = AppModel.live()

    var body: some Scene {
        Window("Meter", id: "main") {
            MainView(model: Self.model)
                .task { Self.model.start() }
        }

        MenuBarExtra {
            MenuContent(model: Self.model)
        } label: {
            MenuBarLabel(model: Self.model)
        }
        .menuBarExtraStyle(.menu)

        Settings { SettingsView(model: Self.model) }
    }
}
