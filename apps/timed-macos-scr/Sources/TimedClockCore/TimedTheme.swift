import SwiftUI

/// Color theme and aesthetic tokens for the Bauhaus Horological Clock Screen Saver
/// Exactly replicating the Bauhaus Clock design (Glacier White Light Mode & Swiss BGW9 Dark Mode).
public struct BauhausDialTheme: Sendable {
    public let isDark: Bool

    // Background Canvas
    public let canvasBackgroundGradientTop: Color
    public let canvasBackgroundGradientBottom: Color
    public let dialAmbientGlow: Color

    // Tier 1: Inner Hour Numerals (1–12)
    public let innerNumeralColor: Color
    public let innerNumeralGlowColor: Color
    public let innerNumeralGlowRadius: CGFloat

    // Tier 2: 3D Embossed Hour Batons (12 pills)
    public let batonFillColor: Color
    public let batonFillGradientBottom: Color
    public let batonHighlightColor: Color
    public let batonShadowColor: Color
    public let batonGlowColor: Color
    public let batonGlowRadius: CGFloat

    // Tier 3: Minute Track Ticks
    public let minuteTickColor: Color

    // Tier 4: Outer 5-Minute Numerals (60, 05, 10, ... 55)
    public let outerNumeralColor: Color

    // Hands: Hour & Minute
    public let handBodyLeftColor: Color
    public let handBodyRightColor: Color
    public let handBodyShadowColor: Color
    public let handGrooveColor: Color
    public let handGrooveGradientBottom: Color
    public let handGrooveGlowColor: Color
    public let handBorderColor: Color

    // Second Hand
    public let secondHandColor: Color
    public let secondHandGlowColor: Color
    public let secondHandShadowColor: Color

    // Center Pinion Washer & Collet
    public let pinionOuterColor: Color
    public let pinionBorderColor: Color
    public let pinionCenterHoleColor: Color
    public let pinionHighlightColor: Color

    // MARK: - Light Mode: Glacier Ceramic White
    public static let light = BauhausDialTheme(
        isDark: false,
        // Soothing off-white glacier ceramic canvas with subtle ambient gradient
        canvasBackgroundGradientTop: Color(red: 0.935, green: 0.948, blue: 0.955),
        canvasBackgroundGradientBottom: Color(red: 0.895, green: 0.912, blue: 0.924),
        dialAmbientGlow: Color.white.opacity(0.40),

        // Inner hour numerals: Deep charcoal graphite
        innerNumeralColor: Color(red: 0.120, green: 0.140, blue: 0.160),
        innerNumeralGlowColor: Color.clear,
        innerNumeralGlowRadius: 0,

        // 3D tactile batons: Soft ice-cyan ceramic fill with embossed bevels
        batonFillColor: Color(red: 0.690, green: 0.830, blue: 0.880),
        batonFillGradientBottom: Color(red: 0.620, green: 0.770, blue: 0.830),
        batonHighlightColor: Color.white.opacity(0.95),
        batonShadowColor: Color(red: 0.10, green: 0.18, blue: 0.25, opacity: 0.24),
        batonGlowColor: Color.clear,
        batonGlowRadius: 0,

        // Minute tick marks: Crisp charcoal
        minuteTickColor: Color(red: 0.140, green: 0.165, blue: 0.195, opacity: 0.92),

        // Outer 5-minute numerals: Subtle slate graphite
        outerNumeralColor: Color(red: 0.200, green: 0.235, blue: 0.270, opacity: 0.88),

        // Hands: Brushed titanium with longitudinal light/shadow split & ice-cyan lume groove
        handBodyLeftColor: Color(red: 0.91, green: 0.93, blue: 0.95),
        handBodyRightColor: Color(red: 0.72, green: 0.76, blue: 0.80),
        handBodyShadowColor: Color(red: 0.08, green: 0.12, blue: 0.18, opacity: 0.22),
        handGrooveColor: Color(red: 0.710, green: 0.840, blue: 0.890),
        handGrooveGradientBottom: Color(red: 0.640, green: 0.780, blue: 0.840),
        handGrooveGlowColor: Color.clear,
        handBorderColor: Color(red: 0.48, green: 0.54, blue: 0.60),

        // Second hand: Fine needle in crisp steel with soft shadow
        secondHandColor: Color(red: 0.26, green: 0.30, blue: 0.35),
        secondHandGlowColor: Color.clear,
        secondHandShadowColor: Color.black.opacity(0.18),

        // Center Pinion
        pinionOuterColor: Color(red: 0.88, green: 0.91, blue: 0.94),
        pinionBorderColor: Color(red: 0.52, green: 0.58, blue: 0.64),
        pinionCenterHoleColor: Color(red: 0.18, green: 0.21, blue: 0.25),
        pinionHighlightColor: Color.white.opacity(0.9)
    )

