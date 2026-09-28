import SwiftUI

/// Full-screen Dark Analog Clock Card with live hour, minute, and sweeping second hands,
/// inset digital pill with seconds and amber dot, and precision vector dial.
public struct TimedClockCardView: View {
    @ObservedObject public var model: TimedClockModel
    public let palette: TimedColorPalette
    public let size: CGFloat

    public init(model: TimedClockModel, palette: TimedColorPalette = .shared, size: CGFloat) {
        self.model = model
        self.palette = palette
        self.size = size
    }

    // Geometry constants scaled by card size
    private var cornerRadius: CGFloat { size * 0.220 }
    private var pillHeight: CGFloat { size * 0.076 }
    private var pillWidth: CGFloat { size * 0.320 }
    private var badgeDiameter: CGFloat { pillHeight }
    private var paddingHorizontal: CGFloat { size * 0.065 }
    private var paddingTop: CGFloat { size * 0.062 }

    // Dial center pivot coordinates relative to card
    private var pivotX: CGFloat { size * 0.500 }
    private var pivotY: CGFloat { size * 0.548 }
    private var dialRadius: CGFloat { size * 0.335 }
    private var hubRadius: CGFloat { size * 0.026 }

    public var body: some View {
        ZStack {
            // 1. Card Surface (Continuous Squircle with Matte Charcoal Obsidian Gradient)
            RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
                .fill(
                    LinearGradient(
                        colors: [palette.cardGradientTop, palette.cardGradientBottom],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
                // Ambient elevation shadows
                .shadow(color: palette.cardShadowPrimary, radius: size * 0.080, x: 0, y: size * 0.040)
                .shadow(color: palette.cardShadowSecondary, radius: size * 0.025, x: 0, y: size * 0.012)
                // Physical rim highlight
                .overlay(
                    RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
                        .stroke(
                            LinearGradient(
                                colors: [palette.cardBorderTop, palette.cardBorderBottom],
                                startPoint: .topLeading,
                                endPoint: .bottomTrailing
                            ),
                            lineWidth: 1.2
                        )
                )

            // 2. The Clock Dial Layer (Radial spokes, minute ticks, neon hour capsules, numerals)
            dialCanvas
                .frame(width: size, height: size)
                .clipShape(RoundedRectangle(cornerRadius: cornerRadius, style: .continuous))

            // 3. Clock Hands Layer (Hour hand, Minute hand, Sweeping Second hand + Center hub)
            handsLayer
                .frame(width: size, height: size)
                .clipShape(RoundedRectangle(cornerRadius: cornerRadius, style: .continuous))

            // 4. Digital Capsule Pill (Top-Left: Hours : Minutes • Seconds)
            pillView
                .position(
                    x: paddingHorizontal + pillWidth / 2.0,
                    y: paddingTop + pillHeight / 2.0
                )

            // 5. AM/PM Circular Indicator Dish (Top-Right)
            amPmBadgeView
                .position(
                    x: size - paddingHorizontal - badgeDiameter / 2.0,
                    y: paddingTop + badgeDiameter / 2.0
                )

            // 6. Perimeter Frosted Depth Vignette
            cornerVignetteOverlay
                .frame(width: size, height: size)
                .clipShape(RoundedRectangle(cornerRadius: cornerRadius, style: .continuous))
                .allowsHitTesting(false)
        }
        .frame(width: size, height: size)
    }

    // MARK: - Digital Capsule Pill View (Hours : Minutes • Seconds)
    private var pillView: some View {
        ZStack {
            // Inset recessed capsule background
            Capsule(style: .continuous)
                .fill(
                    LinearGradient(
                        colors: [palette.pillBackgroundTop, palette.pillBackgroundBottom],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
                // Inset rim highlight
                .overlay(
                    Capsule(style: .continuous)
                        .stroke(
                            LinearGradient(
                                colors: [palette.pillBorderTop, palette.pillBorderBottom],
                                startPoint: .top,
                                endPoint: .bottom
                            ),
                            lineWidth: 1.0
                        )
                )
                .shadow(
                    color: Color.black.opacity(0.50),
                    radius: size * 0.008,
                    x: 0,
                    y: size * 0.004
                )

            // Digits, Amber Dot, and Live Seconds
            HStack(spacing: size * 0.008) {
                // Hour digits (tight pair with internal slashed zero)
                HStack(spacing: size * 0.002) {
                    ForEach(Array(model.hourString.enumerated()), id: \.offset) { _, char in
                        DigitGlyph(char: char, fontSize: size * 0.034, color: palette.pillHourText, isBold: false)
                    }
                }

                // Colon separator
                Text(":")
                    .font(.system(size: size * 0.030, weight: .bold, design: .monospaced))
                    .foregroundColor(palette.pillMinuteText.opacity(0.70))

                // Minute digits (tight pair)
                HStack(spacing: size * 0.002) {
                    ForEach(Array(model.minuteString.enumerated()), id: \.offset) { _, char in
                        DigitGlyph(char: char, fontSize: size * 0.034, color: palette.pillMinuteText, isBold: true)
                    }
                }

                // Warm Amber Glowing Dot
                Circle()
                    .fill(palette.pillDot)
                    .frame(width: size * 0.012, height: size * 0.012)
                    .shadow(color: palette.pillDotGlow, radius: size * 0.008, x: 0, y: 0)
                    .padding(.horizontal, size * 0.002)

                // Live Seconds digits (vibrant neon lime)
                HStack(spacing: size * 0.002) {
                    ForEach(Array(model.secondString.enumerated()), id: \.offset) { _, char in
                        DigitGlyph(char: char, fontSize: size * 0.030, color: palette.pillSecondText, isBold: true)
                    }
                }
            }
            .padding(.horizontal, size * 0.016)
        }
        .frame(width: pillWidth, height: pillHeight)
    }

    // MARK: - AM / PM Badge View
    private var amPmBadgeView: some View {
        ZStack {
            // Concave dish background
            Circle()
                .fill(
                    LinearGradient(
                        colors: [palette.badgeBackgroundTop, palette.badgeBackgroundBottom],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
                .overlay(
                    Circle()
                        .stroke(palette.badgeBorder, lineWidth: 1.0)
                )
                .shadow(
                    color: Color.black.opacity(0.50),
                    radius: size * 0.008,
                    x: 0,
                    y: size * 0.004
                )

            Text(model.amPmLetter)
                .font(.system(size: size * 0.032, weight: .semibold, design: .default))
                .foregroundColor(palette.badgeText)
        }
        .frame(width: badgeDiameter, height: badgeDiameter)
    }

    // MARK: - Dial Canvas (Spokes, Ticks, Hour Markers, Numerals)
    private var dialCanvas: some View {
        Canvas { context, canvasSize in
            let center = CGPoint(x: pivotX, y: pivotY)
            let r = dialRadius
            let hubR = hubRadius

            // 1. Radial Spoke Hairlines from Hub toward Hour Markers
            for hour in 0..<12 {
                let hourDegrees = Double(hour) * 30.0
                let angle = Angle.degrees(hourDegrees - 90.0)

                let startPoint = CGPoint(
                    x: center.x + CGFloat(cos(angle.radians)) * (hubR + size * 0.005),
                    y: center.y + CGFloat(sin(angle.radians)) * (hubR + size * 0.005)
                )
                let endPoint = CGPoint(
                    x: center.x + CGFloat(cos(angle.radians)) * (r - size * 0.032),
                    y: center.y + CGFloat(sin(angle.radians)) * (r - size * 0.032)
                )

                var spokePath = Path()
                spokePath.move(to: startPoint)
                spokePath.addLine(to: endPoint)
                context.stroke(spokePath, with: .color(palette.spokeLine), lineWidth: 0.75)
            }

            // 2. Minute Subdivision Ticks (60 precision marks around circumference)
            for m in 0..<60 {
                let angle = Angle.degrees(Double(m) * 6.0 - 90.0)
                let isHourPosition = (m % 5 == 0)

                let tickInnerR = isHourPosition ? (r - size * 0.016) : (r - size * 0.009)
                let tickOuterR = r + size * 0.009

                let startPoint = CGPoint(
                    x: center.x + CGFloat(cos(angle.radians)) * tickInnerR,
                    y: center.y + CGFloat(sin(angle.radians)) * tickInnerR
                )
                let endPoint = CGPoint(
                    x: center.x + CGFloat(cos(angle.radians)) * tickOuterR,
                    y: center.y + CGFloat(sin(angle.radians)) * tickOuterR
                )

                var tickPath = Path()
                tickPath.move(to: startPoint)
                tickPath.addLine(to: endPoint)

                let strokeColor = isHourPosition ? palette.minuteTick.opacity(0.85) : palette.minuteTick.opacity(0.45)
                let strokeWidth = isHourPosition ? size * 0.0032 : size * 0.0020

                context.stroke(
                    tickPath,
                    with: .color(strokeColor),
                    style: StrokeStyle(lineWidth: strokeWidth, lineCap: .round)
                )
            }

            // 3. Electric Neon Lime Capsule Hour Markers (at all 12 hours)
            let markerLength = size * 0.056
            let markerWidth = size * 0.0115
            let markerRadius = markerWidth / 2.0

            for hour in 0..<12 {
                let hourDegrees = Double(hour) * 30.0
                let angle = Angle.degrees(hourDegrees - 90.0)

                let markerCenter = CGPoint(
                    x: center.x + CGFloat(cos(angle.radians)) * r,
                    y: center.y + CGFloat(sin(angle.radians)) * r
                )

                var markerContext = context
                markerContext.translateBy(x: markerCenter.x, y: markerCenter.y)
                markerContext.rotate(by: Angle.degrees(hourDegrees))

                let capsuleRect = CGRect(
                    x: -markerWidth / 2.0,
                    y: -markerLength / 2.0,
                    width: markerWidth,
                    height: markerLength
                )

                // Outer dark titanium bezel frame
                let frameRect = capsuleRect.insetBy(dx: -size * 0.0022, dy: -size * 0.0022)
                let framePath = Path(roundedRect: frameRect, cornerRadius: markerRadius + size * 0.0022)
                markerContext.fill(framePath, with: .color(palette.hourMarkerFrame))

                // Inner electric neon lime glowing core
                let innerPath = Path(roundedRect: capsuleRect, cornerRadius: markerRadius)
                let neonGradient = Gradient(colors: [palette.hourMarkerCore, palette.hourMarkerCoreGradientBottom])
                markerContext.fill(
                    innerPath,
                    with: .linearGradient(
                        neonGradient,
                        startPoint: CGPoint(x: 0, y: -markerLength / 2.0),
                        endPoint: CGPoint(x: 0, y: markerLength / 2.0)
                    )
                )
            }

            // 4. Upright Clean Hour Numerals (all 12 hours positioned just outside markers, 100% radially centered)
            let numeralRadius = r + size * 0.052

            for hour in 1...12 {
                let text = "\(hour)"
                let hourDegrees = Double(hour) * 30.0
                let angle = Angle.degrees(hourDegrees - 90.0)

                let numeralPos = CGPoint(
                    x: center.x + CGFloat(cos(angle.radians)) * numeralRadius,
                    y: center.y + CGFloat(sin(angle.radians)) * numeralRadius
                )

                let resolvedText = context.resolve(
                    Text(text)
                        .font(.system(size: size * 0.028, weight: .medium, design: .monospaced))
                        .foregroundColor(palette.hourNumeral)
                )

                context.draw(resolvedText, at: numeralPos, anchor: .center)
            }
        }
    }

    // MARK: - Hands Layer (Hour Hand, Minute Hand, Sweeping Second Hand, Center Hub)
    private var handsLayer: some View {
        ZStack {
            let centerPoint = CGPoint(x: pivotX, y: pivotY)

            // 1. Sculpted Minute Hand (Underneath hour hand)
            let minuteLength = dialRadius * 0.88
            ClockHandPointer(
                length: minuteLength,
                baseWidth: size * 0.020,
                tipWidth: size * 0.008,
                palette: palette,
                isHourHand: false,
                size: size
            )
            .rotationEffect(.degrees(model.minuteHandDegrees), anchor: .bottom)
            .position(x: centerPoint.x, y: centerPoint.y - minuteLength / 2.0)

            // 2. Sculpted Hour Hand (Signature bold spear hand)
            let hourLength = dialRadius * 0.62
            ClockHandPointer(
                length: hourLength,
                baseWidth: size * 0.028,
                tipWidth: size * 0.011,
                palette: palette,
                isHourHand: true,
                size: size
            )
            .rotationEffect(.degrees(model.hourHandDegrees), anchor: .bottom)
            .position(x: centerPoint.x, y: centerPoint.y - hourLength / 2.0)

            // 3. Live Sweeping Second Hand (Silky 60 FPS continuous needle)
            LiveSecondHand(
                dialRadius: dialRadius,
                palette: palette,
                size: size
            )
            .rotationEffect(.degrees(model.secondHandDegrees), anchor: .center)
            .position(x: centerPoint.x, y: centerPoint.y)

            // 4. Center Hub Cap (Concentric discs with neon lime center jewel)
            ZStack {
                // Outer titanium hub disc
                Circle()
                    .fill(palette.centerDiscFill)
                    .frame(width: hubRadius * 2.0, height: hubRadius * 2.0)
                    .overlay(Circle().stroke(palette.centerDiscBorder, lineWidth: 1.0))
                    .shadow(color: palette.centerDiscShadow, radius: size * 0.006, x: 0, y: size * 0.003)

                // Inner neon lime center jewel
                Circle()
                    .fill(palette.secondHandCap)
                    .frame(width: hubRadius * 0.65, height: hubRadius * 0.65)
                    .shadow(color: palette.secondHandTipGlow, radius: size * 0.006, x: 0, y: 0)
            }
            .position(x: centerPoint.x, y: centerPoint.y)
        }
    }

    // MARK: - Perimeter Frosted Vignette Overlay
    private var cornerVignetteOverlay: some View {
        Canvas { context, canvasSize in
            let trCenter = CGPoint(x: canvasSize.width, y: 0)
            let blCenter = CGPoint(x: 0, y: canvasSize.height)
            let vignetteRadius = size * 0.45

            let cornerColor = palette.cardGradientBottom

            let radialTR = Gradient(colors: [
                cornerColor.opacity(0.65),
                cornerColor.opacity(0.0)
            ])
            context.fill(
                Path(CGRect(origin: .zero, size: canvasSize)),
                with: .radialGradient(radialTR, center: trCenter, startRadius: 0, endRadius: vignetteRadius)
            )

            let radialBL = Gradient(colors: [
                cornerColor.opacity(0.55),
                cornerColor.opacity(0.0)
            ])
            context.fill(
                Path(CGRect(origin: .zero, size: canvasSize)),
                with: .radialGradient(radialBL, center: blCenter, startRadius: 0, endRadius: vignetteRadius)
            )
        }
    }
}

// MARK: - Live Second Hand Component (High Precision Needle with Neon Tip & Open Ring Counterpoise)
public struct LiveSecondHand: View {
    public let dialRadius: CGFloat
    public let palette: TimedColorPalette
    public let size: CGFloat

    public var body: some View {
        let needleLength = dialRadius * 0.96
        let tailLength = dialRadius * 0.22
        let totalHeight = (needleLength + tailLength) * 2.0

        Canvas { context, canvasSize in
            let midX = canvasSize.width / 2.0
            let centerY = canvasSize.height / 2.0

            // 1. Sleek Skeletonized Open-Ring Counterweight at tail
            let tailCenter = CGPoint(x: midX, y: centerY + tailLength * 0.72)
            let ringRadius = size * 0.0075
            let ringPath = Path(
                ellipseIn: CGRect(
                    x: tailCenter.x - ringRadius,
                    y: tailCenter.y - ringRadius,
                    width: ringRadius * 2.0,
                    height: ringRadius * 2.0
                )
            )
            context.stroke(
                ringPath,
                with: .color(palette.secondHandNeedle),
                style: StrokeStyle(lineWidth: size * 0.0020)
            )

            // 2. Needle body (titanium silver) from tail to tip start
            var needlePath = Path()
            needlePath.move(to: CGPoint(x: midX, y: centerY + tailLength))
            needlePath.addLine(to: CGPoint(x: midX, y: centerY - needleLength * 0.65))
            context.stroke(
                needlePath,
                with: .color(palette.secondHandNeedle),
                style: StrokeStyle(lineWidth: size * 0.0024, lineCap: .round)
            )

            // 3. Vibrant Neon Lime tip section with intense tip glow
            var tipPath = Path()
            tipPath.move(to: CGPoint(x: midX, y: centerY - needleLength * 0.65))
            tipPath.addLine(to: CGPoint(x: midX, y: centerY - needleLength))
            context.stroke(
                tipPath,
                with: .color(palette.secondHandTip),
                style: StrokeStyle(lineWidth: size * 0.0032, lineCap: .round)
            )
        }
        .frame(width: size * 0.06, height: totalHeight)
        .shadow(color: Color.black.opacity(0.40), radius: size * 0.006, x: size * 0.002, y: size * 0.003)
    }
}

// MARK: - Sculpted Tapered Hand (Hour / Minute) with Recessed Neon Spine
public struct ClockHandPointer: View {
    public let length: CGFloat
    public let baseWidth: CGFloat
    public let tipWidth: CGFloat
    public let palette: TimedColorPalette
    public let isHourHand: Bool
    public let size: CGFloat

    public var body: some View {
        ZStack {
            // Hand body path (sculpted tapered pointer with pointed spear tip)
            HandShape(baseWidth: baseWidth, tipWidth: tipWidth, length: length)
                .fill(
                    LinearGradient(
                        colors: [palette.handBodyFillTop, palette.handBodyFillBottom],
                        startPoint: .top,
                        endPoint: .bottom
                    )
                )
                .overlay(
                    HandShape(baseWidth: baseWidth, tipWidth: tipWidth, length: length)
                        .stroke(palette.handBodyBorder, lineWidth: 0.9)
                )
                .shadow(
                    color: palette.handBodyShadow,
                    radius: size * 0.016,
                    x: size * 0.004,
                    y: size * 0.008
                )

            // Recessed Central Neon Lime Groove
            let grooveWidth = baseWidth * 0.22
            let grooveLength = length * 0.54
            Capsule(style: .continuous)
                .fill(palette.handNeonGroove)
                .frame(width: grooveWidth, height: grooveLength)
                .shadow(color: palette.handNeonGlow, radius: size * 0.010, x: 0, y: 0)
                .offset(y: -length * 0.06)
        }
        .frame(width: max(baseWidth, tipWidth) * 2.0, height: length)
    }
}

// MARK: - Custom Shape for the tapered hand with pointed spear tip
public struct HandShape: Shape {
    public let baseWidth: CGFloat
    public let tipWidth: CGFloat
    public let length: CGFloat

    public func path(in rect: CGRect) -> Path {
        var path = Path()
        let midX = rect.midX
        let bottomY = rect.maxY
        let topY = rect.minY

        let halfBase = baseWidth / 2.0
        let halfTip = tipWidth / 2.0
        let tipPointHeight = tipWidth * 1.35

        path.move(to: CGPoint(x: midX - halfBase, y: bottomY))
        path.addLine(to: CGPoint(x: midX - halfTip, y: topY + tipPointHeight))
        path.addLine(to: CGPoint(x: midX, y: topY))
        path.addLine(to: CGPoint(x: midX + halfTip, y: topY + tipPointHeight))
        path.addLine(to: CGPoint(x: midX + halfBase, y: bottomY))
        path.addArc(
            center: CGPoint(x: midX, y: bottomY),
            radius: halfBase,
            startAngle: .degrees(0),
            endAngle: .degrees(180),
            clockwise: false
        )
        path.closeSubpath()

        return path
    }
}

// MARK: - Crisp Digit Glyph with precise internal slashed zero
public struct DigitGlyph: View {
    public let char: Character
    public let fontSize: CGFloat
    public let color: Color
    public let isBold: Bool

    public var body: some View {
        ZStack {
            Text(String(char))
                .font(.system(size: fontSize, weight: isBold ? .bold : .medium, design: .default))
                .foregroundColor(color)

            if char == "0" {
                SlashOverlay(fontSize: fontSize, color: color)
            }
        }
        .frame(width: fontSize * 0.58, height: fontSize * 1.1)
    }
}

public struct SlashOverlay: View {
    public let fontSize: CGFloat
    public let color: Color

    public var body: some View {
        Canvas { context, size in
            let w = size.width
            let h = size.height
            var path = Path()
            path.move(to: CGPoint(x: w * 0.76, y: h * 0.28))
            path.addLine(to: CGPoint(x: w * 0.24, y: h * 0.72))

            context.stroke(
                path,
                with: .color(color.opacity(0.90)),
                style: StrokeStyle(lineWidth: max(1.2, fontSize * 0.052), lineCap: .round)
            )
        }
        .allowsHitTesting(false)
    }
}
