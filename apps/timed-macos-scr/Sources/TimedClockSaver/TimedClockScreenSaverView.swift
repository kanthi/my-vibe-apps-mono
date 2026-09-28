import AppKit
import ScreenSaver
import SwiftUI
import TimedClockCore

/// Official macOS ScreenSaverView implementation
/// Conforms to Apple's ScreenSaver framework lifecycle and animation requirements.
@objc(TimedClockScreenSaverView)
public class TimedClockScreenSaverView: ScreenSaverView {
    private var hostingView: NSHostingView<TimedClockMainView>?
    private let model = TimedClockModel()

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

        let mainView = TimedClockMainView(model: model)
        let host = NSHostingView(rootView: mainView)
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

    public override var hasConfigureSheet: Bool {
        // Pure dark screensaver with zero extraneous settings
        return false
    }

    public override var configureSheet: NSWindow? {
        return nil
    }
}
