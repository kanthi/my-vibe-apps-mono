import Foundation

public enum PaceVerdict: Equatable, Sendable {
    case notEnoughData
    case onCourse(projectedPercent: Double)
    case tight(projectedPercent: Double)
    case willRunOut(at: Date)
}

public struct Note: Equatable, Sendable {
    public enum Kind: String, Sendable { case closestToCap, pace, outOfPattern, idleQuota }
    public var kind: Kind
    public var text: String
}

/// Pure functions. A note exists only when it is true.
public enum Insights {
    /// The window with the least room left.
    public static func tightest(_ quotas: [Quota]) -> Quota? { quotas.max { $0.usedPercent < $1.usedPercent } }

    public static func pace(_ q: Quota) -> PaceVerdict {
        guard let reset = q.resetsAt, let mins = q.windowMinutes, mins > 0 else { return .notEnoughData }
        let window = TimeInterval(mins * 60)
        let start = reset.addingTimeInterval(-window)
        let elapsed = q.observedAt.timeIntervalSince(start)
        guard elapsed >= window * 0.05, elapsed < window, q.usedPercent > 0 else { return .notEnoughData }
        if q.usedPercent >= 100 { return .willRunOut(at: q.observedAt) }
        let projected = q.usedPercent / elapsed * window
        if projected >= 100 { return .willRunOut(at: start.addingTimeInterval(elapsed * 100 / q.usedPercent)) }
        return projected > 90 ? .tight(projectedPercent: projected) : .onCourse(projectedPercent: projected)
    }

    /// "Codex", or "Codex · Work" for an added account, using the user's own label.
    public static func name(_ q: Quota, labels: [String: String] = [:]) -> String {
        guard q.accountID != "default" else { return q.tool.displayName }
        return "\(q.tool.displayName) · \(labels["\(q.tool.rawValue)/\(q.accountID)"] ?? q.accountID)"
    }

    public static func notes(quotas: [Quota], days: [DayUsage], today: String, now: Date, labels: [String: String] = [:]) -> [Note] {
        var out: [Note] = []
        if let t = tightest(quotas), t.usedPercent >= 50 {
            out.append(Note(kind: .closestToCap,
                            text: "\(name(t, labels: labels)) \(t.label) is closest to its cap: \(percent(t.usedPercent)) used."))
        }
        for q in quotas {
            if case .willRunOut(let at) = pace(q) {
                out.append(Note(kind: .pace, text: "At today's pace \(name(q, labels: labels)) \(q.label) runs out \(when(at, now: now)), before it resets."))
            }
        }
        out += outOfPattern(days: days, today: today)
        if let idle = idleQuota(quotas) { out.append(idle) }
        return out
    }

    /// A project whose tokens today are several times its recent daily average.
    public static func outOfPattern(days: [DayUsage], today: String, factor: Double = 3, minTokens: Int = 50_000) -> [Note] {
        struct K: Hashable { var tool: ToolID; var project: String }
        var todayTokens: [K: Int] = [:]
        var prior: [K: [String: Int]] = [:]
        for d in days where !d.project.isEmpty {
            let k = K(tool: d.tool, project: d.project)
            if d.day == today { todayTokens[k, default: 0] += d.tokens }
            else if d.day < today { prior[k, default: [:]][d.day, default: 0] += d.tokens }
        }
        return todayTokens.compactMap { k, t in
            let daily = (prior[k] ?? [:]).values.filter { $0 > 0 }
            guard daily.count >= 3, t >= minTokens else { return nil }
            let mean = Double(daily.reduce(0, +)) / Double(daily.count)
            guard Double(t) >= factor * mean else { return nil }
            let name = k.project.split(separator: "/").last.map(String.init) ?? k.project
            return Note(kind: .outOfPattern,
                        text: "\(k.tool.displayName) in \(name) used about \(String(format: "%.0f", Double(t) / mean))× its recent daily average today.")
        }.sorted { $0.text < $1.text }
    }

    /// One tool barely used while another is near its cap.
    public static func idleQuota(_ quotas: [Quota], nearCap: Double = 80, barely: Double = 10) -> Note? {
        var worst: [ToolID: Double] = [:]
        for q in quotas { worst[q.tool] = max(worst[q.tool] ?? 0, q.usedPercent) }
        guard let hot = worst.filter({ $0.value >= nearCap }).max(by: { $0.value < $1.value }),
              let idle = worst.filter({ $0.key != hot.key && $0.value <= barely }).min(by: { $0.value < $1.value })
        else { return nil }
        return Note(kind: .idleQuota,
                    text: "\(idle.key.displayName) is barely used (\(percent(idle.value))) while \(hot.key.displayName) is at \(percent(hot.value)).")
    }

    static func percent(_ v: Double) -> String { String(format: "%.0f%%", v) }

    static func when(_ at: Date, now: Date) -> String {
        let s = at.timeIntervalSince(now)
        if s <= 60 { return "now" }
        if s < 3600 { return "in about \(Int(s / 60)) min" }
        if s < 86_400 { return "in about \(Int((s / 3600).rounded())) h" }
        return "in about \(Int((s / 86_400).rounded())) d"
    }
}

/// What the minimal menu shows: at most `limit` lines, tightest first, plus a warning only when one is true.
public struct MenuSummary: Equatable, Sendable {
    public var lines: [String]
    public var warning: String?
    public var tightestPercent: Double?

    public static func build(quotas: [Quota], notes: [Note], limit: Int = 3, labels: [String: String] = [:]) -> MenuSummary {
        let sorted = quotas.sorted { $0.usedPercent > $1.usedPercent }
        var seen = Set<String>()
        let lines = sorted.compactMap { q -> String? in
            guard seen.insert("\(q.tool.rawValue)/\(q.accountID)/\(q.label)").inserted else { return nil }
            return "\(Insights.name(q, labels: labels)) \(q.label)  \(Insights.percent(q.usedPercent))"
        }.prefix(limit)
        let warning = notes.first { $0.kind == .pace || $0.kind == .outOfPattern }?.text
        return MenuSummary(lines: Array(lines), warning: warning, tightestPercent: sorted.first?.usedPercent)
    }
}
