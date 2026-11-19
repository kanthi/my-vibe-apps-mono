import AppKit
import ScreenSaver
import SwiftUI
import TimedClockCore

/// Official macOS ScreenSaverView implementation
/// Conforms to Apple's ScreenSaver framework lifecycle and animation requirements.
/// The face (Bauhaus or one of the realistic watches) is picked in
/// System Settings › Screen Saver › Options… and kept in ScreenSaverDefaults.
@objc(TimedClockScreenSaverView)
public class TimedClockScreenSaverView: ScreenSaverView {
    private static let faceKey = "face"

    private var hostingView: NSHostingView<TimedClockFaceView>?
    private let model = TimedClockModel()
    private var sheet: NSWindow?
    private var facePopup: NSPopUpButton?

    private lazy var defaults: ScreenSaverDefaults? = {
        let id = Bundle(for: TimedClockScreenSaverView.self).bundleIdentifier ?? "com.vibe.timed-macos-scr"
        return ScreenSaverDefaults(forModuleWithName: id)
    }()

    private var face: WatchFace {
        WatchFace(rawValue: defaults?.string(forKey: Self.faceKey) ?? "") ?? .bauhaus
    }

    public override init?(frame: NSRect, isPreview: Bool) {
        super.init(frame: frame, isPreview: isPreview)
        setupView()
    }

    public required init?(coder: NSCoder) {
        super.init(coder: coder)
        setupView()
    }

    private func setupView() {
        // Configure 60 FPS animation rate for continuous live second hand sweep
        animationTimeInterval = 1.0 / 60.0
        wantsLayer = true

        let host = NSHostingView(rootView: TimedClockFaceView(model: model, face: face))
        host.frame = bounds
        host.autoresizingMask = [.width, .height]
        addSubview(host)
        self.hostingView = host
    }

    public override func startAnimation() {
        super.startAnimation()
        model.startTimer()
    }

    public override func stopAnimation() {
        super.stopAnimation()
        model.stopTimer()
    }

    public override func animateOneFrame() {
        // Advances time each animation frame tick
        model.tick()
    }

    // MARK: - Options sheet (face picker)

    public override var hasConfigureSheet: Bool {
        return true
    }

    public override var configureSheet: NSWindow? {
        if let sheet {
            facePopup?.selectItem(at: WatchFace.allCases.firstIndex(of: face) ?? 0)
            return sheet
        }

        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 360, height: 120),
            styleMask: [.titled],
            backing: .buffered,
            defer: true
        )
        window.title = "Timed Clock"

        let label = NSTextField(labelWithString: "Watch face:")
        label.frame = NSRect(x: 20, y: 72, width: 90, height: 20)

        let popup = NSPopUpButton(frame: NSRect(x: 110, y: 66, width: 230, height: 28), pullsDown: false)
        popup.addItems(withTitles: WatchFace.allCases.map(\.displayName))
        popup.selectItem(at: WatchFace.allCases.firstIndex(of: face) ?? 0)

        let ok = NSButton(title: "OK", target: self, action: #selector(closeConfigureSheet(_:)))
        ok.frame = NSRect(x: 250, y: 16, width: 90, height: 32)
        ok.bezelStyle = .rounded
        ok.keyEquivalent = "\r"

        window.contentView?.addSubview(label)
        window.contentView?.addSubview(popup)
        window.contentView?.addSubview(ok)

        self.sheet = window
        self.facePopup = popup
        return window
    }

    @objc private func closeConfigureSheet(_ sender: Any?) {
        if let popup = facePopup {
            let index = max(0, popup.indexOfSelectedItem)
            let chosen = WatchFace.allCases[index]
            defaults?.set(chosen.rawValue, forKey: Self.faceKey)
            defaults?.synchronize()
            hostingView?.rootView = TimedClockFaceView(model: model, face: chosen)
        }
        guard let sheet else { return }
        if let parent = sheet.sheetParent {
            parent.endSheet(sheet)
        } else {
            sheet.orderOut(nil)
        }
    }
}
