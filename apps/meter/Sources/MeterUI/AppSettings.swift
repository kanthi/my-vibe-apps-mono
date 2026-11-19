import Foundation
import MeterCore

public struct AppSettings: Codable, Equatable, Sendable {
    public var showPercentInMenuBar = true
    public var refreshSeconds = 60
    public var launchAtLogin = false                // off until the user turns it on
    public var thresholdLow = 80.0
    public var thresholdHigh = 95.0
    public var alertPolicy = AlertPolicy()
    public init() {}
}

/// Where settings live. The app uses UserDefaults; tests use memory so they never write to the host.
public protocol KeyValueStore: AnyObject {
    func data(forKey key: String) -> Data?
    func set(_ value: Any?, forKey key: String)
}
extension UserDefaults: KeyValueStore {}

public final class MemoryStore: KeyValueStore, @unchecked Sendable {
    private var items: [String: Data] = [:]
    public init() {}
    public func data(forKey key: String) -> Data? { items[key] }
    public func set(_ value: Any?, forKey key: String) { items[key] = value as? Data }
}

/// Settings in a JSON file inside a folder you choose. Used by sandbox mode so nothing goes to UserDefaults.
public final class FileStore: KeyValueStore, @unchecked Sendable {
    private let url: URL
    private var items: [String: Data]
    public init(url: URL) {
        self.url = url
        items = (try? Data(contentsOf: url)).flatMap { try? JSONDecoder().decode([String: Data].self, from: $0) } ?? [:]
    }
    public func data(forKey key: String) -> Data? { items[key] }
    public func set(_ value: Any?, forKey key: String) {
        items[key] = value as? Data
        try? FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        try? JSONEncoder().encode(items).write(to: url, options: .atomic)
    }
}

public enum SettingsStorage {
    static let key = "meter.settings"
    public static func load(_ defaults: KeyValueStore) -> AppSettings {
        defaults.data(forKey: key).flatMap { try? JSONDecoder().decode(AppSettings.self, from: $0) } ?? AppSettings()
    }
    public static func save(_ s: AppSettings, _ defaults: KeyValueStore) {
        if let d = try? JSONEncoder().encode(s) { defaults.set(d, forKey: key) }
    }
}
