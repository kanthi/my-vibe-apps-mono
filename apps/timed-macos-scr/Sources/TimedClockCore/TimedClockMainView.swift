import SwiftUI

/// Borderless Bauhaus Minimalist Horological Clock macOS Screen Saver
/// Authentic replication of the Bauhaus Clock by Atilla Taşkıran (bauhausclock.com).
/// 
/// Exact Horological Geometry:
/// - Outer 5-minute numerals (60, 05, 10, ... 55) tangentially aligned with circular Bauhaus glyphs
/// - Precision minute track (4 ticks between each hour baton) sharing exact outer perimeter with batons
/// - 3D embossed tactile pill batons at each hour
/// - Inner upright Bauhaus rounded hour numerals (1–12)
/// - Sculpted pencil hands with longitudinal 3D chamfers, luminous central channels & continuous 60 FPS sweeping second hand
/// - Fully adaptive Light (Glacier White) and Dark (Swiss BGW9 Cyan Lume) modes
public struct TimedClockMainView: View {
    @ObservedObject public var model: TimedClockModel
    public var explicitTheme: BauhausDialTheme?

    @Environment(\.colorScheme) private var systemColorScheme

    public init(model: TimedClockModel = .shared, theme: BauhausDialTheme? = nil) {
        self.model = model
        self.explicitTheme = theme
    }

    public var theme: BauhausDialTheme {
        if let explicit = explicitTheme { return explicit }
        return systemColorScheme == .dark ? .dark : .light
    }

    public var body: some View {
        GeometryReader { proxy in
            let w = proxy.size.width
            let h = proxy.size.height
            let minDim = min(w, h)
            let center = CGPoint(x: w / 2.0, y: h / 2.0)

            // Empirical horological proportions derived from reference video
            let outerRadius = minDim * 0.405

            // Batons & Minute Track share exact outer perimeter
            let dialPerimeterRadius = outerRadius * 0.885
            let batonLength = outerRadius * 0.145
            let batonWidth = outerRadius * 0.030
            let batonCenterRadius = dialPerimeterRadius - batonLength / 2.0

            let minuteTickOuter = dialPerimeterRadius
            let minuteTickInner = dialPerimeterRadius - batonLength * 0.44
            let minuteTickWidth = max(1.1, outerRadius * 0.0055)

            // Numerals
            let innerHourRadius = outerRadius * 0.530
            let innerHourFontSize = outerRadius * 0.106
            let outerFiveMinRadius = outerRadius * 0.965
            let outerFiveMinFontSize = outerRadius * 0.054

            // Hands
            let hourHandLength = outerRadius * 0.520
            let hourHandWidth = outerRadius * 0.038
            let minuteHandLength = outerRadius * 0.735
            let minuteHandWidth = outerRadius * 0.028
            let secondHandLength = dialPerimeterRadius * 0.985
            let secondHandTail = outerRadius * 0.175
            let pinionRadius = outerRadius * 0.036
            let pinionHoleRadius = outerRadius * 0.012

            ZStack {
                // 1. Edge-to-Edge Canvas Background
                canvasBackground(width: w, height: h)

                // 2. Dial Ambient Center Glow (Soft vignette)
                dialAmbientLighting(center: center, radius: outerRadius * 1.25)

                // 3. Tier 3: Minute Track Ticks
                minuteTrackTicks(
                    center: center,
                    innerRadius: minuteTickInner,
                    outerRadius: minuteTickOuter,
                    lineWidth: minuteTickWidth
                )

                // 4. Tier 2: 3D Embossed Hour Batons (12 radial pills)
                hourBatons(
                    center: center,
                    centerRadius: batonCenterRadius,
                    length: batonLength,
                    width: batonWidth
                )

                // 5. Tier 1: Inner Upright Hour Numerals (1–12)
                innerHourNumerals(
                    center: center,
                    radius: innerHourRadius,
                    fontSize: innerHourFontSize
                )

                // 6. Tier 4: Outer Tangentially Aligned 5-Minute Numerals (60, 05, 10, ... 55)
                outerFiveMinNumerals(
                    center: center,
                    radius: outerFiveMinRadius,
                    fontSize: outerFiveMinFontSize
                )

                // 7. Center Pinion Washer & Collet (beneath hands)
                centerPinion(
                    radius: pinionRadius,
                    holeRadius: pinionHoleRadius,
                    center: center
                )

                // 8. Hour Hand (3D Faceted pencil hand with luminous groove)
                WatchHandView(
                    length: hourHandLength,
                    baseWidth: hourHandWidth,
                    tipWidth: hourHandWidth * 0.65,
                    grooveWidth: hourHandWidth * 0.38,
                    grooveLength: hourHandLength * 0.70,
                    theme: theme,
                    degrees: model.hourHandDegrees
                )
                .position(center)

                // 9. Minute Hand (3D Faceted pencil hand with luminous groove)
                WatchHandView(
                    length: minuteHandLength,
                    baseWidth: minuteHandWidth,
                    tipWidth: minuteHandWidth * 0.65,
                    grooveWidth: minuteHandWidth * 0.36,
                    grooveLength: minuteHandLength * 0.80,
                    theme: theme,
                    degrees: model.minuteHandDegrees
                )
                .position(center)

                // 10. Silky Sweeping Second Hand (60 FPS continuous)
                SweepingSecondHandView(
                    length: secondHandLength,
                    tail: secondHandTail,
                    needleWidth: max(1.3, outerRadius * 0.0055),
                    capRadius: max(3.5, outerRadius * 0.018),
                    theme: theme,
                    degrees: model.secondHandDegrees
                )
                .position(center)
            }
            .frame(width: w, height: h)
        }
        .ignoresSafeArea()
    }

