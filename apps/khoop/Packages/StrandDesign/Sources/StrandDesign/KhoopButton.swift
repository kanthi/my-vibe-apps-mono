import SwiftUI

// MARK: - KhoopButton — the unified button system (Design Reset, 2026-06-22)
//
// One button, four kinds, no glow. Beauty comes from a crisp filled accent, honest
// surface fills, restrained spacing and a subtle press — never neon, bloom or a halo.
// Every colour is a token from `StrandPalette`; every dimension reads off `KhoopMetrics`.
//
// Two front doors:
//   • `KhoopButton("Save", kind: .primary) { … }`     — the convenience view.
//   • `Button("Save") { … }.buttonStyle(KhoopButtonStyle(.primary))`  — adopt on an
//     existing Button (e.g. a Menu/role button) without rewriting it.
//
// Labels are sentence-case (never ALL CAPS), single line, optical-centred with the
// optional leading icon as one unit, and degrade gracefully under Reduce Motion (the
// press scale drops; only the dim remains).

/// The four button roles. Colour + emphasis differ; geometry is identical across all four.
public enum KhoopButtonKind: Sendable {
    /// Filled accent (blue), white label — the one primary action on a screen.
    case primary
    /// Raised-surface fill, primary-text label, hairline edge — secondary actions.
    case secondary
    /// No fill, accent label — low-emphasis / inline actions.
    case tertiary
    /// Filled critical (red), white label — destructive / irreversible actions.
    case destructive
}

// MARK: - Shared geometry / resolved styling

/// Fixed geometry shared by the convenience view and the ButtonStyle so the two paths
/// are pixel-identical. The single source of truth for button shape.
public enum KhoopButtonMetrics {
    /// Standard control height (48) — also the source for the min hit target floor.
    public static let height: CGFloat = KhoopMetrics.controlHeight
    /// Corner radius (14) — softer than a card, not a pill.
    public static let cornerRadius: CGFloat = 14
    /// Horizontal label inset.
    public static let hPadding: CGFloat = 18
    /// Spacing between a leading icon and the label.
    public static let iconSpacing: CGFloat = 8
    /// Label tracking — a hair of openness on the semibold face.
    public static let tracking: CGFloat = 0.2
    /// Apple's minimum touch target. The button never reports a hit area below this.
    public static let minHitTarget: CGFloat = 44
    /// Pressed scale (spec: subtle 0.97). Reduce-Motion collapses this to 1 (dim only).
    public static let pressedScale: CGFloat = 0.97
    /// Pressed dim — a slight opacity drop, applied in BOTH motion modes.
    public static let pressedOpacity: Double = 0.82
    /// Disabled dim, shared so call sites don't invent their own.
    public static let disabledOpacity: Double = 0.4
}

/// Resolves a `KhoopButtonKind` to its concrete fill / label / border tokens. Internal
/// so the fill model stays in one place; both the style and the view read from here.
struct KhoopButtonAppearance {
    let fill: Color?          // nil = no fill (tertiary)
    let label: Color
    let border: Color?        // nil = no hairline edge
    let usesPanelSurface: Bool

    init(_ kind: KhoopButtonKind) {
        switch kind {
        case .primary:
            fill = StrandPalette.accent
            label = StrandPalette.goldDeepText   // designated crisp white for text on accent fills
            border = nil
            usesPanelSurface = false
        case .secondary:
            fill = nil
            label = StrandPalette.textPrimary
            border = nil
            usesPanelSurface = true
        case .tertiary:
            fill = nil
            label = StrandPalette.accent
            border = nil
            usesPanelSurface = false
        case .destructive:
            fill = StrandPalette.statusCritical
            label = StrandPalette.goldDeepText   // crisp white on the critical fill
            border = nil
            usesPanelSurface = false
        }
    }
}

// MARK: - The crisp background (no glow, ever)

/// The flat, glow-free button background: a filled (or unfilled) rounded rect with an
/// optional hairline edge. No shadow, no blur halo, no additive bloom — restraint only.
private struct KhoopButtonBackground: View {
    let appearance: KhoopButtonAppearance

    var body: some View {
        let shape = RoundedRectangle(cornerRadius: KhoopButtonMetrics.cornerRadius, style: .continuous)
        ZStack {
            if appearance.usesPanelSurface {
                KhoopPanelSurface(cornerRadius: KhoopButtonMetrics.cornerRadius)
            }
            if let fill = appearance.fill {
                shape.fill(fill)
            }
            if let border = appearance.border {
                shape.strokeBorder(border, lineWidth: 1)
            }
        }
    }
}

// MARK: - ButtonStyle (adopt on any existing Button)

