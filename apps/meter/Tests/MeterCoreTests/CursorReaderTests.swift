import Foundation
import Testing
@testable import MeterCore

@Suite struct CursorReaderTests {
    @Test func readsTurnsAndModelAndSkipsDrafts() throws {
        let root = try Fixtures.tempDir()
        let used = #"{"createdAt":1791092136848,"lastUpdatedAt":1791092166622,"text":"SECRET","modelConfig":{"modelName":"grok-4.6"},"fullConversationHeadersOnly":[{},{},{}]}"#
        let draft = #"{"createdAt":1791092136848,"modelConfig":{"modelName":"grok-4.6"},"fullConversationHeadersOnly":[]}"#
        let noDate = #"{"fullConversationHeadersOnly":[{}]}"#
        try Fixtures.cursorDB(at: root.appendingPathComponent("User/globalStorage/state.vscdb"), values: [
            ("composerData:abc", used), ("composerData:empty-state-draft", draft), ("composerData:nodate", noDate),
            ("bubbleId:abc:1", #"{"text":"SECRET"}"#)
        ])
        let out = try CursorReader(support: root).read(since: .distantPast)
        #expect(out.sessions.count == 1)
        let s = out.sessions[0]
        #expect(s.id == "abc" && s.model == "grok-4.6" && s.turns == 3)
        #expect(s.inputTokens == nil && s.outputTokens == nil)
        #expect(s.end == Date(timeIntervalSince1970: 1_791_092_166.622))
        #expect(out.quotas.isEmpty)
    }

    @Test func notInstalledWhenNoDatabase() throws {
        let r = CursorReader(support: try Fixtures.tempDir())
        #expect(!r.isInstalled)
        #expect(try r.read(since: .distantPast) == ReaderOutput())
    }
}
