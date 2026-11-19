import MeterCore
import SwiftUI

extension PaceVerdict {
    /// Words and a symbol always accompany the color, so meaning never rests on color alone.
    func presentation(now: Date) -> (text: String, symbol: String, color: Color)? {
        switch self {
        case .notEnoughData: nil
        case .onCourse(let p): ("On course, about \(Int(p))% by reset", "checkmark.circle", .green)
        case .tight(let p): ("Tight, about \(Int(p))% by reset", "exclamationmark.triangle", .orange)
        case .willRunOut(let at): ("Runs out \(Format.until(at, now: now))", "flame", .red)
        }
    }
}

/// A plain capsule bar, so its color does not depend on the window being active.
struct UsageBar: View {
    let percent: Double
    let color: Color
    var body: some View {
        GeometryReader { g in
            ZStack(alignment: .leading) {
                Capsule().fill(.quaternary)
                Capsule().fill(color).frame(width: max(6, g.size.width * min(max(percent, 0), 100) / 100))
            }
        }
        .frame(height: 8)
        .accessibilityHidden(true)
    }
}

struct QuotaRow: View {
    let quota: Quota
    let now: Date

    var body: some View {
        let pace = Insights.pace(quota).presentation(now: now)
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(quota.label).font(.subheadline.weight(.medium))
                Spacer()
                Text("\(Int(quota.usedPercent.rounded()))% used").font(.subheadline.monospacedDigit())
            }
            UsageBar(percent: quota.usedPercent, color: pace?.color ?? .accentColor)
            HStack(spacing: 6) {
                if let reset = quota.resetsAt {
                    Text("Resets \(Format.until(reset, now: now))").foregroundStyle(.secondary)
                }
                Spacer()
                if let pace { Label(pace.text, systemImage: pace.symbol).foregroundStyle(pace.color) }
            }
            .font(.caption)
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(quota.tool.displayName) \(quota.label), \(Int(quota.usedPercent.rounded())) percent used")
    }
}

struct ToolCard: View {
    let status: AccountStatus
    let now: Date

    var title: String {
        status.account.id == "default" ? status.account.tool.displayName : "\(status.account.tool.displayName) · \(status.account.label)"
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Text(title).font(.headline)
                Spacer()
                if status.isStale { Label("Outdated", systemImage: "clock.badge.exclamationmark").font(.caption).foregroundStyle(.orange) }
                if status.error != nil { Image(systemName: "exclamationmark.triangle").foregroundStyle(.orange).help(status.error ?? "") }
            }
            if status.quotas.isEmpty {
                Text("No limit is recorded on this Mac for \(status.account.tool.displayName).")
                    .font(.caption).foregroundStyle(.secondary)
            }
            ForEach(Array(status.quotas.enumerated()), id: \.offset) { _, q in QuotaRow(quota: q, now: now) }
            if !status.endedWindows.isEmpty && status.quotas.isEmpty {
                Text("Last recorded window has ended; waiting for new data.").font(.caption).foregroundStyle(.secondary)
            }
            Divider()
            Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 4) {
                if status.tokensToday > 0 {
                    GridRow { Text("Tokens today").foregroundStyle(.secondary); Text(Format.tokens(status.tokensToday)).monospacedDigit() }
                }
                if status.turnsToday > 0 {
                    GridRow { Text("Turns today").foregroundStyle(.secondary); Text("\(status.turnsToday)").monospacedDigit() }
                }
                if status.tokensToday == 0 && status.turnsToday == 0 {
                    GridRow { Text("Today").foregroundStyle(.secondary); Text("No activity") }
                }
                if let m = status.topModel { GridRow { Text("Model").foregroundStyle(.secondary); Text(m) } }
                if let p = status.topProject { GridRow { Text("Busiest project").foregroundStyle(.secondary); Text(Format.baseName(p)).lineLimit(1) } }
            }
            .font(.caption)
        }
        .padding(14)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(.background.secondary, in: RoundedRectangle(cornerRadius: 10))
        .accessibilityElement(children: .contain)
    }
}
