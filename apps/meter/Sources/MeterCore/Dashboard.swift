import Foundation

/// The last result for one account. A failed refresh keeps the previous output.
public struct AccountRun: Sendable {
    public var installed: Bool
    public var output: ReaderOutput
    public var lastSuccess: Date?
    public var error: String?
    public init(installed: Bool = true, output: ReaderOutput = ReaderOutput(), lastSuccess: Date? = nil, error: String? = nil) {
        self.installed = installed; self.output = output; self.lastSuccess = lastSuccess; self.error = error
    }
}

public enum Refresher {
    /// Reads every account concurrently. On failure the last good output stays and the error is recorded.
    public static func run(accounts: [Account], previous: [String: AccountRun], now: Date, since: Date) async -> [String: AccountRun] {
        await withTaskGroup(of: (String, AccountRun).self) { group in
            for account in accounts {
                let key = Dashboard.key(account)
                let prior = previous[key]
                group.addTask {
                    let reader = ReaderRegistry.make(account.tool, root: account.dataRoot.map { URL(fileURLWithPath: $0) })
                    guard reader.isInstalled else { return (key, AccountRun(installed: false)) }
                    do {
                        let out = try reader.read(since: since).tagged(accountID: account.id)
                        return (key, AccountRun(output: out, lastSuccess: now))
                    } catch {
                        var kept = prior ?? AccountRun()
                        kept.installed = true; kept.error = "Could not read \(account.tool.displayName) data"
                        return (key, kept)
                    }
                }
            }
            var result: [String: AccountRun] = [:]
            for await (k, r) in group { result[k] = r }
            return result
        }
    }
}

public struct AccountStatus: Identifiable, Equatable, Sendable {
    public var id: String
    public var account: Account
    public var installed: Bool
    public var quotas: [Quota]            // current windows only
    public var endedWindows: [Quota]      // windows whose reset time has passed
    public var tokensToday: Int
    public var turnsToday: Int
    public var sessionsToday: Int
    public var topModel: String?
    public var topProject: String?
    public var lastSuccess: Date?
    public var error: String?
    public var isStale: Bool
}

public struct Dashboard: Sendable {
    public var accounts: [AccountStatus]
    public var notes: [Note]
    public var menu: MenuSummary
    public var days: [DayUsage]
    public var visibleQuotas: [Quota]

    public static func key(_ a: Account) -> String { "\(a.tool.rawValue)/\(a.id)" }

    public static func build(accounts: [Account], runs: [String: AccountRun], days: [DayUsage],
                             calendar: Calendar, now: Date, staleAfter: TimeInterval = 600) -> Dashboard {
        let today = Rollup.dayKey(now, calendar: calendar)
        let visibleKeys = Set(accounts.filter { !$0.isHidden }.map(key))
        var statuses: [AccountStatus] = []
        for a in accounts {
            let run = runs[key(a)] ?? AccountRun(installed: true)
            let todays = run.output.sessions.filter { Rollup.dayKey($0.start, calendar: calendar) == today && !$0.isSubagent }
            func top(_ f: (Session) -> String?) -> String? {
                var w: [String: Int] = [:]
                for s in todays { if let k = f(s), !k.isEmpty { w[k, default: 0] += (s.inputTokens ?? 0) + (s.outputTokens ?? 0) + (s.turns ?? 0) } }
                return w.max { $0.value < $1.value }?.key
            }
            statuses.append(AccountStatus(
                id: key(a), account: a, installed: run.installed,
                quotas: run.output.quotas.filter { $0.isCurrent(at: now) }.sorted { $0.label < $1.label },
                endedWindows: run.output.quotas.filter { !$0.isCurrent(at: now) },
                tokensToday: todays.reduce(0) { $0 + ($1.inputTokens ?? 0) + ($1.outputTokens ?? 0) },
                turnsToday: todays.reduce(0) { $0 + ($1.turns ?? 0) },
                sessionsToday: todays.count,
                topModel: top(\.model), topProject: top(\.project),
                lastSuccess: run.lastSuccess, error: run.error,
                isStale: run.lastSuccess.map { now.timeIntervalSince($0) > staleAfter } ?? false))
        }
        let quotas = statuses.filter { visibleKeys.contains($0.id) && $0.installed }.flatMap(\.quotas)
        let visibleDays = days.filter { visibleKeys.contains("\($0.tool.rawValue)/\($0.accountID)") }
        let labels = Dictionary(accounts.map { (key($0), $0.label) }, uniquingKeysWith: { a, _ in a })
        let notes = Insights.notes(quotas: quotas, days: visibleDays, today: today, now: now, labels: labels)
        var menu = MenuSummary.build(quotas: quotas, notes: notes, labels: labels)
        if menu.lines.isEmpty { menu.lines = fallbackLines(statuses) }
        return Dashboard(accounts: statuses, notes: notes, menu: menu, days: visibleDays, visibleQuotas: quotas)
    }
}

/// Alert delivery rules the user controls: which kinds, quiet hours, snooze.
public struct AlertPolicy: Codable, Equatable, Sendable {
    public var enabledKinds: Set<String> = []      // Alert.Kind raw values; empty = all off
    public var quietStartMinute: Int? = nil        // minutes after midnight
    public var quietEndMinute: Int? = nil
    public var snoozeUntil: Date? = nil
    public init() {}

    public func allows(_ alert: Alert, at now: Date, calendar: Calendar) -> Bool {
        guard enabledKinds.contains(alert.kind.rawValue) else { return false }
        if let s = snoozeUntil, now < s { return false }
        if let qs = quietStartMinute, let qe = quietEndMinute, qs != qe {
            let c = calendar.dateComponents([.hour, .minute], from: now)
            let m = (c.hour ?? 0) * 60 + (c.minute ?? 0)
            let inQuiet = qs < qe ? (m >= qs && m < qe) : (m >= qs || m < qe)   // overnight window wraps midnight
            if inQuiet { return false }
        }
        return true
    }
}
