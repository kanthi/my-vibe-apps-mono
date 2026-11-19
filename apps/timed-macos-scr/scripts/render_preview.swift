import SwiftUI
import AppKit
import TimedClockCore

@MainActor
func renderClock(face: WatchFace, bauhausTheme: BauhausDialTheme? = nil, filename: String, width: CGFloat = 1440, height: CGFloat = 900) {
    let model = TimedClockModel()
    model.stopTimer()

    let calendar = Calendar.current
    var comps = DateComponents()
    comps.year = 2026
    comps.month = 9
    comps.day = 28
    // 10:10:35 - Classic horological display time
    comps.hour = 10
    comps.minute = 10
    comps.second = 35

    if let date = calendar.date(from: comps) {
        model.currentDate = date
    }

    let view = TimedClockFaceView(model: model, face: face, bauhausTheme: bauhausTheme)
        .frame(width: width, height: height)

    let hostingView = NSHostingView(rootView: view)
    hostingView.frame = NSRect(x: 0, y: 0, width: width, height: height)
    hostingView.layoutSubtreeIfNeeded()

    guard let bitmapRep = hostingView.bitmapImageRepForCachingDisplay(in: hostingView.bounds) else {
        print("Failed to allocate bitmapRep for \(filename)")
        return
    }
    hostingView.cacheDisplay(in: hostingView.bounds, to: bitmapRep)

    guard let pngData = bitmapRep.representation(using: .png, properties: [:]) else {
        print("Failed to encode PNG for \(filename)")
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
    /// Usage: render_preview [output-dir]   (default: Resources)
    @MainActor
    static func main() {
        let dir = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "Resources"
        renderClock(face: .bauhaus, bauhausTheme: .light, filename: "\(dir)/preview_light.png")
        renderClock(face: .bauhaus, bauhausTheme: .dark, filename: "\(dir)/preview_dark.png")
        for face in WatchFace.allCases where face != .bauhaus {
            renderClock(face: face, filename: "\(dir)/preview_\(face.rawValue).png")
        }
    }
}
