import SwiftUI

/// Pure Dark Palette and aesthetic tokens for the Timed macOS Screen Saver
public struct TimedColorPalette: Sendable {
    // Backdrop
    public let backdropGradientTop: Color
    public let backdropGradientBottom: Color

    // Squircle card
    public let cardGradientTop: Color
    public let cardGradientBottom: Color
    public let cardBorderTop: Color
    public let cardBorderBottom: Color
    public let cardShadowPrimary: Color
    public let cardShadowSecondary: Color

    // Digital pill well
    public let pillBackgroundTop: Color
    public let pillBackgroundBottom: Color
    public let pillBorderTop: Color
    public let pillBorderBottom: Color
    public let pillHourText: Color
    public let pillMinuteText: Color
    public let pillSecondText: Color
    public let pillDot: Color
    public let pillDotGlow: Color

    // AM/PM Dish
    public let badgeBackgroundTop: Color
    public let badgeBackgroundBottom: Color
    public let badgeBorder: Color
    public let badgeText: Color

    // Dial elements
    public let spokeLine: Color
    public let minuteTick: Color
    public let hourNumeral: Color
    public let centerDiscFill: Color
    public let centerDiscBorder: Color
    public let centerDiscShadow: Color

    // Hour marker lime capsule: electric neon lime #8cf026
    public let hourMarkerCore: Color
    public let hourMarkerCoreGradientBottom: Color
    public let hourMarkerFrame: Color
    public let hourMarkerGlow: Color
    public let hourMarkerShadow: Color

    // Clock hands
    public let handBodyFillTop: Color
    public let handBodyFillBottom: Color
    public let handBodyBorder: Color
    public let handBodyShadow: Color
    public let handNeonGroove: Color
    public let handNeonGlow: Color

    // Second hand
    public let secondHandNeedle: Color
    public let secondHandTip: Color
    public let secondHandTipGlow: Color
    public let secondHandCap: Color

    public static let shared = TimedColorPalette(
        // Deep obsidian OLED backdrop
        backdropGradientTop: Color(red: 0.050, green: 0.052, blue: 0.058),
        backdropGradientBottom: Color(red: 0.025, green: 0.026, blue: 0.030),

        // Dark squircle card: matte charcoal obsidian
        cardGradientTop: Color(red: 0.125, green: 0.128, blue: 0.138),
        cardGradientBottom: Color(red: 0.082, green: 0.084, blue: 0.092),
        cardBorderTop: Color.white.opacity(0.12),
        cardBorderBottom: Color.white.opacity(0.03),
        cardShadowPrimary: Color.black.opacity(0.70),
        cardShadowSecondary: Color.black.opacity(0.40),

        // Inset digital capsule
        pillBackgroundTop: Color(red: 0.065, green: 0.068, blue: 0.074),
        pillBackgroundBottom: Color(red: 0.085, green: 0.088, blue: 0.095),
        pillBorderTop: Color.white.opacity(0.04),
        pillBorderBottom: Color.white.opacity(0.08),
        pillHourText: Color(red: 0.95, green: 0.95, blue: 0.97),
        pillMinuteText: Color(red: 0.58, green: 0.60, blue: 0.65),
        pillSecondText: Color(red: 0.55, green: 0.94, blue: 0.15),
        pillDot: Color(red: 1.0, green: 0.612, blue: 0.035),
        pillDotGlow: Color(red: 1.0, green: 0.612, blue: 0.035, opacity: 0.60),

        // AM/PM Dish
        badgeBackgroundTop: Color(red: 0.065, green: 0.068, blue: 0.074),
        badgeBackgroundBottom: Color(red: 0.085, green: 0.088, blue: 0.095),
        badgeBorder: Color.white.opacity(0.06),
        badgeText: Color(red: 0.55, green: 0.58, blue: 0.64),

        // Dial spokes and markings
        spokeLine: Color.white.opacity(0.08),
        minuteTick: Color(red: 0.32, green: 0.34, blue: 0.38),
        hourNumeral: Color(red: 0.42, green: 0.44, blue: 0.50),
        centerDiscFill: Color(red: 0.13, green: 0.135, blue: 0.145),
        centerDiscBorder: Color.white.opacity(0.18),
        centerDiscShadow: Color.black.opacity(0.50),

        // Hour marker lime capsule: electric neon lime #8cf026
        hourMarkerCore: Color(red: 0.55, green: 0.94, blue: 0.15),
        hourMarkerCoreGradientBottom: Color(red: 0.45, green: 0.83, blue: 0.12),
        hourMarkerFrame: Color(red: 0.17, green: 0.18, blue: 0.20),
        hourMarkerGlow: Color(red: 0.55, green: 0.94, blue: 0.15, opacity: 0.50),
        hourMarkerShadow: Color.black.opacity(0.60),

        // Hour / Minute hands: sculpted dark titanium pointer with neon lime spine
        handBodyFillTop: Color(red: 0.20, green: 0.205, blue: 0.22),
        handBodyFillBottom: Color(red: 0.12, green: 0.125, blue: 0.135),
        handBodyBorder: Color.white.opacity(0.20),
        handBodyShadow: Color.black.opacity(0.70),
        handNeonGroove: Color(red: 0.55, green: 0.94, blue: 0.15),
        handNeonGlow: Color(red: 0.55, green: 0.94, blue: 0.15, opacity: 0.65),

        // Seconds hand: needle with neon lime tip & glowing aura
        secondHandNeedle: Color(red: 0.75, green: 0.77, blue: 0.82),
        secondHandTip: Color(red: 0.55, green: 0.94, blue: 0.15),
        secondHandTipGlow: Color(red: 0.55, green: 0.94, blue: 0.15, opacity: 0.75),
        secondHandCap: Color(red: 0.55, green: 0.94, blue: 0.15)
    )
}
