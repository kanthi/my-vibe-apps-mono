import Foundation

/// One day of usage for a tool, account, project, and model. Survives the tools' own log cleanup.
public struct DayUsage: Codable, Equatable, Sendable {
    public var day: String            // "yyyy-MM-dd" in the user's calendar
    public var tool: ToolID
    public var accountID: String
    public var project: String
    public var model: String
    public var inputTokens = 0
    public var cachedTokens = 0
    public var outputTokens = 0
    public var turns = 0
    public var sessions = 0

    public init(day: String, tool: ToolID, accountID: String, project: String, model: String,
                inputTokens: Int = 0, cachedTokens: Int = 0, outputTokens: Int = 0, turns: Int = 0, sessions: Int = 0) {
        self.day = day; self.tool = tool; self.accountID = accountID; self.project = project; self.model = model
        self.inputTokens = inputTokens; self.cachedTokens = cachedTokens; self.outputTokens = outputTokens
        self.turns = turns; self.sessions = sessions
    }

    public var key: String { [day, tool.rawValue, accountID, project, model].joined(separator: "|") }
    public var tokens: Int { inputTokens + outputTokens }
}

public enum Rollup {
    public static func dayKey(_ date: Date, calendar: Calendar) -> String {
        let c = calendar.dateComponents([.year, .month, .day], from: date)
        return String(format: "%04d-%02d-%02d", c.year!, c.month!, c.day!)
    }

    /// A session counts wholly toward the day it started.
    public static func days(from sessions: [Session], calendar: Calendar) -> [DayUsage] {
        var map: [String: DayUsage] = [:]
        for s in sessions where !s.isSubagent {
            var d = DayUsage(day: dayKey(s.start, calendar: calendar), tool: s.tool, accountID: s.accountID,
                             project: s.project ?? "", model: s.model ?? "")
            var cur = map[d.key] ?? d
            cur.inputTokens += s.inputTokens ?? 0
            cur.cachedTokens += s.cachedTokens ?? 0
            cur.outputTokens += s.outputTokens ?? 0
            cur.turns += s.turns ?? 0
            cur.sessions += 1
            d = cur
            map[d.key] = d
        }
        return map.values.sorted { $0.key < $1.key }
    }
}

/// JSON file holding all days seen so far. Path is injected so tests never use the real one.
public struct RollupStore: Sendable {
    public let url: URL
    public init(url: URL) { self.url = url }

    public func load() -> [DayUsage] {
        guard let data = try? Data(contentsOf: url) else { return [] }
        return (try? JSONDecoder().decode([DayUsage].self, from: data)) ?? []
    }

    /// Replaces stored days for the given tools from `fromDay` onward with fresh data, so a re-read
    /// never double counts. Older days (already cleaned up by the tool) are kept.
    @discardableResult
    public func merge(_ fresh: [DayUsage], tools: Set<ToolID>, fromDay: String) throws -> [DayUsage] {
        var kept = load().filter { !(tools.contains($0.tool) && $0.day >= fromDay) }
        kept += fresh
        kept.sort { $0.key < $1.key }
        try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        try JSONEncoder().encode(kept).write(to: url, options: .atomic)
        return kept
    }
}
