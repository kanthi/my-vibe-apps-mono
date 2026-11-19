import AppKit
import SwiftUI
import TimedClockCore

/// Fullscreen runner to preview the Timed Clock screen saver faces.
///   ← / →      previous / next face
///   1 – 8      jump to a face
///   Space / T  Bauhaus face: toggle Light / Dark
///   Esc / Q    quit (other keys and clicks are ignored)
/// Arguments: --face <name> to start on a face, --cycle <secs> to tour every face.
@MainActor
final class PreviewState: ObservableObject {
    @Published var face: WatchFace = .obsidian
    @Published var bauhausTheme: BauhausDialTheme = .dark
    @Published var hintVisible = true
    private var hintToken = 0

    func select(_ newFace: WatchFace) {
        face = newFace
        showHint()
    }

    func step(_ delta: Int) {
        let all = WatchFace.allCases
        let i = all.firstIndex(of: face) ?? 0
        select(all[(i + delta + all.count) % all.count])
    }

    func toggleBauhausTheme() {
        guard face == .bauhaus else { return }
        bauhausTheme = bauhausTheme.isDark ? .light : .dark
        showHint()
    }

    func showHint() {
        hintToken += 1
        let token = hintToken
        hintVisible = true
        Task { @MainActor in
            try? await Task.sleep(for: .seconds(2.5))
            if token == self.hintToken { self.hintVisible = false }
        }
    }
}

struct PreviewRootView: View {
    @ObservedObject var state: PreviewState
    let model: TimedClockModel

    var body: some View {
        ZStack(alignment: .bottom) {
            TimedClockFaceView(model: model, face: state.face, bauhausTheme: state.bauhausTheme)
                .ignoresSafeArea()
            Text("\(state.face.displayName)    ← → or 1–\(WatchFace.allCases.count) switch face · Esc quits")
                .font(.system(size: 13, weight: .medium))
                .foregroundColor(.white.opacity(0.75))
                .padding(.horizontal, 14)
                .padding(.vertical, 7)
                .background(Capsule().fill(Color.black.opacity(0.45)))
                .padding(.bottom, 28)
                .opacity(state.hintVisible ? 1 : 0)
                .animation(.easeInOut(duration: 0.4), value: state.hintVisible)
        }
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
        window.title = "Timed Clock Screen Saver Preview"
        window.center()
        window.previewState = state

        let hostingView = NSHostingView(rootView: PreviewRootView(state: state, model: model))
        hostingView.frame = window.contentView!.bounds
        hostingView.autoresizingMask = [.width, .height]
        window.contentView = hostingView

        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        self.window = window

        // Optional arguments:
        //   --face <name>     start on a face (bauhaus, bauhausLume, obsidian, pilot, chronograph, lunar, meridian, abyss)
        //   --cycle <secs>    tour every face, one after another, then keep looping
        let args = CommandLine.arguments
        if let i = args.firstIndex(of: "--face"), i + 1 < args.count, let f = WatchFace(rawValue: args[i + 1]) {
            state.face = f
        }
        if let i = args.firstIndex(of: "--cycle"), i + 1 < args.count, let secs = Double(args[i + 1]), secs > 0 {
            if !args.contains("--face") { state.face = WatchFace.allCases[0] }
            cycleTimer = Timer.scheduledTimer(withTimeInterval: secs, repeats: true) { [weak self] _ in
                Task { @MainActor in self?.state.step(1) }
            }
        }
        state.showHint()
    }

    var cycleTimer: Timer?
}

@MainActor
class KeyWindow: NSWindow {
    weak var previewState: PreviewState?

    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { true }

    override func keyDown(with event: NSEvent) {
        let key = event.charactersIgnoringModifiers?.lowercased() ?? ""
        switch event.keyCode {
        case 53: // Esc
            NSApp.terminate(nil)
            return
        case 123: // ←
            previewState?.step(-1)
            return
        case 124: // →
            previewState?.step(1)
            return
        case 49: // Space
            previewState?.toggleBauhausTheme()
            return
        default:
            break
        }
        if key == "q" {
            NSApp.terminate(nil)
        } else if key == "t" {
            previewState?.toggleBauhausTheme()
        } else if let n = Int(key), n >= 1, n <= WatchFace.allCases.count {
            previewState?.select(WatchFace.allCases[n - 1])
        }
        // Any other key is ignored, so a stray key press no longer closes the preview.
    }

    override func mouseDown(with event: NSEvent) {
        // Ignored on purpose: a stray click no longer closes the preview.
    }
}
