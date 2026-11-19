import SwiftUI

/// Plain RGB color that can be mixed, so metal facets can be shaded by angle.
public struct RGB: Sendable, Equatable {
    public var r: Double
    public var g: Double
    public var b: Double

    public init(_ r: Double, _ g: Double, _ b: Double) {
        self.r = r
        self.g = g
        self.b = b
    }

    public init(hex: UInt32) {
        self.init(
            Double((hex >> 16) & 0xFF) / 255.0,
            Double((hex >> 8) & 0xFF) / 255.0,
            Double(hex & 0xFF) / 255.0
        )
    }

    public func color(_ opacity: Double = 1) -> Color {
        Color(.sRGB, red: r, green: g, blue: b, opacity: opacity)
    }

    public static func mix(_ a: RGB, _ b: RGB, _ t: Double) -> RGB {
        let t = min(max(t, 0), 1)
        return RGB(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t)
    }

    public func scaled(_ k: Double) -> RGB {
        RGB(min(r * k, 1), min(g * k, 1), min(b * k, 1))
    }
}

/// Colors and layout for one full-screen, dark, 3D-lit dial.
public struct RealisticWatchTheme: Sendable, Equatable {
    /// The dial's layout and complications.
    public enum Layout: Sendable, Equatable {
        /// Original Bauhaus layout: pill batons, upright 1–12, outer 05–55 ring.
        case bauhaus
        /// Dress: applied polished batons, double baton at 12, date.
        case dress
        /// Diver: triangle at 12, batons at 6 and 9, lume dots, date.
        case diver
        /// Pilot: large printed numerals, triangle with two dots at 12.
        case pilot
        /// Chronograph: three recessed sub-dials, tachymeter, fine 1/5 s track.
        case chronograph
        /// Moonphase: guilloché, Roman numerals, railroad track, moon window at 6.
        case lunar
        /// GMT: two-tone 24-hour ring, arrowhead markers, UTC hand, date.
        case gmt
    }

    public enum Surface: Sendable, Equatable {
        case sunburst
        case matte
        case guilloche
    }

    public enum HandStyle: Sendable, Equatable {
        /// Faceted sword hands with a lume insert.
        case sword
        /// Tapered pencil hands with a lume groove (Bauhaus).
        case pencil
        /// Straight stick hands with a lume stripe (chronograph).
        case stick
        /// Thin hands with an open ring near the tip (moonphase dress).
        case breguet
    }

    public var name: String
    public var layout: Layout
    public var surface: Surface
    public var hands: HandStyle

    // Dial (fills the whole screen)
    public var dialCenter: RGB
    public var dialEdge: RGB
    /// Minute track, scales and small printing.
    public var print: RGB
    /// Accent (seconds tip, GMT hand, depth rating).
    public var accent: RGB
    /// Large printed numerals.
    public var numeral: RGB
    /// Sub-dial color, 24-hour day half, or moon-window sky.
    public var secondary: RGB
    /// 24-hour night half, or the moon.
    public var tertiary: RGB

    // Applied markers and lume
    public var indexLight: RGB
    public var indexDark: RGB
    public var lume: RGB
    public var lumeGlow: Double

    // Hands
    public var handLight: RGB
    public var handDark: RGB
    public var second: RGB
    public var secondTip: RGB

    public var showsDate: Bool
    /// Date window distance from the center, as a fraction of the dial radius.
    public var dateRadius: Double
    public var dateWheel: RGB
    public var dateText: RGB
    /// Name printed under 12. Empty for none.
    public var brand: String
    public var subtitle: String
    public var depthRating: String

    private static let steelLight = RGB(0.97, 0.97, 0.98)
    private static let steelDark = RGB(0.28, 0.29, 0.32)

    // MARK: - Bauhaus Lume: the original Bauhaus layout, in 3D, cyan lume on black
    public static let bauhausLume = RealisticWatchTheme(
        name: "Bauhaus Lume", layout: .bauhaus, surface: .matte, hands: .pencil,
        dialCenter: RGB(hex: 0x0E1216), dialEdge: RGB(hex: 0x020304),
        print: RGB(0.20, 0.36, 0.48), accent: RGB(0.0, 0.95, 1.0), numeral: RGB(0.0, 0.95, 1.0),
        secondary: RGB(hex: 0x0E1216), tertiary: RGB(hex: 0x0E1216),
        indexLight: RGB(0.62, 0.68, 0.74), indexDark: RGB(0.10, 0.12, 0.15),
        lume: RGB(0.35, 0.97, 1.0), lumeGlow: 0.80,
        handLight: RGB(0.55, 0.60, 0.68), handDark: RGB(0.07, 0.09, 0.12),
        second: RGB(0.30, 0.92, 1.0), secondTip: RGB(0.30, 0.92, 1.0),
        showsDate: false, dateRadius: 0.8, dateWheel: RGB(hex: 0x0A0C0E), dateText: RGB(0.9, 0.9, 0.9),
        brand: "", subtitle: "", depthRating: ""
    )

