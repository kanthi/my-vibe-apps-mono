import SwiftUI
import AppKit
import TimedClockCore

@MainActor
func renderClock(filename: String, width: CGFloat = 1440, height: CGFloat = 900) {
    let model = TimedClockModel()
    model.stopTimer()

    let calendar = Calendar.current
    var comps = DateComponents()
    comps.year = 2026
    comps.month = 9
    comps.day = 28
    comps.hour = 9
    comps.minute = 41
    comps.second = 28

    if let date = calendar.date(from: comps) {
        model.currentDate = date
    }

    let view = TimedClockMainView(model: model)
        .frame(width: width, height: height)

    let hostingView = NSHostingView(rootView: view)
    hostingView.frame = NSRect(x: 0, y: 0, width: width, height: height)
    hostingView.layoutSubtreeIfNeeded()

    guard let bitmapRep = hostingView.bitmapImageRepForCachingDisplay(in: hostingView.bounds) else {
        print("Failed to allocate bitmapRep")
        return
    }
    hostingView.cacheDisplay(in: hostingView.bounds, to: bitmapRep)

    guard let pngData = bitmapRep.representation(using: .png, properties: [:]) else {
        print("Failed to encode PNG")
        return
    }

    let outURL = URL(fileURLWithPath: filename)
    do {
        try pngData.write(to: outURL)
        print("Successfully rendered \(filename) (\(Int(width))x\(Int(height)))")
    } catch {
        print("Error saving PNG: \(error)")
    }
}

@main
struct RenderPreview {
    @MainActor
    static func main() {
        let dest = "Resources/preview_dark.png"
        renderClock(filename: dest, width: 1440, height: 900)
    }
}
