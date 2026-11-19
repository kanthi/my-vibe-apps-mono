import Foundation

public enum ToolID: String, Codable, Sendable, CaseIterable {
    case codex, grok, claudeCode, cursor, agy
}

/// One agent session. Never carries prompt or reply text.
public struct Session: Sendable, Equatable {
    public var tool: ToolID
    public var id: String
    public var accountID: String
    public var model: String?
    public var project: String?
    public var start: Date
    public var end: Date
    public var inputTokens: Int?
    public var cachedTokens: Int?
    public var outputTokens: Int?
    public var isSubagent: Bool
    /// Model calls or turns, for tools that record activity but not tokens.
    public var turns: Int?

    public init(tool: ToolID, id: String, accountID: String = "default", model: String? = nil,
                project: String? = nil, start: Date, end: Date, inputTokens: Int? = nil,
                cachedTokens: Int? = nil, outputTokens: Int? = nil, isSubagent: Bool = false, turns: Int? = nil) {
        self.tool = tool; self.id = id; self.accountID = accountID; self.model = model
        self.project = project; self.start = start; self.end = end
        self.inputTokens = inputTokens; self.cachedTokens = cachedTokens
        self.outputTokens = outputTokens; self.isSubagent = isSubagent; self.turns = turns
    }
}

/// A quota window as the tool last recorded it. Percent is kept as reported.
public struct Quota: Sendable, Equatable {
    public var tool: ToolID
    public var accountID: String
    public var label: String
    public var usedPercent: Double
    public var windowMinutes: Int?
    public var resetsAt: Date?
    public var observedAt: Date

    public init(tool: ToolID, accountID: String = "default", label: String, usedPercent: Double,
                windowMinutes: Int? = nil, resetsAt: Date? = nil, observedAt: Date) {
        self.tool = tool; self.accountID = accountID; self.label = label
        self.usedPercent = usedPercent; self.windowMinutes = windowMinutes
        self.resetsAt = resetsAt; self.observedAt = observedAt
    }
}

extension Quota {
    /// A window whose reset time has passed describes the past; do not present it as the present.
    public func isCurrent(at now: Date) -> Bool { resetsAt.map { $0 > now } ?? true }
}

public struct ReaderOutput: Sendable, Equatable {
    public var sessions: [Session] = []
    public var quotas: [Quota] = []
    public init(sessions: [Session] = [], quotas: [Quota] = []) {
        self.sessions = sessions; self.quotas = quotas
    }
}

public protocol UsageReader: Sendable {
    var tool: ToolID { get }
    /// False when the tool's data folder does not exist. Not installed is not zero usage.
    var isInstalled: Bool { get }
    func read(since: Date) throws -> ReaderOutput
}

enum Parse {
    /// Accepts "Z" or "+00:00" offsets, with or without fractional seconds.
    static func date(_ s: String) -> Date? {
        let styles: [Date.ISO8601FormatStyle] = [
            .init(includingFractionalSeconds: true),
            .init(),
            .init(timeZoneSeparator: .colon, includingFractionalSeconds: true),
            .init(timeZoneSeparator: .colon)
        ]
        for style in styles { if let d = try? Date(s, strategy: style) { return d } }
        return nil
    }

    static func int(_ v: Any?) -> Int? {
        if let n = v as? NSNumber { return n.intValue }
        return nil
    }

    static func windowLabel(minutes: Int?) -> String {
        guard let m = minutes else { return "Limit" }
        if m <= 600 { return "Session" }
        if m >= 9_000 && m <= 11_000 { return "Weekly" }
        if m >= 40_000 { return "Monthly" }
        return "\(m) min"
    }

    /// JSON objects for lines of a JSONL file that contain `marker`. Cheap substring test first.
    static func jsonLines(in url: URL, containing markers: [String]) -> [[String: Any]] {
        guard let data = try? Data(contentsOf: url) else { return [] }
        var out: [[String: Any]] = []
        for line in data.split(separator: UInt8(ascii: "\n")) {
            guard let s = String(data: line, encoding: .utf8), markers.contains(where: s.contains) else { continue }
            if let obj = try? JSONSerialization.jsonObject(with: Data(s.utf8)) as? [String: Any] { out.append(obj) }
        }
        return out
    }
}
