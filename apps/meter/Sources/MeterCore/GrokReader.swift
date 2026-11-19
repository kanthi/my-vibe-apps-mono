import Foundation

/// Reads Grok's local log: weekly credit percent and per-inference token counts.
/// Never reads `auth.json` or chat history.
public struct GrokReader: UsageReader {
    public let tool = ToolID.grok
    let root: URL

    public init(root: URL = FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent(".grok")) {
        self.root = root
    }

    public var isInstalled: Bool { FileManager.default.fileExists(atPath: root.path) }

    public func read(since: Date) throws -> ReaderOutput {
        let log = root.appendingPathComponent("logs/unified.jsonl")
        let lines = Parse.jsonLines(in: log, containing: ["billing: fetched credits config", "shell.turn.inference_done"])
        return ReaderOutput(sessions: sessions(from: lines, since: since), quotas: quota(from: lines))
    }

    func quota(from lines: [[String: Any]]) -> [Quota] {
        var latest: (Date, [String: Any])?
        for l in lines where l["msg"] as? String == "billing: fetched credits config" {
            guard let ts = (l["ts"] as? String).flatMap(Parse.date),
                  let cfg = (l["ctx"] as? [String: Any])?["config"] as? [String: Any] else { continue }
            if latest == nil || ts >= latest!.0 { latest = (ts, cfg) }
        }
        guard let (observed, cfg) = latest, let used = (cfg["creditUsagePercent"] as? NSNumber)?.doubleValue else { return [] }
        let period = cfg["currentPeriod"] as? [String: Any]
        let start = (period?["start"] as? String).flatMap(Parse.date)
        let end = (period?["end"] as? String).flatMap(Parse.date)
        let minutes = start.flatMap { s in end.map { Int($0.timeIntervalSince(s) / 60) } }
        let label = (period?["type"] as? String)?.contains("WEEKLY") == true ? "Weekly" : Parse.windowLabel(minutes: minutes)
        return [Quota(tool: .grok, label: label, usedPercent: used, windowMinutes: minutes, resetsAt: end, observedAt: observed)]
    }

    func sessions(from lines: [[String: Any]], since: Date) -> [Session] {
        struct Acc { var start: Date; var end: Date; var input = 0; var cached = 0; var output = 0 }
        var bySid: [String: Acc] = [:]
        for l in lines where l["msg"] as? String == "shell.turn.inference_done" {
            guard let sid = l["sid"] as? String, let ts = (l["ts"] as? String).flatMap(Parse.date), ts >= since,
                  let ctx = l["ctx"] as? [String: Any] else { continue }
            var a = bySid[sid] ?? Acc(start: ts, end: ts)
            a.start = min(a.start, ts); a.end = max(a.end, ts)
            a.input += Parse.int(ctx["prompt_tokens"]) ?? 0
            a.cached += Parse.int(ctx["cached_prompt_tokens"]) ?? 0
            a.output += (Parse.int(ctx["completion_tokens"]) ?? 0) + (Parse.int(ctx["reasoning_tokens"]) ?? 0)
            bySid[sid] = a
        }
        let dirs = sessionDirectories()
        return bySid.map { sid, a in
            Session(tool: .grok, id: sid, model: dirs[sid].flatMap(model(in:)), project: dirs[sid].map(project(of:)),
                    start: a.start, end: a.end, inputTokens: a.input, cachedTokens: a.cached, outputTokens: a.output)
        }.sorted { $0.start < $1.start }
    }

    /// session id -> its folder under sessions/<url-encoded cwd>/<id>
    func sessionDirectories() -> [String: URL] {
        let base = root.appendingPathComponent("sessions")
        let fm = FileManager.default
        var map: [String: URL] = [:]
        for proj in (try? fm.contentsOfDirectory(at: base, includingPropertiesForKeys: nil)) ?? [] {
            for s in (try? fm.contentsOfDirectory(at: proj, includingPropertiesForKeys: nil)) ?? [] { map[s.lastPathComponent] = s }
        }
        return map
    }

    func project(of dir: URL) -> String { dir.deletingLastPathComponent().lastPathComponent.removingPercentEncoding ?? "" }

    func model(in dir: URL) -> String? {
        let events = Parse.jsonLines(in: dir.appendingPathComponent("events.jsonl"), containing: ["turn_started"])
        guard let first = events.first(where: { $0["type"] as? String == "turn_started" }), let m = first["model_id"] else { return nil }
        return "\(m)"
    }
}