    // MARK: - 1. Canvas Background
    private func canvasBackground(width: CGFloat, height: CGFloat) -> some View {
        LinearGradient(
            colors: [
                theme.canvasBackgroundGradientTop,
                theme.canvasBackgroundGradientBottom
            ],
            startPoint: .top,
            endPoint: .bottom
        )
        .ignoresSafeArea()
    }

    // MARK: - 2. Dial Ambient Lighting
    private func dialAmbientLighting(center: CGPoint, radius: CGFloat) -> some View {
        RadialGradient(
            colors: [
                theme.dialAmbientGlow,
                Color.clear
            ],
            center: .center,
            startRadius: 0,
            endRadius: radius
        )
        .allowsHitTesting(false)
    }

    // MARK: - 3. Tier 3: Minute Track Ticks
    private func minuteTrackTicks(
        center: CGPoint,
        innerRadius: CGFloat,
        outerRadius: CGFloat,
        lineWidth: CGFloat
    ) -> some View {
        Canvas { context, _ in
            for m in 0..<60 {
                // Skip the hour marks where the 12 batons sit
                if m % 5 == 0 { continue }

                let angleDeg = Double(m) * 6.0
                let rad = (angleDeg - 90.0) * .pi / 180.0
                let cosA = cos(rad)
                let sinA = sin(rad)

                let p1 = CGPoint(
                    x: center.x + cosA * innerRadius,
                    y: center.y + sinA * innerRadius
                )
                let p2 = CGPoint(
                    x: center.x + cosA * outerRadius,
                    y: center.y + sinA * outerRadius
                )

                var path = Path()
                path.move(to: p1)
                path.addLine(to: p2)

                context.stroke(
                    path,
                    with: .color(theme.minuteTickColor),
                    style: StrokeStyle(lineWidth: lineWidth, lineCap: .round)
                )
            }
        }
        .allowsHitTesting(false)
    }

    // MARK: - 4. Tier 2: 3D Embossed Hour Batons (12 pills)
    private func hourBatons(
        center: CGPoint,
        centerRadius: CGFloat,
        length: CGFloat,
        width: CGFloat
    ) -> some View {
        ZStack {
            ForEach(0..<12, id: \.self) { i in
                let angle = Double(i) * 30.0

                ZStack {
                    if theme.isDark {
                        // Dark Mode: Radiant Super-LumiNova BGW9 Cyan
                        // 1. Ambient Bloom
                        Capsule()
                            .fill(theme.batonGlowColor)
                            .frame(width: width + 5, height: length + 5)
                            .blur(radius: theme.batonGlowRadius)

                        // 2. Luminous Body
                        Capsule()
                            .fill(
                                LinearGradient(
                                    colors: [theme.batonFillColor, theme.batonFillGradientBottom],
                                    startPoint: .top,
                                    endPoint: .bottom
                                )
                            )
                            .frame(width: width, height: length)
                            .shadow(color: theme.batonGlowColor, radius: 4)

                        // 3. Bright Core
                        Capsule()
                            .fill(Color.white.opacity(0.85))
                            .frame(width: max(1.2, width * 0.35), height: length * 0.75)
                            .blur(radius: 0.8)
                    } else {
                        // Light Mode: Tactile 3D Embossed Ceramic Pill
                        // 1. Cast Drop Shadow (Bottom-Right)
                        Capsule()
                            .fill(theme.batonShadowColor)
                            .frame(width: width, height: length)
                            .offset(x: 1.4, y: 1.8)
                            .blur(radius: 1.5)

                        // 2. Beveled Highlight Rim (Top-Left)
                        Capsule()
                            .fill(theme.batonHighlightColor)
                            .frame(width: width + 0.8, height: length + 0.8)
                            .offset(x: -0.8, y: -0.8)

                        // 3. Ceramic Baton Body
                        Capsule()
                            .fill(
                                LinearGradient(
                                    colors: [theme.batonFillColor, theme.batonFillGradientBottom],
                                    startPoint: .topLeading,
                                    endPoint: .bottomTrailing
                                )
                            )
                            .frame(width: width, height: length)

                        // 4. Subtle Inner Highlight Glint
                        Capsule()
                            .stroke(theme.batonHighlightColor.opacity(0.70), lineWidth: 0.6)
                            .frame(width: width - 0.6, height: length - 0.6)
                    }
                }
                .offset(y: -centerRadius)
                .rotationEffect(.degrees(angle))
            }
        }
        .position(center)
        .allowsHitTesting(false)
    }

