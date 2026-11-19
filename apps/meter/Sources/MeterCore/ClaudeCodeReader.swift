import Foundation

/// Reads Claude Code session logs under `projects/`. Only usage numbers, model, cwd,
/// session id and timestamps are taken from each line; message text is never kept.
/// Claude Code writes no quota to disk, so this reader reports no quotas.
public struct ClaudeCodeReader: UsageReader {
    public let tool = ToolID.claudeCode
    let root: URL

    public init(root: URL = FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent(".claude")) {
        self.root = root
    }

    public var isInstalled: Bool { FileManager.default.fileExists(atPath: root.appendingPathComponent("projects").path) }

    struct Call { var session: String; var model: String?; var cwd: String?; var at: Date
                  var input: Int; var cacheRead: Int; var output: Int; var sidechain: Bool }

    public func read(since: Date) throws -> ReaderOutput {
        let base = root.appendingPathComponent("projects")
        guard let walker = FileManager.default.enumerator(at: base, includingPropertiesForKeys: nil) else { return ReaderOutput() }
        // A streamed response can be logged several times with the same requestId; keep the fullest one.
        var byRequest: [String: Call] = [:]
        var anonymous: [Call] = []
        for case let file as URL in walker where file.pathExtension == "jsonl" {
            for obj in Parse.jsonLines(in: file, containing: ["\"usage\""]) {
                guard obj["type"] as? String == "assistant",
                      let msg = obj["message"] as? [String: Any], let usage = msg["usage"] as? [String: Any],
                      let at = (obj["timestamp"] as? String).flatMap(Parse.date), at >= since,
                      let session = obj["sessionId"] as? String else { continue }
                let call = Call(
                    session: session, model: msg["model"] as? String, cwd: obj["cwd"] as? String, at: at,
                    input: (Parse.int(usage["input_tokens"]) ?? 0) + (Parse.int(usage["cache_creation_input_tokens"]) ?? 0),
                    cacheRead: Parse.int(usage["cache_read_input_tokens"]) ?? 0,
                    output: Parse.int(usage["output_tokens"]) ?? 0,
                    sidechain: obj["isSidechain"] as? Bool ?? file.path.contains("/subagents/"))
                if let rid = obj["requestId"] as? String {
                    if let old = byRequest[rid], old.output > call.output { continue }
                    byRequest[rid] = call
                } else { anonymous.append(call) }
            }
        }
        var bySession: [String: [Call]] = [:]
        for c in Array(byRequest.values) + anonymous { bySession[c.session, default: []].append(c) }
        let sessions = bySession.map { id, calls -> Session in
            let sorted = calls.sorted { $0.at < $1.at }
            // Subagent work is counted once, under the parent session that spawned it.
            return Session(tool: .claudeCode, id: id, model: sorted.last?.model, project: sorted.last?.cwd,
                           start: sorted.first!.at, end: sorted.last!.at,
                           inputTokens: calls.reduce(0) { $0 + $1.input },
                           cachedTokens: calls.reduce(0) { $0 + $1.cacheRead },
                           outputTokens: calls.reduce(0) { $0 + $1.output },
                           isSubagent: false, turns: calls.count)
        }.sorted { $0.start < $1.start }
        return ReaderOutput(sessions: sessions, quotas: [])
    }
}