    // MARK: - Obsidian: dress watch, black sunburst, polished steel batons
    public static let obsidian = RealisticWatchTheme(
        name: "Obsidian", layout: .dress, surface: .sunburst, hands: .sword,
        dialCenter: RGB(hex: 0x26282D), dialEdge: RGB(hex: 0x050607),
        print: RGB(0.86, 0.87, 0.89), accent: RGB(0.86, 0.18, 0.14), numeral: RGB(0.86, 0.87, 0.89),
        secondary: RGB(hex: 0x111214), tertiary: RGB(hex: 0x111214),
        indexLight: steelLight, indexDark: RGB(0.30, 0.31, 0.34),
        lume: RGB(0.88, 0.95, 0.89), lumeGlow: 0.30,
        handLight: steelLight, handDark: steelDark,
        second: RGB(0.88, 0.89, 0.91), secondTip: RGB(0.86, 0.18, 0.14),
        showsDate: true, dateRadius: 0.80, dateWheel: RGB(hex: 0x0D0E10), dateText: RGB(0.92, 0.92, 0.93),
        brand: "TIMED", subtitle: "AUTOMATIC", depthRating: ""
    )

    // MARK: - Pilot: matte black, big printed numerals, blued hands
    public static let pilot = RealisticWatchTheme(
        name: "Pilot", layout: .pilot, surface: .matte, hands: .sword,
        dialCenter: RGB(hex: 0x1B1C1D), dialEdge: RGB(hex: 0x040404),
        print: RGB(0.93, 0.90, 0.82), accent: RGB(0.93, 0.90, 0.82), numeral: RGB(0.94, 0.91, 0.83),
        secondary: RGB(hex: 0x111111), tertiary: RGB(hex: 0x111111),
        indexLight: steelLight, indexDark: steelDark,
        lume: RGB(0.94, 0.91, 0.80), lumeGlow: 0.30,
        handLight: RGB(0.40, 0.52, 0.86), handDark: RGB(0.04, 0.07, 0.20),
        second: RGB(0.93, 0.90, 0.82), secondTip: RGB(0.93, 0.90, 0.82),
        showsDate: false, dateRadius: 0.8, dateWheel: RGB(hex: 0x0D0E10), dateText: RGB(0.9, 0.9, 0.9),
        brand: "", subtitle: "", depthRating: ""
    )

    // MARK: - Chronograph: charcoal sunburst, recessed sub-dials, tachymeter, red seconds
    public static let chronograph = RealisticWatchTheme(
        name: "Chronograph", layout: .chronograph, surface: .sunburst, hands: .stick,
        dialCenter: RGB(hex: 0x2C2E31), dialEdge: RGB(hex: 0x060607),
        print: RGB(0.88, 0.89, 0.90), accent: RGB(0.92, 0.22, 0.16), numeral: RGB(0.88, 0.89, 0.90),
        secondary: RGB(hex: 0x111214), tertiary: RGB(hex: 0x111214),
        indexLight: steelLight, indexDark: RGB(0.30, 0.31, 0.34),
        lume: RGB(0.90, 0.96, 0.92), lumeGlow: 0.30,
        handLight: steelLight, handDark: steelDark,
        second: RGB(0.92, 0.22, 0.16), secondTip: RGB(0.92, 0.22, 0.16),
        showsDate: false, dateRadius: 0.8, dateWheel: RGB(hex: 0x0D0E10), dateText: RGB(0.9, 0.9, 0.9),
        brand: "TIMED", subtitle: "CHRONOGRAPH", depthRating: ""
    )