    // MARK: - 5. Tier 1: Inner Upright Hour Numerals (1–12)
    private func innerHourNumerals(
        center: CGPoint,
        radius: CGFloat,
        fontSize: CGFloat
    ) -> some View {
        ZStack {
            ForEach(1...12, id: \.self) { hour in
                let hourAngleDeg = Double(hour) * 30.0
                let rad = (hourAngleDeg - 90.0) * .pi / 180.0
                let x = center.x + cos(rad) * radius
                let y = center.y + sin(rad) * radius

                Text("\(hour)")
                    .font(.system(size: fontSize, weight: .regular, design: .rounded))
                    .foregroundColor(theme.innerNumeralColor)
                    .if(theme.isDark) { view in
                        view
                            .shadow(color: theme.innerNumeralGlowColor, radius: theme.innerNumeralGlowRadius)
                            .shadow(color: theme.innerNumeralGlowColor.opacity(0.8), radius: 2)
                    }
                    .position(x: x, y: y)
            }
        }
        .allowsHitTesting(false)
    }

    // MARK: - 6. Tier 4: Outer Tangentially Aligned 5-Minute Numerals
    private func outerFiveMinNumerals(
        center: CGPoint,
        radius: CGFloat,
        fontSize: CGFloat
    ) -> some View {
        let labels = ["60", "05", "10", "15", "20", "25", "30", "35", "40", "45", "50", "55"]

        return ZStack {
            ForEach(0..<12, id: \.self) { i in
                let hourDeg = Double(i) * 30.0
                let rad = (hourDeg - 90.0) * .pi / 180.0
                let x = center.x + cos(rad) * radius
                let y = center.y + sin(rad) * radius

                let rotationAngle = rotationForFiveMin(hourDeg: hourDeg)

                Text(labels[i])
                    // Using .rounded without monospacedDigit gives authentic circular 0, 6, 8, 9
                    .font(.system(size: fontSize, weight: .regular, design: .rounded))
                    .foregroundColor(theme.outerNumeralColor)
                    .rotationEffect(.degrees(rotationAngle))
                    .position(x: x, y: y)
            }
        }
        .allowsHitTesting(false)
    }

    private func rotationForFiveMin(hourDeg: Double) -> Double {
        if hourDeg > 90.0 && hourDeg < 270.0 {
            return hourDeg - 180.0
        } else if hourDeg >= 270.0 {
            return hourDeg - 360.0
        } else {
            return hourDeg
        }
    }

    // MARK: - Center Pinion Washer & Collet
    private func centerPinion(
        radius: CGFloat,
        holeRadius: CGFloat,
        center: CGPoint
    ) -> some View {
        ZStack {
            // Outer Pinion Washer with subtle drop shadow
            Circle()
                .fill(theme.pinionOuterColor)
                .frame(width: radius * 2.0, height: radius * 2.0)
                .shadow(color: Color.black.opacity(0.25), radius: 2.0, x: 1.0, y: 1.5)

            // Outer Pinion Highlight Rim
            Circle()
                .stroke(theme.pinionBorderColor, lineWidth: 0.8)
                .frame(width: radius * 2.0, height: radius * 2.0)

            // Center Aperture Hole
            Circle()
                .fill(theme.pinionCenterHoleColor)
                .frame(width: holeRadius * 2.0, height: holeRadius * 2.0)

            // Center Hole Inner Shadow / Rim
            Circle()
                .stroke(theme.pinionBorderColor.opacity(0.5), lineWidth: 0.5)
                .frame(width: holeRadius * 2.0, height: holeRadius * 2.0)
        }
        .position(center)
    }
}

