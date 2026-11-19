import Foundation

public struct Alert: Equatable, Sendable {
    public enum Kind: String, Sendable { case threshold, exhausted, restored, pace }
    public var kind: Kind
    public var tool: ToolID
    public var accountID: String
    public var label: String
    public var title: String
    public var body: String
}

/// Delivery is behind a protocol so tests use a fake and never reach macOS notifications.
public protocol Notifier: Sendable {
    func deliver(_ alert: Alert)
}

/// Decides which alerts fire. Each fires once per account, window, and reset cycle.
/// The first evaluation only records a baseline, so a quota already high at launch stays quiet.
public final class AlertEngine: @unchecked Sendable {
    struct Entry { var resetsAt: Date?; var windowMinutes: Int?; var fired: Set<String> = []; var exhausted = false }

    public var thresholds: [Double]
    private var state: [String: Entry] = [:]
    private var baselined = false
    private let lock = NSLock()

    public init(thresholds: [Double] = [80, 95]) { self.thresholds = thresholds.sorted() }

    public func evaluate(_ quotas: [Quota]) -> [Alert] {
        lock.lock(); defer { lock.unlock() }
        var alerts: [Alert] = []
        for q in quotas {
            let key = "\(q.tool.rawValue)/\(q.accountID)/\(q.label)"
            var e = state[key] ?? Entry(resetsAt: q.resetsAt, windowMinutes: q.windowMinutes)
            if state[key] != nil, !sameCycle(e, q) {
                if e.exhausted && baselined {
                    alerts.append(make(.restored, q, "\(q.tool.displayName) \(q.label) has reset", "Room is available again."))
                }
                e = Entry(resetsAt: q.resetsAt, windowMinutes: q.windowMinutes)
            }
            let levels = thresholds.filter { q.usedPercent >= $0 }
            let current = Set(levels.map { "t\(Int($0))" } + (q.usedPercent >= 100 ? ["x"] : []))
            for l in levels.reversed().prefix(1) where baselined && !e.fired.contains("t\(Int(l))") && q.usedPercent < 100 {
                alerts.append(make(.threshold, q, "\(q.tool.displayName) \(q.label) at \(Insights.percent(q.usedPercent))",
                                   "Crossed \(Int(l))% of this window."))
            }
            if baselined, q.usedPercent >= 100, !e.exhausted {
                alerts.append(make(.exhausted, q, "\(q.tool.displayName) \(q.label) is used up", "The window is at 100%."))
            }
            if baselined, case .willRunOut(let at) = Insights.pace(q), q.usedPercent < 100, !e.fired.contains("p") {
                alerts.append(make(.pace, q, "\(q.tool.displayName) \(q.label) may run out",
                                   "At this pace it runs out \(Insights.when(at, now: q.observedAt))."))
                e.fired.insert("p")
            }
            e.fired.formUnion(current)
            // Already on course to run out at launch: part of the baseline, not news.
            if !baselined, case .willRunOut = Insights.pace(q) { e.fired.insert("p") }
            e.exhausted = q.usedPercent >= 100
            e.resetsAt = q.resetsAt; e.windowMinutes = q.windowMinutes
            state[key] = e
        }
        baselined = true
        return alerts
    }

    /// A slightly shifted reset time is still the same cycle.
    private func sameCycle(_ e: Entry, _ q: Quota) -> Bool {
        guard let a = e.resetsAt, let b = q.resetsAt else { return e.resetsAt == nil && q.resetsAt == nil }
        let tolerance = max(TimeInterval((q.windowMinutes ?? 0) * 60) / 2, 300)
        return abs(a.timeIntervalSince(b)) < tolerance
    }

    private func make(_ k: Alert.Kind, _ q: Quota, _ title: String, _ body: String) -> Alert {
        Alert(kind: k, tool: q.tool, accountID: q.accountID, label: q.label, title: title, body: body)
    }
}
