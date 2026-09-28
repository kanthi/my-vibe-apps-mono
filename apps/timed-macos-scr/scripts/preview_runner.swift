import AppKit
import SwiftUI
import TimedClockCore

/// Fullscreen runner to preview the Bauhaus Clock macOS Screen Saver
/// Press SPACE or T to toggle between Light (Glacier White) and Dark (BGW9 Cyan Lume) modes.
/// Press ESC or Q to exit.
@MainActor
final class PreviewState: ObservableObject {
    @Published var theme: BauhausDialTheme = .light

    func toggleTheme() {
        if theme.isDark {
            theme = .light
        } else {
            theme = .dark
        }
    }
}

struct PreviewRootView: View {
    @ObservedObject var state: PreviewState
    @ObservedObject var model: TimedClockModel

    var body: some View {
        TimedClockMainView(model: model, theme: state.theme)
            .ignoresSafeArea()
    }
}

@main
@MainActor
final class PreviewApp: NSObject, NSApplicationDelegate {
    var window: NSWindow!
    let model = TimedClockModel.shared
    let state = PreviewState()

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

        let window = KeyWindow(
            contentRect: screenRect,
            styleMask: [.borderless, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        window.level = .floating
        window.isOpaque = true
        window.backgroundColor = .black
        window.title = "Bauhaus Clock Screen Saver Preview"
        window.center()
        window.previewState = state

        let rootView = PreviewRootView(state: state, model: model)
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
    weak var previewState: PreviewState?

    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { true }

    override func keyDown(with event: NSEvent) {
        if event.keyCode == 49 || event.charactersIgnoringModifiers?.lowercased() == "t" {
            // Spacebar (keyCode 49) or 'T' toggles Light / Dark mode
            previewState?.toggleTheme()
            return
        }
        // ESC (53) or Q or any other key dismisses preview
        NSApp.terminate(nil)
    }

    override func mouseDown(with event: NSEvent) {
        NSApp.terminate(nil)
    }
}
