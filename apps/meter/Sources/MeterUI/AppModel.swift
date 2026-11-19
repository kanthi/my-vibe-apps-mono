import Foundation
import MeterCore
import Observation
import ServiceManagement

@MainActor @Observable
public final class AppModel {
    public var settings: AppSettings { didSet { if settings != oldValue { settingsChanged(from: oldValue) } } }
    public private(set) var accounts: [Account] = []
    public private(set) var dashboard: Dashboard
    public private(set) var isRefreshing = false
    public private(set) var lastRefresh: Date?
    public private(set) var launchAtLoginError: String?
    /// Set by the menu bar label, which exists for the app's whole life, so the Dock menu can reopen the window.
    public var openMainWindow: (() -> Void)?

    private var runs: [String: AccountRun] = [:]
    private let accountStore: AccountStore
    private let rollupStore: RollupStore
    private let defaults: KeyValueStore
    private let notifier: Notifier
    private let userNotifier: UserNotifier?
    private let engine = AlertEngine()
    private let calendar = Calendar.current
    private var loop: Task<Void, Never>?

    /// `supportDirectory` is injected so tests and previews never touch the real Application Support.
    public init(supportDirectory: URL, defaults: KeyValueStore, notifier: Notifier? = nil) {
        accountStore = AccountStore(url: supportDirectory.appendingPathComponent("accounts.json"))
        rollupStore = RollupStore(url: supportDirectory.appendingPathComponent("rollup.json"))
        self.defaults = defaults
        let real = UserNotifier()
        userNotifier = notifier == nil ? real : nil
        self.notifier = notifier ?? real
        settings = SettingsStorage.load(defaults)
        dashboard = Dashboard.build(accounts: [], runs: [:], days: [], calendar: .current, now: .now)
        engine.thresholds = [settings.thresholdLow, settings.thresholdHigh]
        accounts = accountStore.effective()
    }

    /// True when running with METER_SANDBOX: no UserDefaults, no notifications, no login item.
    public private(set) var isSandbox = false

    public static func live(environment: [String: String] = ProcessInfo.processInfo.environment) -> AppModel {
        if let box = environment["METER_SANDBOX"], !box.isEmpty {
            let root = URL(fileURLWithPath: box)
            let model = AppModel(supportDirectory: root.appendingPathComponent("Meter-support"),
                                 defaults: FileStore(url: root.appendingPathComponent("settings.json")),
                                 notifier: RecordingNotifier())
            model.isSandbox = true
            return model
        }
        let dir = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0].appendingPathComponent("Meter")
        return AppModel(supportDirectory: dir, defaults: UserDefaults.standard)
    }

    // MARK: refresh

    public func start() {
        guard loop == nil else { return }
        loop = Task { [weak self] in
            while !Task.isCancelled {
                await self?.refresh()
                let secs = self?.settings.refreshSeconds ?? 60
                try? await Task.sleep(for: .seconds(max(15, secs)))
            }
        }
    }

    public func refresh() async {
        guard !isRefreshing else { return }
        isRefreshing = true
        defer { isRefreshing = false }
        let now = Date()
        let startOfWindow = calendar.startOfDay(for: now.addingTimeInterval(-35 * 86_400))
        runs = await Refresher.run(accounts: accounts, previous: runs, now: now, since: startOfWindow)
        // Hidden accounts keep accumulating history; only the display and alerts skip them.
        let sessions = runs.values.flatMap(\.output.sessions)
        let fromDay = Rollup.dayKey(startOfWindow, calendar: calendar)
        let fresh = Rollup.days(from: sessions, calendar: calendar).filter { $0.day >= fromDay }
        let tools = Set(accounts.map(\.tool))
        let days = (try? rollupStore.merge(fresh, tools: tools, fromDay: fromDay)) ?? rollupStore.load()
        lastRefresh = now
        rebuild(days: days, now: now)
        deliverAlerts(now: now)
    }

    private func rebuild(days: [DayUsage]? = nil, now: Date = .now) {
        dashboard = Dashboard.build(accounts: accounts, runs: runs, days: days ?? rollupStore.load(), calendar: calendar, now: now)
    }

    private func deliverAlerts(now: Date) {
        for alert in engine.evaluate(dashboard.visibleQuotas) where settings.alertPolicy.allows(alert, at: now, calendar: calendar) {
            notifier.deliver(alert)
        }
    }

    // MARK: accounts

    public func addAccount(tool: ToolID, label: String, dataRoot: String?) {
        let account = Account(tool: tool, label: label, dataRoot: dataRoot)
        _ = try? accountStore.add(account)
        accounts = accountStore.effective()
        Task { await refresh() }
    }

    public func setHidden(_ hidden: Bool, for account: Account) {
        _ = try? accountStore.setHidden(hidden, tool: account.tool, id: account.id)
        accounts = accountStore.effective()
        rebuild()
    }

    public func remove(_ account: Account) {
        guard account.id != "default" else { return }
        _ = try? accountStore.remove(tool: account.tool, id: account.id)
        runs[Dashboard.key(account)] = nil
        accounts = accountStore.effective()
        rebuild()
    }

    // MARK: settings

    private func settingsChanged(from old: AppSettings) {
        SettingsStorage.save(settings, defaults)
        engine.thresholds = [settings.thresholdLow, settings.thresholdHigh]
        if settings.launchAtLogin != old.launchAtLogin { applyLaunchAtLogin(settings.launchAtLogin) }
        // Permission is asked only when the first alert is switched on.
        if old.alertPolicy.enabledKinds.isEmpty, !settings.alertPolicy.enabledKinds.isEmpty, let n = userNotifier {
            Task { _ = await n.requestAuthorization() }
        }
    }

    private func applyLaunchAtLogin(_ on: Bool) {
        guard Bundle.main.bundleIdentifier != nil, !isSandbox else { return }
        do {
            if on { try SMAppService.mainApp.register() } else { try SMAppService.mainApp.unregister() }
            launchAtLoginError = nil
        } catch {
            launchAtLoginError = "Could not change launch at login."
            settings.launchAtLogin = !on
        }
    }

    // MARK: test and preview support

    public func load(runs: [String: AccountRun], accounts: [Account], days: [DayUsage], now: Date) {
        self.runs = runs; self.accounts = accounts; lastRefresh = now
        dashboard = Dashboard.build(accounts: accounts, runs: runs, days: days, calendar: calendar, now: now)
    }
}