    // MARK: - Dark Mode: OLED Pitch Black with Swiss BGW9 Cyan Lume
    public static let dark = BauhausDialTheme(
        isDark: true,
        // Deep obsidian OLED black canvas
        canvasBackgroundGradientTop: Color(red: 0.020, green: 0.024, blue: 0.028),
        canvasBackgroundGradientBottom: Color(red: 0.008, green: 0.010, blue: 0.014),
        dialAmbientGlow: Color(red: 0.0, green: 0.95, blue: 1.0, opacity: 0.03),

        // Inner hour numerals: Radiant Swiss BGW9 electric cyan with soft bloom
        innerNumeralColor: Color(red: 0.0, green: 0.96, blue: 1.0),
        innerNumeralGlowColor: Color(red: 0.0, green: 0.94, blue: 1.0, opacity: 0.60),
        innerNumeralGlowRadius: 9.0,

        // 3D tactile batons: Intense glowing Super-LumiNova cyan capsules
        batonFillColor: Color(red: 0.40, green: 0.98, blue: 1.0),
        batonFillGradientBottom: Color(red: 0.0, green: 0.88, blue: 0.98),
        batonHighlightColor: Color.white.opacity(0.95),
        batonShadowColor: Color.black.opacity(0.70),
        batonGlowColor: Color(red: 0.0, green: 0.94, blue: 1.0, opacity: 0.75),
        batonGlowRadius: 12.0,

        // Minute tick marks: Faint deep slate blue (barely visible in the dark)
        minuteTickColor: Color(red: 0.12, green: 0.18, blue: 0.24, opacity: 0.80),

        // Outer 5-minute numerals: Subtle deep twilight slate cyan
        outerNumeralColor: Color(red: 0.16, green: 0.30, blue: 0.42, opacity: 0.75),

        // Hands: Sculpted dark titanium frame with glowing BGW9 cyan luminous core
        handBodyLeftColor: Color(red: 0.18, green: 0.21, blue: 0.26),
        handBodyRightColor: Color(red: 0.08, green: 0.10, blue: 0.13),
        handBodyShadowColor: Color.black.opacity(0.85),
        handGrooveColor: Color(red: 0.0, green: 0.96, blue: 1.0),
        handGrooveGradientBottom: Color(red: 0.0, green: 0.90, blue: 1.0),
        handGrooveGlowColor: Color(red: 0.0, green: 0.94, blue: 1.0, opacity: 0.85),
        handBorderColor: Color.white.opacity(0.25),

        // Second hand: Precision needle in glowing cyan with soft tip aura
        secondHandColor: Color(red: 0.30, green: 0.92, blue: 1.0),
        secondHandGlowColor: Color(red: 0.0, green: 0.94, blue: 1.0, opacity: 0.70),
        secondHandShadowColor: Color.black.opacity(0.80),

        // Center Pinion
        pinionOuterColor: Color(red: 0.18, green: 0.22, blue: 0.28),
        pinionBorderColor: Color(red: 0.0, green: 0.90, blue: 1.0, opacity: 0.50),
        pinionCenterHoleColor: Color.black,
        pinionHighlightColor: Color.white.opacity(0.35)
    )
}
