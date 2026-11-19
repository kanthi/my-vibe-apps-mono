import Charts
import MeterCore
import SwiftUI

public enum SidebarItem: String, CaseIterable, Identifiable {
    case overview = "Overview", usage = "Usage", accounts = "Accounts"
    public var id: String { rawValue }
    var symbol: String {
        switch self { case .overview: "gauge.with.dots.needle.50percent"; case .usage: "chart.bar"; case .accounts: "person.2" }
    }
}

public struct MainView: View {
    @Bindable var model: AppModel
    @State private var selection: SidebarItem? = .overview
    public init(model: AppModel) { self.model = model }

    public var body: some View {
        NavigationSplitView {
            List(SidebarItem.allCases, selection: $selection) { item in
                Label(item.rawValue, systemImage: item.symbol)
            }
            .navigationSplitViewColumnWidth(min: 150, ideal: 170)
        } detail: {
            Group {
                switch selection ?? .overview {
                case .overview: OverviewView(model: model)
                case .usage: UsageView(model: model)
                case .accounts: AccountsView(model: model)
                }
            }
            .navigationTitle(selection?.rawValue ?? "Overview")
        }
        .toolbar {
            ToolbarItem {
                Button { Task { await model.refresh() } } label: { Label("Refresh", systemImage: "arrow.clockwise") }
                    .disabled(model.isRefreshing)
                    .help(model.lastRefresh.map { "Updated \($0.formatted(date: .omitted, time: .shortened))" } ?? "Not refreshed yet")
            }
        }
        .frame(minWidth: 760, minHeight: 480)
    }
}

struct OverviewView: View {
    @Bindable var model: AppModel

    var body: some View {
        let now = model.lastRefresh ?? .now
        let shown = model.dashboard.accounts.filter { !$0.account.isHidden && $0.installed }
        let missing = model.dashboard.accounts.filter { !$0.installed && !$0.account.isHidden }
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if !model.dashboard.notes.isEmpty {
                    VStack(alignment: .leading, spacing: 6) {
                        ForEach(Array(model.dashboard.notes.enumerated()), id: \.offset) { _, n in
                            Label(n.text, systemImage: "lightbulb").font(.callout)
                        }
                    }
                    .padding(12).frame(maxWidth: .infinity, alignment: .leading)
                    .background(.background.secondary, in: RoundedRectangle(cornerRadius: 10))
                }
                LazyVGrid(columns: [GridItem(.adaptive(minimum: 320), spacing: 14, alignment: .top)], spacing: 14) {
                    ForEach(shown) { ToolCard(status: $0, now: now) }
                }
                if shown.isEmpty {
                    ContentUnavailableView("No tools found", systemImage: "questionmark.folder",
                                           description: Text("Meter looks for Codex, Grok, Claude Code, Cursor and agy on this Mac."))
                }
                if !missing.isEmpty {
                    Text("Not installed: " + missing.map { $0.account.tool.displayName }.joined(separator: ", "))
                        .font(.caption).foregroundStyle(.secondary)
                }
            }
            .padding(20)
        }
    }
}

struct UsageView: View {
    enum Metric: String, CaseIterable { case tokens = "Tokens", turns = "Turns" }
    @Bindable var model: AppModel
    @State private var metric: Metric = .tokens
    @State private var range = 14

    private struct Point: Identifiable { var id: String { day + tool }; var day: String; var tool: String; var value: Int
        var date: Date { (try? Date("\(day)T12:00:00Z", strategy: .iso8601)) ?? .now } }

