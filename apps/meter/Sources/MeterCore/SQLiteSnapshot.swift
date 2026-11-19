import Foundation
import SQLite3

/// Opens a read-only connection to a private copy of a database and its -wal/-shm files, so a
/// reader can never interfere with the app that owns the live file (locks, checkpoints, -shm writes).
struct SQLiteSnapshot {
    let db: OpaquePointer
    private let dir: URL

    init(copying source: URL) throws {
        let fm = FileManager.default
        dir = fm.temporaryDirectory.appendingPathComponent("meter-snap-\(UUID().uuidString)")
        try fm.createDirectory(at: dir, withIntermediateDirectories: true)
        let name = source.lastPathComponent
        for suffix in ["", "-wal", "-shm"] {
            let from = URL(fileURLWithPath: source.path + suffix)
            if fm.fileExists(atPath: from.path) { try fm.copyItem(at: from, to: dir.appendingPathComponent(name + suffix)) }
        }
        var handle: OpaquePointer?
        guard sqlite3_open_v2(dir.appendingPathComponent(name).path, &handle, SQLITE_OPEN_READWRITE, nil) == SQLITE_OK, let h = handle else {
            sqlite3_close(handle); try? fm.removeItem(at: dir); throw CocoaError(.fileReadUnknown)
        }
        db = h
    }

    func close() {
        sqlite3_close(db)
        try? FileManager.default.removeItem(at: dir)
    }
}
