import Foundation

public enum Format {
    /// 1_234 → "1.2k", 94_351 → "94k", 8_616_973 → "8.6M"
    public static func tokens(_ n: Int) -> String {
        switch n {
        case ..<1_000: "\(n)"
        case ..<10_000: String(format: "%.1fk", Double(n) / 1_000)
        case ..<1_000_000: String(format: "%.0fk", Double(n) / 1_000)
        case ..<10_000_000: String(format: "%.1fM", Double(n) / 1_000_000)
        default: String(format: "%.0fM", Double(n) / 1_000_000)
        }
    }

    /// "in 3 h", "in 2 d", "in 40 min", or "now"
    public static func until(_ date: Date, now: Date) -> String {
        let s = date.timeIntervalSince(now)
        if s <= 60 { return "now" }
        if s < 3600 { return "in \(Int(s / 60)) min" }
        if s < 86_400 { return "in \(Int((s / 3600).rounded())) h" }
        return "in \(Int((s / 86_400).rounded())) d"
    }

    public static func baseName(_ path: String) -> String { path.split(separator: "/").last.map(String.init) ?? path }
}

extension Dashboard {
    /// When no tool records a quota, the menu falls back to tokens (or turns) today.
    static func fallbackLines(_ statuses: [AccountStatus]) -> [String] {
        let live = statuses.filter { !$0.account.isHidden && $0.installed }
        let byTokens = live.filter { $0.tokensToday > 0 }.sorted { $0.tokensToday > $1.tokensToday }
        if !byTokens.isEmpty { return byTokens.prefix(3).map { "\($0.account.tool.displayName)  \(Format.tokens($0.tokensToday)) tokens today" } }
        return live.filter { $0.turnsToday > 0 }.sorted { $0.turnsToday > $1.turnsToday }.prefix(3)
            .map { "\($0.account.tool.displayName)  \($0.turnsToday) turns today" }
    }
}
