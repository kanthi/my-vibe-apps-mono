import Foundation
import MeterCore

// `meter-probe --fake-home <dir>` writes invented tool data for sandbox testing and exits.
if CommandLine.arguments.count == 3, CommandLine.arguments[1] == "--fake-home" {
    try FakeHome.build(at: URL(fileURLWithPath: CommandLine.arguments[2]))
    print("Fake home written to \(CommandLine.arguments[2])")
    exit(0)
}

// Read-only check of the readers against the real home folder. Prints counts and totals only:
// no project names, no text. Never writes anywhere except private temp copies it deletes.
let since = Date().addingTimeInterval(-30 * 86_400)
for reader in ReaderRegistry.standard() {
    let name = reader.tool.displayName
    guard reader.isInstalled else { print("\(name): not installed"); continue }
    do {
        let out = try reader.read(since: since)
        let inTok = out.sessions.compactMap(\.inputTokens).reduce(0, +)
        let outTok = out.sessions.compactMap(\.outputTokens).reduce(0, +)
        let turns = out.sessions.compactMap(\.turns).reduce(0, +)
        let models = Set(out.sessions.compactMap(\.model)).sorted()
        print("\(name): \(out.sessions.count) sessions (30d), input \(inTok), output \(outTok), turns \(turns), models \(models)")
        for q in out.quotas {
            print("  quota \(q.label): \(q.usedPercent)% resets \(q.resetsAt.map { "\($0)" } ?? "?") observed \(q.observedAt)")
        }
    } catch {
        print("\(name): read failed: \(error)")
    }
}