/// Apply the Khoop button look to ANY `Button` — e.g. a role/`Menu` button you can't
/// replace with `KhoopButton`. Honours Reduce Motion: the press scale drops to a dim-only
/// state. Pixel-identical to `KhoopButton` since both share `KhoopButtonMetrics`/appearance.
public struct KhoopButtonStyle: ButtonStyle {
    private let kind: KhoopButtonKind
    private let fullWidth: Bool
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.isEnabled) private var isEnabled

    public init(_ kind: KhoopButtonKind = .primary, fullWidth: Bool = false) {
        self.kind = kind
        self.fullWidth = fullWidth
    }

    public func makeBody(configuration: Configuration) -> some View {
        let appearance = KhoopButtonAppearance(kind)
        let pressed = configuration.isPressed
        // Reduce Motion: no scale, dim only. Otherwise subtle scale + dim.
        let scale: CGFloat = (pressed && !reduceMotion) ? KhoopButtonMetrics.pressedScale : 1
        let opacity: Double = pressed ? KhoopButtonMetrics.pressedOpacity : 1

        configuration.label
            .labelStyle(.titleAndIcon)
            .font(StrandFont.headline.weight(.semibold))
            .tracking(KhoopButtonMetrics.tracking)
            .lineLimit(1)
            .minimumScaleFactor(0.9)
            .foregroundStyle(appearance.label)
            .frame(maxWidth: fullWidth ? .infinity : nil)
            .padding(.horizontal, KhoopButtonMetrics.hPadding)
            .frame(height: KhoopButtonMetrics.height)
            .frame(minHeight: KhoopButtonMetrics.minHitTarget)
            .contentShape(Rectangle())
            .background(KhoopButtonBackground(appearance: appearance))
            .clipShape(RoundedRectangle(cornerRadius: KhoopButtonMetrics.cornerRadius, style: .continuous))
            .opacity(isEnabled ? opacity : KhoopButtonMetrics.disabledOpacity)
            .scaleEffect(scale)
            .animation(reduceMotion ? nil : StrandMotion.interactive, value: pressed)
    }
}

// MARK: - KhoopButton (the convenience view)

/// The unified button. A title (sentence-case `LocalizedStringKey`), an optional leading
/// SF Symbol, a `KhoopButtonKind`, an optional `fullWidth`, and an action. Crisp, flat,
/// glow-free; subtle press; 44pt hit floor; Reduce-Motion aware.
///
/// ```swift
/// KhoopButton("Save changes", systemImage: "checkmark", kind: .primary, fullWidth: true) {
///     save()
/// }
/// ```
public struct KhoopButton: View {
    private let title: LocalizedStringKey
    private let systemImage: String?
    private let kind: KhoopButtonKind
    private let fullWidth: Bool
    private let action: () -> Void

    public init(
        _ title: LocalizedStringKey,
        systemImage: String? = nil,
        kind: KhoopButtonKind = .primary,
        fullWidth: Bool = false,
        action: @escaping () -> Void
    ) {
        self.title = title
        self.systemImage = systemImage
        self.kind = kind
        self.fullWidth = fullWidth
        self.action = action
    }

    public var body: some View {
        Button(action: action) {
            // The ButtonStyle owns the chrome (fill / colour / press / padding). The label here is
            // just the icon + word as one centred unit at the exact 8pt token spacing. When there's
            // no icon the HStack holds a single Text, so the word sits dead-centre with no phantom gap.
            HStack(spacing: KhoopButtonMetrics.iconSpacing) {
                if let systemImage {
                    Image(systemName: systemImage)
                        .imageScale(.medium)  // optically centres to the cap height of the label
                }
                Text(title)
            }
        }
        .buttonStyle(KhoopButtonStyle(kind, fullWidth: fullWidth))
    }
}

#if DEBUG
#Preview("KhoopButton") {
    ScrollView {
        VStack(spacing: KhoopMetrics.rowSpacing) {
            KhoopButton("Primary action", systemImage: "checkmark", kind: .primary) {}
            KhoopButton("Secondary action", systemImage: "square.and.arrow.up", kind: .secondary) {}
            KhoopButton("Tertiary action", kind: .tertiary) {}
            KhoopButton("Delete recording", systemImage: "trash", kind: .destructive) {}

            Divider().overlay(StrandPalette.hairline)

            KhoopButton("Full-width primary", systemImage: "bolt.fill", kind: .primary, fullWidth: true) {}
            KhoopButton("Full-width secondary", kind: .secondary, fullWidth: true) {}

            // Adopting the style on a vanilla Button.
            Button("Adopted via KhoopButtonStyle") {}
                .buttonStyle(KhoopButtonStyle(.secondary, fullWidth: true))

            KhoopButton("Disabled", kind: .primary) {}
                .disabled(true)
        }
        .screenPadding()
        .padding(.vertical, KhoopMetrics.space6)
    }
    .frame(width: 380, height: 560)
    .background(StrandPalette.surfaceBase)
    .preferredColorScheme(.dark)
}
#endif
