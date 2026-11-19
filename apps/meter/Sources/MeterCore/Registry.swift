import Foundation

extension ToolID {
    public var displayName: String {
        switch self {
        case .codex: "Codex"
        case .grok: "Grok"
        case .claudeCode: "Claude Code"
        case .cursor: "Cursor"
        case .agy: "agy"
        }
    }
}

public enum ReaderRegistry {
    /// Every reader Meter knows about, pointed at the real home folder.
    public static func standard() -> [any UsageReader] {
        [CodexReader(), GrokReader(), ClaudeCodeReader(), CursorReader(), AgyReader()]
    }

    /// A reader for one account: `root` overrides the tool's default data folder.
    public static func make(_ tool: ToolID, root: URL? = nil) -> any UsageReader {
        switch tool {
        case .codex: root.map { CodexReader(root: $0) } ?? CodexReader()
        case .grok: root.map { GrokReader(root: $0) } ?? GrokReader()
        case .claudeCode: root.map { ClaudeCodeReader(root: $0) } ?? ClaudeCodeReader()
        case .cursor: root.map { CursorReader(support: $0) } ?? CursorReader()
        case .agy: root.map { AgyReader(root: $0) } ?? AgyReader()
        }
    }

    /// Only tools that are installed; an absent tool is not shown as zero usage.
    public static func installed(_ readers: [any UsageReader]) -> [any UsageReader] {
        readers.filter(\.isInstalled)
    }
}
