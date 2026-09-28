import SwiftUI
import Combine

/// Real-time clock model driving the analog and digital displays with high-precision 60 FPS sweep
@MainActor
public final class TimedClockModel: ObservableObject {
    public static let shared = TimedClockModel()

    @Published public var currentDate: Date = Date()
    private var timer: Timer?

    public init() {
        startTimer()
    }

    public func tick() {
        self.currentDate = Date()
    }

    public func startTimer() {
        timer?.invalidate()
        // 60 FPS timer for continuous fluid second hand glide
        timer = Timer.scheduledTimer(withTimeInterval: 1.0 / 60.0, repeats: true) { [weak self] _ in
            Task { @MainActor [weak self] in
                self?.currentDate = Date()
            }
        }
        if let timer = timer {
            RunLoop.main.add(timer, forMode: .common)
        }
    }

    public func stopTimer() {
        timer?.invalidate()
        timer = nil
    }

    // MARK: - Time Components
    public var calendar: Calendar { Calendar.current }

    public var hour12: Int {
        let h = calendar.component(.hour, from: currentDate) % 12
        return h == 0 ? 12 : h
    }

    public var hourString: String {
        String(format: "%02d", hour12)
    }

    public var minuteString: String {
        let m = calendar.component(.minute, from: currentDate)
        return String(format: "%02d", m)
    }

    public var secondString: String {
        let s = calendar.component(.second, from: currentDate)
        return String(format: "%02d", s)
    }

    public var isPM: Bool {
        calendar.component(.hour, from: currentDate) >= 12
    }

    public var amPmLetter: String {
        isPM ? "P" : "A"
    }

    // Precise sub-second time for smooth continuous motion
    private var preciseSecond: Double {
        let sec = Double(calendar.component(.second, from: currentDate))
        let nano = Double(calendar.component(.nanosecond, from: currentDate))
        return sec + (nano / 1_000_000_000.0)
    }

    private var minuteFraction: Double {
        let min = Double(calendar.component(.minute, from: currentDate))
        return min + (preciseSecond / 60.0)
    }

    /// Hour angle in degrees (0 at 12 o'clock, 90 at 3 o'clock, 180 at 6 o'clock)
    public var hourHandDegrees: Double {
        let hour = Double(calendar.component(.hour, from: currentDate) % 12)
        let totalHours = hour + (minuteFraction / 60.0)
        return totalHours * 30.0
    }

    /// Minute angle in degrees
    public var minuteHandDegrees: Double {
        return minuteFraction * 6.0
    }

    /// Live sweeping second angle in degrees (silky continuous 60 FPS motion)
    public var secondHandDegrees: Double {
        return preciseSecond * 6.0
    }
}
