import Foundation

/// A tool can have several accounts (for example personal and work). Labels are the user's own;
/// Meter never reads credentials or emails to build them.
public struct Account: Codable, Equatable, Sendable, Identifiable {
    public var id: String
    public var tool: ToolID
    public var label: String
    public var dataRoot: String?      // alternate data folder for this account, nil = the tool's default
    public var isHidden: Bool

    public init(id: String = UUID().uuidString, tool: ToolID, label: String, dataRoot: String? = nil, isHidden: Bool = false) {
        self.id = id; self.tool = tool; self.label = label; self.dataRoot = dataRoot; self.isHidden = isHidden
    }
}

public struct AccountStore: Sendable {
    public let url: URL
    public init(url: URL) { self.url = url }

    public func load() -> [Account] {
        guard let data = try? Data(contentsOf: url), let a = try? JSONDecoder().decode([Account].self, from: data) else { return [] }
        return a
    }

    func save(_ accounts: [Account]) throws {
        try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        try JSONEncoder().encode(accounts).write(to: url, options: .atomic)
    }

    @discardableResult
    public func add(_ account: Account) throws -> [Account] {
        var all = load(); all.append(account); try save(all); return all
    }

    /// Hiding keeps the account and its history; it only leaves the menu, notes, and alerts.
    /// A tool's built-in account is created on first use so it can be hidden like any other.
    @discardableResult
    public func setHidden(_ hidden: Bool, tool: ToolID, id: String) throws -> [Account] {
        var all = load()
        if let i = all.firstIndex(where: { $0.tool == tool && $0.id == id }) { all[i].isHidden = hidden }
        else { all.append(Account(id: id, tool: tool, label: "Default", isHidden: hidden)) }
        try save(all); return all
    }

    @discardableResult
    public func remove(tool: ToolID, id: String) throws -> [Account] {
        let all = load().filter { !($0.tool == tool && $0.id == id) }
        try save(all); return all
    }

    /// Stored accounts plus a built-in "default" for each tool that has none stored.
    public func effective(tools: [ToolID] = ToolID.allCases) -> [Account] {
        var all = load()
        for t in tools where !all.contains(where: { $0.tool == t && $0.id == "default" }) {
            all.append(Account(id: "default", tool: t, label: "Default"))
        }
        return all.sorted { ($0.tool.rawValue, $0.id == "default" ? "" : $0.label) < ($1.tool.rawValue, $1.id == "default" ? "" : $1.label) }
    }

    /// "tool/accountID" keys, because every tool's built-in account is called "default".
    public func hiddenKeys() -> Set<String> { Set(load().filter(\.isHidden).map { "\($0.tool.rawValue)/\($0.id)" }) }
}

extension ReaderOutput {
    /// Stamps every record with the account it was read for.
    public func tagged(accountID: String) -> ReaderOutput {
        ReaderOutput(sessions: sessions.map { var s = $0; s.accountID = accountID; return s },
                     quotas: quotas.map { var q = $0; q.accountID = accountID; return q })
    }

    /// Drops sessions and quotas that belong to hidden accounts.
    public func excluding(hiddenKeys: Set<String>) -> ReaderOutput {
        ReaderOutput(sessions: sessions.filter { !hiddenKeys.contains("\($0.tool.rawValue)/\($0.accountID)") },
                     quotas: quotas.filter { !hiddenKeys.contains("\($0.tool.rawValue)/\($0.accountID)") })
    }
}
