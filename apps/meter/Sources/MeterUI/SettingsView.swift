import MeterCore
import SwiftUI

public struct SettingsView: View {
    @Bindable var model: AppModel
    public init(model: AppModel) { self.model = model }

    public var body: some View {
        TabView {
            GeneralSettings(model: model).tabItem { Label("General", systemImage: "gearshape") }
            AlertSettings(model: model).tabItem { Label("Alerts", systemImage: "bell") }
        }
        .frame(width: 480, height: 560)
    }
}

struct GeneralSettings: View {
    @Bindable var model: AppModel
    var body: some View {
        Form {
            Toggle("Show percent in the menu bar", isOn: $model.settings.showPercentInMenuBar)
            Picker("Refresh every", selection: $model.settings.refreshSeconds) {
                Text("30 seconds").tag(30); Text("1 minute").tag(60); Text("5 minutes").tag(300)
            }
            Toggle("Open Meter at login", isOn: $model.settings.launchAtLogin)
            if let e = model.launchAtLoginError { Text(e).font(.caption).foregroundStyle(.red) }
            Text("Meter keeps running in the menu bar when its window is closed. Quit it from the menu bar or the Dock.")
                .font(.caption).foregroundStyle(.secondary)
        }
        .formStyle(.grouped)
    }
}

struct AlertSettings: View {
    @Bindable var model: AppModel

    private func kind(_ k: MeterCore.Alert.Kind, _ title: String, _ detail: String) -> some View {
        Toggle(isOn: Binding(
            get: { model.settings.alertPolicy.enabledKinds.contains(k.rawValue) },
            set: { on in
                if on { model.settings.alertPolicy.enabledKinds.insert(k.rawValue) }
                else { model.settings.alertPolicy.enabledKinds.remove(k.rawValue) }
            })) {
            VStack(alignment: .leading) { Text(title); Text(detail).font(.caption).foregroundStyle(.secondary) }
        }
    }

    private func minuteBinding(_ path: WritableKeyPath<AlertPolicy, Int?>, default def: Int) -> Binding<Date> {
        Binding(
            get: {
                let m = model.settings.alertPolicy[keyPath: path] ?? def
                return Calendar.current.date(bySettingHour: m / 60, minute: m % 60, second: 0, of: .now) ?? .now
            },
            set: { d in
                let c = Calendar.current.dateComponents([.hour, .minute], from: d)
                model.settings.alertPolicy[keyPath: path] = (c.hour ?? 0) * 60 + (c.minute ?? 0)
            })
    }

    var body: some View {
        Form {
            Section("Notify me when") {
                kind(.threshold, "A limit is filling up", "At \(Int(model.settings.thresholdLow))% and \(Int(model.settings.thresholdHigh))% of a window.")
                kind(.exhausted, "A limit is used up", "The window reaches 100%.")
                kind(.restored, "A limit has reset", "Room is available again after being used up.")
                kind(.pace, "I may run out early", "At the current pace a limit runs out before it resets.")
            }
            Section("Thresholds") {
                Stepper("First: \(Int(model.settings.thresholdLow))%", value: $model.settings.thresholdLow, in: 50...90, step: 5)
                Stepper("Second: \(Int(model.settings.thresholdHigh))%", value: $model.settings.thresholdHigh, in: 55...99, step: 1)
            }
            Section("Quiet hours") {
                Toggle("Hold alerts during quiet hours", isOn: Binding(
                    get: { model.settings.alertPolicy.quietStartMinute != nil },
                    set: {
                        model.settings.alertPolicy.quietStartMinute = $0 ? 22 * 60 : nil
                        model.settings.alertPolicy.quietEndMinute = $0 ? 8 * 60 : nil
                    }))
                if model.settings.alertPolicy.quietStartMinute != nil {
                    DatePicker("From", selection: minuteBinding(\.quietStartMinute, default: 22 * 60), displayedComponents: .hourAndMinute)
                    DatePicker("Until", selection: minuteBinding(\.quietEndMinute, default: 8 * 60), displayedComponents: .hourAndMinute)
                }
                Menu("Snooze") {
                    Button("For 1 hour") { model.settings.alertPolicy.snoozeUntil = .now.addingTimeInterval(3600) }
                    Button("For 8 hours") { model.settings.alertPolicy.snoozeUntil = .now.addingTimeInterval(8 * 3600) }
                    Button("Off") { model.settings.alertPolicy.snoozeUntil = nil }
                }
                if let s = model.settings.alertPolicy.snoozeUntil, s > .now {
                    Text("Snoozed until \(s.formatted(date: .omitted, time: .shortened))").font(.caption).foregroundStyle(.secondary)
                }
            }
            Text("All alerts are off until you turn one on. macOS asks for notification permission at that moment.")
                .font(.caption).foregroundStyle(.secondary)
        }
        .formStyle(.grouped)
    }
}