// MARK: - 3D Faceted Watch Hand View (Hour & Minute)
struct WatchHandView: View {
    let length: CGFloat
    let baseWidth: CGFloat
    let tipWidth: CGFloat
    let grooveWidth: CGFloat
    let grooveLength: CGFloat
    let theme: BauhausDialTheme
    let degrees: Double

    var body: some View {
        let chiselHeight = tipWidth * 0.75
        let frameSize = length * 2.2

        ZStack {
            // 1. Hand Elevation Drop Shadow
            TaperedHandShape(
                length: length,
                baseWidth: baseWidth,
                tipWidth: tipWidth,
                chiselHeight: chiselHeight
            )
            .fill(theme.handBodyShadowColor)
            .offset(
                x: theme.isDark ? 1.5 : 2.5,
                y: theme.isDark ? 2.5 : 3.8
            )
            .blur(radius: theme.isDark ? 3.0 : 3.5)

            // 2. Hand Metal Body (Split 3D longitudinal chamfer)
            // Left flank (illuminated)
            TaperedHandHalfShape(
                length: length,
                baseWidth: baseWidth,
                tipWidth: tipWidth,
                chiselHeight: chiselHeight,
                isLeft: true
            )
            .fill(theme.handBodyLeftColor)

            // Right flank (shadow)
            TaperedHandHalfShape(
                length: length,
                baseWidth: baseWidth,
                tipWidth: tipWidth,
                chiselHeight: chiselHeight,
                isLeft: false
            )
            .fill(theme.handBodyRightColor)

            // Outer border stroke
            TaperedHandShape(
                length: length,
                baseWidth: baseWidth,
                tipWidth: tipWidth,
                chiselHeight: chiselHeight
            )
            .stroke(theme.handBorderColor, lineWidth: 0.75)

            // 3. Central Luminous Groove
            if theme.isDark {
                Capsule()
                    .fill(theme.handGrooveGlowColor)
                    .frame(width: grooveWidth + 3.0, height: grooveLength)
                    .blur(radius: 4.0)
                    .offset(y: -length * 0.50)

                Capsule()
                    .fill(theme.handGrooveColor)
                    .frame(width: grooveWidth, height: grooveLength)
                    .shadow(color: theme.handGrooveGlowColor, radius: 2.0)
                    .offset(y: -length * 0.50)

                Capsule()
                    .fill(Color.white.opacity(0.85))
                    .frame(width: max(1.0, grooveWidth * 0.35), height: grooveLength * 0.85)
                    .offset(y: -length * 0.50)
            } else {
                Capsule()
                    .fill(
                        LinearGradient(
                            colors: [theme.handGrooveColor, theme.handGrooveGradientBottom],
                            startPoint: .top,
                            endPoint: .bottom
                        )
                    )
                    .frame(width: grooveWidth, height: grooveLength)
                    .offset(y: -length * 0.50)

                Capsule()
                    .stroke(theme.handBorderColor.opacity(0.60), lineWidth: 0.6)
                    .frame(width: grooveWidth, height: grooveLength)
                    .offset(y: -length * 0.50)
            }
        }
        .frame(width: frameSize, height: frameSize)
        .rotationEffect(.degrees(degrees))
    }
}

// MARK: - Sweeping Second Hand View
struct SweepingSecondHandView: View {
    let length: CGFloat
    let tail: CGFloat
    let needleWidth: CGFloat
    let capRadius: CGFloat
    let theme: BauhausDialTheme
    let degrees: Double

    var body: some View {
        let frameSize = (length + tail) * 2.2

        ZStack {
            // Shadow
            SecondHandShape(length: length, tail: tail, width: needleWidth)
                .fill(theme.secondHandShadowColor)
                .offset(x: 1.0, y: 1.8)
                .blur(radius: 1.5)

            // Needle Body
            SecondHandShape(length: length, tail: tail, width: needleWidth)
                .fill(theme.secondHandColor)
                .if(theme.isDark) { view in
                    view.shadow(color: theme.secondHandGlowColor, radius: 3)
                }

            // Pivot Center Cap
            Circle()
                .fill(theme.secondHandColor)
                .frame(width: capRadius * 2.0, height: capRadius * 2.0)
                .shadow(color: theme.secondHandShadowColor, radius: 1, x: 0.5, y: 1)
        }
        .frame(width: frameSize, height: frameSize)
        .rotationEffect(.degrees(degrees))
    }
}