    private var points: [Point] {
        let cutoff = Rollup.dayKey(Date().addingTimeInterval(-Double(range) * 86_400), calendar: .current)
        var map: [String: Point] = [:]
        for d in model.dashboard.days where d.day >= cutoff {
            let v = metric == .tokens ? d.tokens : d.turns
            guard v > 0 else { continue }
            let k = d.day + d.tool.displayName
            map[k, default: Point(day: d.day, tool: d.tool.displayName, value: 0)].value += v
        }
        return map.values.sorted { $0.day < $1.day }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Picker("Metric", selection: $metric) { ForEach(Metric.allCases, id: \.self) { Text($0.rawValue) } }
                    .pickerStyle(.segmented).frame(width: 200).labelsHidden()
                Picker("Range", selection: $range) { Text("7 days").tag(7); Text("14 days").tag(14); Text("30 days").tag(30) }
                    .frame(width: 120).labelsHidden()
                Spacer()
            }
            if points.isEmpty {
                ContentUnavailableView("No usage yet", systemImage: "chart.bar",
                                       description: Text("Usage appears here as Meter reads your tools. History is kept after a tool cleans up its logs."))
            } else {
                Chart(points) { p in
                    BarMark(x: .value("Day", p.date, unit: .day), y: .value(metric.rawValue, p.value))
                        .foregroundStyle(by: .value("Tool", p.tool))
                }
                .chartXAxis { AxisMarks(values: .stride(by: .day, count: range > 14 ? 5 : 2)) { _ in AxisGridLine(); AxisValueLabel(format: .dateTime.month(.abbreviated).day()) } }
                .chartYAxis { AxisMarks { v in AxisGridLine(); AxisValueLabel { if let n = v.as(Int.self) { Text(metric == .tokens ? Format.tokens(n) : "\(n)") } } } }
                .accessibilityLabel("\(metric.rawValue) per day by tool")
            }
        }
        .padding(20)
    }
}

struct AccountsView: View {
    @Bindable var model: AppModel
    @State private var adding = false

    var body: some View {
        List {
            ForEach(ToolID.allCases, id: \.self) { tool in
                let rows = model.accounts.filter { $0.tool == tool }
                Section(tool.displayName) {
                    ForEach(rows) { a in
                        HStack {
                            VStack(alignment: .leading) {
                                Text(a.label)
                                if let r = a.dataRoot { Text(r).font(.caption).foregroundStyle(.secondary).lineLimit(1) }
                            }
                            Spacer()
                            Toggle("Show", isOn: Binding(get: { !a.isHidden }, set: { model.setHidden(!$0, for: a) }))
                                .toggleStyle(.switch).labelsHidden()
                                .accessibilityLabel("Show \(tool.displayName) \(a.label)")
                            if a.id != "default" {
                                Button(role: .destructive) { model.remove(a) } label: { Image(systemName: "trash") }
                                    .buttonStyle(.borderless).accessibilityLabel("Remove \(a.label)")
                            }
                        }
                    }
                }
            }
        }
        .toolbar { ToolbarItem { Button { adding = true } label: { Label("Add Account", systemImage: "plus") } } }
        .sheet(isPresented: $adding) { AddAccountSheet(model: model) }
        .safeAreaInset(edge: .bottom) {
            Text("Hidden accounts leave the menu, notes and alerts. Their history is kept.")
                .font(.caption).foregroundStyle(.secondary).padding(8)
        }
    }
}

struct AddAccountSheet: View {
    @Bindable var model: AppModel
    @Environment(\.dismiss) private var dismiss
    @State private var tool: ToolID = .codex
    @State private var label = ""
    @State private var root = ""

    var body: some View {
        Form {
            Picker("Tool", selection: $tool) { ForEach(ToolID.allCases, id: \.self) { Text($0.displayName).tag($0) } }
            TextField("Name", text: $label, prompt: Text("Work"))
            HStack {
                TextField("Data folder", text: $root, prompt: Text("Where this account keeps its records"))
                Button("Choose…") {
                    let p = NSOpenPanel(); p.canChooseDirectories = true; p.canChooseFiles = false
                    p.showsHiddenFiles = true
                    if p.runModal() == .OK, let u = p.url { root = u.path }
                }
            }
            Text("Meter reads only usage numbers from this folder. It never reads credentials or messages.")
                .font(.caption).foregroundStyle(.secondary)
        }
        .formStyle(.grouped)
        .frame(width: 440)
        .padding(.bottom, 8)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) { Button("Cancel") { dismiss() } }
            ToolbarItem(placement: .confirmationAction) {
                Button("Add") {
                    model.addAccount(tool: tool, label: label.trimmingCharacters(in: .whitespaces), dataRoot: root.isEmpty ? nil : root)
                    dismiss()
                }
                .disabled(label.trimmingCharacters(in: .whitespaces).isEmpty || root.isEmpty)
            }
        }
    }
}