    // MARK: - Lunar: blue guilloché, Roman numerals, Breguet hands, live moon phase
    public static let lunar = RealisticWatchTheme(
        name: "Lunar", layout: .lunar, surface: .guilloche, hands: .breguet,
        dialCenter: RGB(0.13, 0.24, 0.48), dialEdge: RGB(0.02, 0.05, 0.13),
        print: RGB(0.88, 0.90, 0.94), accent: RGB(0.98, 0.88, 0.60), numeral: RGB(0.90, 0.92, 0.96),
        secondary: RGB(0.03, 0.07, 0.20), tertiary: RGB(0.98, 0.89, 0.62),
        indexLight: steelLight, indexDark: RGB(0.34, 0.37, 0.44),
        lume: RGB(0.9, 0.9, 0.9), lumeGlow: 0.0,
        handLight: RGB(0.99, 0.99, 1.0), handDark: RGB(0.34, 0.37, 0.44),
        second: RGB(0.90, 0.92, 0.96), secondTip: RGB(0.90, 0.92, 0.96),
        showsDate: false, dateRadius: 0.8, dateWheel: RGB(hex: 0x0D0E10), dateText: RGB(0.9, 0.9, 0.9),
        brand: "TIMED", subtitle: "PHASES DE LUNE", depthRating: ""
    )

    // MARK: - Meridian: GMT, two-tone 24-hour ring, arrowhead markers, UTC hand
    public static let meridian = RealisticWatchTheme(
        name: "Meridian", layout: .gmt, surface: .sunburst, hands: .sword,
        dialCenter: RGB(hex: 0x202328), dialEdge: RGB(hex: 0x050607),
        print: RGB(0.92, 0.93, 0.95), accent: RGB(0.96, 0.38, 0.14), numeral: RGB(0.92, 0.93, 0.95),
        secondary: RGB(0.74, 0.78, 0.84), tertiary: RGB(0.05, 0.09, 0.22),
        indexLight: steelLight, indexDark: RGB(0.30, 0.31, 0.34),
        lume: RGB(0.85, 0.97, 0.88), lumeGlow: 0.35,
        handLight: steelLight, handDark: steelDark,
        second: RGB(0.90, 0.91, 0.93), secondTip: RGB(0.96, 0.38, 0.14),
        showsDate: true, dateRadius: 0.68, dateWheel: RGB(hex: 0x0D0E10), dateText: RGB(0.92, 0.92, 0.93),
        brand: "TIMED", subtitle: "GMT", depthRating: ""
    )

    // MARK: - Abyss: matte black diver, lume dots, orange seconds
    public static let abyss = RealisticWatchTheme(
        name: "Abyss", layout: .diver, surface: .matte, hands: .sword,
        dialCenter: RGB(hex: 0x16181A), dialEdge: RGB(hex: 0x040505),
        print: RGB(0.88, 0.90, 0.90), accent: RGB(1.0, 0.48, 0.12), numeral: RGB(0.88, 0.90, 0.90),
        secondary: RGB(hex: 0x111214), tertiary: RGB(hex: 0x111214),
        indexLight: RGB(0.96, 0.97, 0.98), indexDark: RGB(0.32, 0.33, 0.36),
        lume: RGB(0.80, 1.0, 0.84), lumeGlow: 0.60,
        handLight: RGB(0.96, 0.97, 0.98), handDark: RGB(0.30, 0.31, 0.34),
        second: RGB(1.0, 0.48, 0.12), secondTip: RGB(1.0, 0.48, 0.12),
        showsDate: true, dateRadius: 0.80, dateWheel: RGB(hex: 0x0C0D0E), dateText: RGB(0.92, 0.93, 0.93),
        brand: "TIMED", subtitle: "AUTOMATIC", depthRating: "300 M"
    )
}

/// Every face the screen saver can show.
public enum WatchFace: String, CaseIterable, Sendable, Identifiable {
    case bauhaus
    case bauhausLume
    case obsidian
    case pilot
    case chronograph
    case lunar
    case meridian
    case abyss

    public var id: String { rawValue }

    public var displayName: String {
        switch self {
        case .bauhaus: return "Bauhaus (original, follows system)"
        case .bauhausLume: return "Bauhaus Lume (3D)"
        case .obsidian: return "Obsidian (dress)"
        case .pilot: return "Pilot"
        case .chronograph: return "Chronograph"
        case .lunar: return "Lunar (moon phase)"
        case .meridian: return "Meridian (GMT)"
        case .abyss: return "Abyss (diver)"
        }
    }

    public var realisticTheme: RealisticWatchTheme? {
        switch self {
        case .bauhaus: return nil
        case .bauhausLume: return .bauhausLume
        case .obsidian: return .obsidian
        case .pilot: return .pilot
        case .chronograph: return .chronograph
        case .lunar: return .lunar
        case .meridian: return .meridian
        case .abyss: return .abyss
        }
    }
}