// MARK: - Hand Geometry Shapes

/// Full Tapered Pencil Hand Shape
struct TaperedHandShape: Shape {
    var length: CGFloat
    var baseWidth: CGFloat
    var tipWidth: CGFloat
    var chiselHeight: CGFloat

    func path(in rect: CGRect) -> Path {
        var path = Path()
        let cx = rect.midX
        let cy = rect.midY
        let halfBase = baseWidth / 2.0
        let halfTip = tipWidth / 2.0

        path.move(to: CGPoint(x: cx, y: cy + halfBase * 0.8))
        path.addArc(
            center: CGPoint(x: cx, y: cy),
            radius: halfBase * 0.8,
            startAngle: .degrees(90),
            endAngle: .degrees(180),
            clockwise: false
        )
        path.addLine(to: CGPoint(x: cx - halfTip, y: cy - length + chiselHeight))
        path.addLine(to: CGPoint(x: cx, y: cy - length))
        path.addLine(to: CGPoint(x: cx + halfTip, y: cy - length + chiselHeight))
        path.addLine(to: CGPoint(x: cx + halfBase * 0.8, y: cy))
        path.addArc(
            center: CGPoint(x: cx, y: cy),
            radius: halfBase * 0.8,
            startAngle: .degrees(0),
            endAngle: .degrees(90),
            clockwise: false
        )
        path.closeSubpath()
        return path
    }
}

/// Half Tapered Hand Shape (for split 3D lighting facet)
struct TaperedHandHalfShape: Shape {
    var length: CGFloat
    var baseWidth: CGFloat
    var tipWidth: CGFloat
    var chiselHeight: CGFloat
    var isLeft: Bool

    func path(in rect: CGRect) -> Path {
        var path = Path()
        let cx = rect.midX
        let cy = rect.midY
        let halfBase = baseWidth / 2.0
        let halfTip = tipWidth / 2.0

        if isLeft {
            path.move(to: CGPoint(x: cx, y: cy + halfBase * 0.8))
            path.addArc(
                center: CGPoint(x: cx, y: cy),
                radius: halfBase * 0.8,
                startAngle: .degrees(90),
                endAngle: .degrees(180),
                clockwise: false
            )
            path.addLine(to: CGPoint(x: cx - halfTip, y: cy - length + chiselHeight))
            path.addLine(to: CGPoint(x: cx, y: cy - length))
            path.addLine(to: CGPoint(x: cx, y: cy + halfBase * 0.8))
        } else {
            path.move(to: CGPoint(x: cx, y: cy - length))
            path.addLine(to: CGPoint(x: cx + halfTip, y: cy - length + chiselHeight))
            path.addLine(to: CGPoint(x: cx + halfBase * 0.8, y: cy))
            path.addArc(
                center: CGPoint(x: cx, y: cy),
                radius: halfBase * 0.8,
                startAngle: .degrees(0),
                endAngle: .degrees(90),
                clockwise: false
            )
            path.addLine(to: CGPoint(x: cx, y: cy - length))
        }
        path.closeSubpath()
        return path
    }
}

/// Precision Sweeping Second Hand Needle Shape
struct SecondHandShape: Shape {
    var length: CGFloat
    var tail: CGFloat
    var width: CGFloat

    func path(in rect: CGRect) -> Path {
        var path = Path()
        let cx = rect.midX
        let cy = rect.midY
        let halfWidth = width / 2.0

        // Sweeping needle from (cx, cy - length) to (cx, cy + tail)
        path.move(to: CGPoint(x: cx - halfWidth * 0.7, y: cy - length))
        path.addLine(to: CGPoint(x: cx + halfWidth * 0.7, y: cy - length))
        path.addLine(to: CGPoint(x: cx + halfWidth, y: cy))
        path.addLine(to: CGPoint(x: cx + halfWidth * 1.6, y: cy + tail))
        path.addArc(
            center: CGPoint(x: cx, y: cy + tail),
            radius: halfWidth * 1.6,
            startAngle: .degrees(0),
            endAngle: .degrees(180),
            clockwise: false
        )
        path.addLine(to: CGPoint(x: cx - halfWidth * 1.6, y: cy + tail))
        path.addLine(to: CGPoint(x: cx - halfWidth, y: cy))
        path.closeSubpath()
        return path
    }
}

// MARK: - Conditional View Modifier Helper
extension View {
    @ViewBuilder
    func `if`<Transform: View>(_ condition: Bool, transform: (Self) -> Transform) -> some View {
        if condition {
            transform(self)
        } else {
            self
        }
    }
}
