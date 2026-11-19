import Foundation
import MeterCore
import UserNotifications

/// Posts alerts through macOS notifications. Nothing here runs until the user turns an alert on,
/// and permission is requested at that moment, not at launch.
public final class UserNotifier: Notifier, @unchecked Sendable {
    public init() {}

    private var center: UNUserNotificationCenter? {
        // A bare executable (swift run, tests) has no bundle; the notification center would crash.
        Bundle.main.bundleIdentifier == nil ? nil : UNUserNotificationCenter.current()
    }

    public func requestAuthorization() async -> Bool {
        guard let center else { return false }
        return (try? await center.requestAuthorization(options: [.alert, .sound])) ?? false
    }

    public func deliver(_ alert: Alert) {
        guard let center else { return }
        let content = UNMutableNotificationContent()
        content.title = alert.title
        content.body = alert.body
        content.threadIdentifier = "\(alert.tool.rawValue)/\(alert.accountID)/\(alert.label)"
        center.add(UNNotificationRequest(identifier: UUID().uuidString, content: content, trigger: nil))
    }
}

/// Collects alerts instead of showing them. Used by tests and previews.
public final class RecordingNotifier: Notifier, @unchecked Sendable {
    private let lock = NSLock()
    private var items: [Alert] = []
    public init() {}
    public func deliver(_ alert: Alert) { lock.lock(); items.append(alert); lock.unlock() }
    public var delivered: [Alert] { lock.lock(); defer { lock.unlock() }; return items }
}
