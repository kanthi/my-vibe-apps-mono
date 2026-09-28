import AppKit
import SwiftUI
import TimedClockCore

/// Minimal ephemeral runner to preview the Timed macOS Screen Saver in full-screen
/// without installing or touching system preferences.
@main
@MainActor
final class PreviewApp: NSObject, NSApplicationDelegate {
    var window: NSWindow!
    let model = TimedClockModel.shared

    static func main() {
        let app = NSApplication.shared
        let delegate = PreviewApp()
        app.delegate = delegate
        app.setActivationPolicy(.regular)
        app.run()
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        let screen = NSScreen.main ?? NSScreen.screens[0]
        let screenRect = screen.frame

        // Create fullscreen preview window
        let window = KeyWindow(
            contentRect: screenRect,
            styleMask: [.borderless, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        window.level = .floating
        window.isOpaque = true
        window.backgroundColor = .black
        window.title = "Timed Screen Saver Preview"
        window.center()

        let rootView = TimedClockMainView(model: model)
        let hostingView = NSHostingView(rootView: rootView)
        hostingView.frame = window.contentView!.bounds
        hostingView.autoresizingMask = [.width, .height]
        window.contentView = hostingView

        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        self.window = window
    }
}

@MainActor
class KeyWindow: NSWindow {
    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { true }

    override func keyDown(with event: NSEvent) {
        // Any key dismisses the screensaver preview (Esc, Q, Space, Return, etc.)
        NSApp.terminate(nil)
    }

    override func mouseDown(with event: NSEvent) {
        // Mouse click also dismisses the screensaver preview
        NSApp.terminate(nil)
    }
}
