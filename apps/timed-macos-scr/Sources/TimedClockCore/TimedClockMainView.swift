import SwiftUI

/// Full-Screen Realistic 3D Watch Face macOS Screen Saver
/// Features authentic horological proportions, multi-tier brushed titanium bezel,
/// angled rehaut with ambient occlusion, pristine symmetrical obsidian dial,
/// 3D faceted hands with multi-level depth shadows, Super-LumiNova glowing batons,
/// precision minute chapter ring, curvature-aligned Swiss typography,
/// and external floating digital clock HUD outside the ring at top-right.
public struct TimedClockMainView: View {
    @ObservedObject public var model: TimedClockModel
    public let palette: TimedColorPalette

    public init(model: TimedClockModel = .shared, palette: TimedColorPalette = .shared) {
        self.model = model
        self.palette = palette
    }

    public var body: some View {
        GeometryReader { proxy in
            let w = proxy.size.width
            let h = proxy.size.height

            // The watch itself is FULL SCREEN:
            // Sized to commandingly occupy 97% of the screen's smaller dimension.
            let minDim = min(w, h)
            let center = CGPoint(x: w / 2.0, y: h / 2.0)

            // Watch case & bezel geometry
            let caseRadius = minDim * 0.485
            let bezelOuterRadius = caseRadius * 0.990
            let bezelInnerRadius = caseRadius * 0.925
            let rehautRadius = caseRadius * 0.865
            let dialRadius = rehautRadius
            let hubRadius = dialRadius * 0.080

            ZStack {
                // 1. Full Screen Deep OLED Obsidian Backdrop
                LinearGradient(
                    colors: [palette.cardGradientBottom, Color.black],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )
                .ignoresSafeArea()

                // 2. Corner Studio Vignettes
                studioCornerVignettes(width: w, height: h, minDim: minDim)
                    .ignoresSafeArea()
                    .allowsHitTesting(false)

                // 3. 3D Watch Case & Brushed Titanium Bezel
                watchCaseAndBezel(
                    center: center,
                    caseRadius: caseRadius,
                    bezelOuterRadius: bezelOuterRadius,
                    bezelInnerRadius: bezelInnerRadius,
                    rehautRadius: rehautRadius
                )

                // 4. Dial Surface Plate (Pure Symmetrical Obsidian Radial Gradient)
                dialPlate(
                    center: center,
                    dialRadius: dialRadius
                )

                // 5. Applied 3D Neon Capsule Indices & Curvature-Aligned Numerals (All 12 hours)
                dialMarkersAndNumerals(center: center, dialRadius: dialRadius, hubRadius: hubRadius)

                // 6. 3D Faceted Hands Layer (Hour hand, Minute hand, Sweeping Second hand, Center Hub)
                hands3DLayer(center: center, dialRadius: dialRadius, hubRadius: hubRadius)

                // 7. Curved Sapphire Crystal Glass Reflection Sheen (Silky, seamless AR coating)
                sapphireCrystalReflection(center: center, dialRadius: dialRadius)
                    .allowsHitTesting(false)

                // 8. Outside the Ring: Floating Digital Clock HUD at Top-Right
                topRightDigitalClock(width: w, height: h, minDim: minDim)
                    .allowsHitTesting(false)
            }
            .frame(width: w, height: h)
        }
        .ignoresSafeArea()
    }

    // MARK: - 1. Studio Corner Vignettes
    private func studioCornerVignettes(width: CGFloat, height: CGFloat, minDim: CGFloat) -> some View {
        Canvas { context, size in
            let vigRadius = minDim * 0.75
            let grad = Gradient(colors: [Color.black.opacity(0.88), Color.clear])
            context.fill(Path(CGRect(origin: .zero, size: size)), with: .radialGradient(grad, center: CGPoint(x: 0, y: 0), startRadius: 0, endRadius: vigRadius))
            context.fill(Path(CGRect(origin: .zero, size: size)), with: .radialGradient(grad, center: CGPoint(x: size.width, y: 0), startRadius: 0, endRadius: vigRadius))
            context.fill(Path(CGRect(origin: .zero, size: size)), with: .radialGradient(grad, center: CGPoint(x: 0, y: size.height), startRadius: 0, endRadius: vigRadius))
            context.fill(Path(CGRect(origin: .zero, size: size)), with: .radialGradient(grad, center: CGPoint(x: size.width, y: size.height), startRadius: 0, endRadius: vigRadius))
        }
    }

    // MARK: - 2. 3D Watch Case & Brushed Titanium Bezel
    private func watchCaseAndBezel(
        center: CGPoint,
        caseRadius: CGFloat,
        bezelOuterRadius: CGFloat,
        bezelInnerRadius: CGFloat,
        rehautRadius: CGFloat
    ) -> some View {
        ZStack {
            // Case Ambient Elevation Shadow onto OLED Backdrop
            Circle()
                .fill(Color.black.opacity(0.92))
                .frame(width: caseRadius * 2.0, height: caseRadius * 2.0)
                .shadow(color: Color.black.opacity(0.95), radius: caseRadius * 0.16, x: 0, y: caseRadius * 0.08)
                .position(x: center.x, y: center.y)

            // Outer Brushed Titanium Bezel Ring with Directional 135° Lighting
            Circle()
                .fill(
                    LinearGradient(
                        colors: [
                            Color(red: 0.30, green: 0.32, blue: 0.37), // Top-left specular sheen
                            Color(red: 0.19, green: 0.20, blue: 0.23),
                            Color(red: 0.12, green: 0.13, blue: 0.15),
                            Color(red: 0.07, green: 0.08, blue: 0.09)  // Bottom-right shadow
                        ],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
                .frame(width: bezelOuterRadius * 2.0, height: bezelOuterRadius * 2.0)
                // Diamond-cut outer chamfered edge highlight
                .overlay(
                    Circle().stroke(
                        LinearGradient(
                            colors: [Color.white.opacity(0.45), Color.white.opacity(0.05)],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        ),
                        lineWidth: 1.5
                    )
                )
                .position(x: center.x, y: center.y)

            // Inner Stepped Bezel Groove
            Circle()
                .fill(
                    LinearGradient(
                        colors: [
                            Color(red: 0.07, green: 0.08, blue: 0.09),
                            Color(red: 0.16, green: 0.17, blue: 0.20)
                        ],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
                .frame(width: bezelInnerRadius * 2.0, height: bezelInnerRadius * 2.0)
                .position(x: center.x, y: center.y)

            // 3D Angled Rehaut Wall (Conical Inner Wall Casting Ambient Occlusion Shadow onto Dial)
            Circle()
                .fill(
                    LinearGradient(
                        colors: [
                            Color.black.opacity(0.88), // Deep shadow under top-left rehaut
                            Color(red: 0.09, green: 0.10, blue: 0.12),
                            Color(red: 0.16, green: 0.17, blue: 0.20)  // Reflected rim light on bottom-right
                        ],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
                .frame(width: rehautRadius * 2.0, height: rehautRadius * 2.0)
                .overlay(
                    Circle().stroke(
                        LinearGradient(
                            colors: [Color.black.opacity(0.92), Color.white.opacity(0.20)],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        ),
                        lineWidth: 1.8
                    )
                )
                .position(x: center.x, y: center.y)
        }
    }

    // MARK: - 3. Dial Plate (Pure Symmetrical Obsidian Dial)
    private func dialPlate(
        center: CGPoint,
        dialRadius: CGFloat
    ) -> some View {
        Circle()
            .fill(
                RadialGradient(
                    colors: [
                        Color(red: 0.125, green: 0.130, blue: 0.145), // Subtle upper center highlight
                        Color(red: 0.082, green: 0.085, blue: 0.095),
                        Color(red: 0.040, green: 0.042, blue: 0.048)  // Deep perimeter rim
                    ],
                    center: UnitPoint(x: 0.42, y: 0.38),
                    startRadius: 0,
                    endRadius: dialRadius * 1.02
                )
            )
            .frame(width: dialRadius * 2.0, height: dialRadius * 2.0)
            .position(x: center.x, y: center.y)
    }

    // MARK: - 4. Applied 3D Indices & Curvature-Aligned Swiss Numerals
    private func dialMarkersAndNumerals(center: CGPoint, dialRadius: CGFloat, hubRadius: CGFloat) -> some View {
        Canvas { context, canvasSize in
            let dialR = dialRadius
            let hubR = hubRadius

            // Tier 1: Radial Spoke Hairlines milled into the dial surface (All 12 hours)
            for hour in 0..<12 {
                let hourDegrees = Double(hour) * 30.0
                let angle = Angle.degrees(hourDegrees - 90.0)

                let startPoint = CGPoint(
                    x: center.x + CGFloat(cos(angle.radians)) * (hubR + dialR * 0.020),
                    y: center.y + CGFloat(sin(angle.radians)) * (hubR + dialR * 0.020)
                )
                let endPoint = CGPoint(
                    x: center.x + CGFloat(cos(angle.radians)) * (dialR * 0.730),
                    y: center.y + CGFloat(sin(angle.radians)) * (dialR * 0.730)
                )

                var spokePath = Path()
                spokePath.move(to: startPoint)
                spokePath.addLine(to: endPoint)
                context.stroke(spokePath, with: .color(palette.spokeLine), lineWidth: 0.9)
            }

            // Tier 2: 3D Applied Super-LumiNova Neon Capsule Hour Indices (All 12 hours)
            let batonCenterRadius = dialR * 0.635
            let markerLength = dialR * 0.125
            let markerWidth = dialR * 0.026
            let markerRadius = markerWidth / 2.0

            for hour in 0..<12 {
                let hourDegrees = Double(hour) * 30.0
                let angle = Angle.degrees(hourDegrees - 90.0)

                let markerCenter = CGPoint(
                    x: center.x + CGFloat(cos(angle.radians)) * batonCenterRadius,
                    y: center.y + CGFloat(sin(angle.radians)) * batonCenterRadius
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

                // 3D Metallic Bezel Frame (Polished chamfer)
                let frameRect = capsuleRect.insetBy(dx: -dialR * 0.005, dy: -dialR * 0.005)
                let framePath = Path(roundedRect: frameRect, cornerRadius: markerRadius + dialR * 0.005)
                markerContext.fill(framePath, with: .color(Color(red: 0.16, green: 0.17, blue: 0.20)))
                markerContext.stroke(framePath, with: .color(Color.white.opacity(0.30)), lineWidth: 0.9)

                // Ambient Super-LumiNova Glow Aura onto surrounding dial
                let glowRect = capsuleRect.insetBy(dx: -dialR * 0.010, dy: -dialR * 0.010)
                let glowPath = Path(roundedRect: glowRect, cornerRadius: markerRadius + dialR * 0.010)
                markerContext.fill(glowPath, with: .color(Color(red: 0.55, green: 0.95, blue: 0.15, opacity: 0.25)))

                // Glowing Super-LumiNova Core
                let innerPath = Path(roundedRect: capsuleRect, cornerRadius: markerRadius)
                let neonGradient = Gradient(colors: [
                    Color(red: 0.65, green: 1.0, blue: 0.22),
                    Color(red: 0.48, green: 0.92, blue: 0.12)
                ])
                markerContext.fill(
                    innerPath,
                    with: .linearGradient(
                        neonGradient,
                        startPoint: CGPoint(x: 0, y: -markerLength / 2.0),
                        endPoint: CGPoint(x: 0, y: markerLength / 2.0)
                    )
                )
            }

            // Tier 3: 60 Precision Minute Subdivision Ticks along Chapter Ring
            for m in 0..<60 {
                let angle = Angle.degrees(Double(m) * 6.0 - 90.0)
                let isHourPosition = (m % 5 == 0)

                let tickInnerR = isHourPosition ? (dialR * 0.740) : (dialR * 0.760)
                let tickOuterR = dialR * 0.790

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

                let strokeColor = isHourPosition ? Color(red: 0.55, green: 0.58, blue: 0.65) : Color(red: 0.30, green: 0.32, blue: 0.38)
                let strokeWidth = isHourPosition ? dialR * 0.008 : dialR * 0.0048

                context.stroke(
                    tickPath,
                    with: .color(strokeColor),
                    style: StrokeStyle(lineWidth: strokeWidth, lineCap: .round)
                )
            }

            // Tier 4: Precision Swiss Numerals (Curvature-Aligned SF Pro Display)
            let numeralRadius = dialR * 0.860
            let numFontSize = dialR * 0.062

            for hour in 1...12 {
                let text = "\(hour)"
                let hourDegrees = Double(hour) * 30.0
                let angle = Angle.degrees(hourDegrees - 90.0)

                let numeralPos = CGPoint(
                    x: center.x + CGFloat(cos(angle.radians)) * numeralRadius,
                    y: center.y + CGFloat(sin(angle.radians)) * numeralRadius
                )

                var numContext = context
                numContext.translateBy(x: numeralPos.x, y: numeralPos.y)

                var rotAngle: Double = 0.0
                if hour == 12 || hour == 1 || hour == 2 || hour == 10 || hour == 11 {
                    rotAngle = hourDegrees <= 180 ? hourDegrees : hourDegrees - 360
                } else if hour == 4 || hour == 5 || hour == 6 || hour == 7 || hour == 8 {
                    rotAngle = hourDegrees - 180.0
                } else if hour == 3 {
                    rotAngle = 90.0
                } else if hour == 9 {
                    rotAngle = -90.0
                }
                numContext.rotate(by: .degrees(rotAngle))

                let resolvedText = numContext.resolve(
                    Text(text)
                        .font(.system(size: numFontSize, weight: .medium, design: .default).monospacedDigit())
                        .foregroundColor(Color(red: 0.80, green: 0.83, blue: 0.90))
                )

                numContext.draw(resolvedText, at: .zero, anchor: .center)
            }
        }
    }

    // MARK: - 5. 3D Faceted Hands Layer with Multi-Level Depth Shadows
    private func hands3DLayer(center: CGPoint, dialRadius: CGFloat, hubRadius: CGFloat) -> some View {
        ZStack {
            // --- Plane 1: Sculpted 3D Hour Hand (Lowest plane above dial) ---
            let hourLength = dialRadius * 0.58
            FacetedClockHand(
                length: hourLength,
                baseWidth: dialRadius * 0.076,
                tipWidth: dialRadius * 0.026,
                palette: palette,
                dialRadius: dialRadius
            )
            .shadow(color: Color.black.opacity(0.85), radius: dialRadius * 0.028, x: dialRadius * 0.012, y: dialRadius * 0.018)
            .rotationEffect(.degrees(model.hourHandDegrees), anchor: .bottom)
            .position(x: center.x, y: center.y - hourLength / 2.0)

            // --- Plane 2: Sculpted 3D Minute Hand (Higher plane, longer cast shadow) ---
            let minuteLength = dialRadius * 0.82
            FacetedClockHand(
                length: minuteLength,
                baseWidth: dialRadius * 0.052,
                tipWidth: dialRadius * 0.018,
                palette: palette,
                dialRadius: dialRadius
            )
            .shadow(color: Color.black.opacity(0.75), radius: dialRadius * 0.040, x: dialRadius * 0.018, y: dialRadius * 0.026)
            .rotationEffect(.degrees(model.minuteHandDegrees), anchor: .bottom)
            .position(x: center.x, y: center.y - minuteLength / 2.0)

            // --- Plane 3: Live Sweeping Seconds Hand (Highest plane) ---
            LiveSecondHand3D(
                dialRadius: dialRadius,
                palette: palette
            )
            .shadow(color: Color.black.opacity(0.65), radius: dialRadius * 0.030, x: dialRadius * 0.024, y: dialRadius * 0.034)
            .rotationEffect(.degrees(model.secondHandDegrees), anchor: .center)
            .position(x: center.x, y: center.y)

            // --- Multi-Tier 3D Center Pinion Cap ---
            centerPinion3D(hubRadius: hubRadius, dialRadius: dialRadius)
                .position(x: center.x, y: center.y)
        }
    }

    // MARK: - Multi-Tier 3D Center Pinion Cap
    private func centerPinion3D(hubRadius: CGFloat, dialRadius: CGFloat) -> some View {
        ZStack {
            // Tier 1: Base Turned Metal Ring
            Circle()
                .fill(
                    LinearGradient(
                        colors: [Color(red: 0.32, green: 0.34, blue: 0.38), Color(red: 0.12, green: 0.13, blue: 0.15)],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
                .frame(width: hubRadius * 2.0, height: hubRadius * 2.0)
                .overlay(Circle().stroke(Color.white.opacity(0.35), lineWidth: 1.0))
                .shadow(color: Color.black.opacity(0.85), radius: dialRadius * 0.025, x: dialRadius * 0.008, y: dialRadius * 0.012)

            // Tier 2: Chamfered Polished Bevel
            Circle()
                .fill(Color(red: 0.18, green: 0.19, blue: 0.22))
                .frame(width: hubRadius * 1.35, height: hubRadius * 1.35)

            // Tier 3: Domed Super-LumiNova Neon Jewel Cap
            Circle()
                .fill(
                    RadialGradient(
                        colors: [Color(red: 0.70, green: 1.0, blue: 0.25), Color(red: 0.45, green: 0.85, blue: 0.12)],
                        center: UnitPoint(x: 0.35, y: 0.35),
                        startRadius: 0,
                        endRadius: hubRadius * 0.75
                    )
                )
                .frame(width: hubRadius * 0.75, height: hubRadius * 0.75)
                .shadow(color: palette.secondHandTipGlow, radius: dialRadius * 0.015, x: 0, y: 0)
        }
    }

    // MARK: - 7. Curved Sapphire Crystal Reflection (Seamless Optical Anti-Reflective Sheen)
    private func sapphireCrystalReflection(center: CGPoint, dialRadius: CGFloat) -> some View {
        ZStack {
            // Optical anti-reflective coating sheen across entire crystal dome (zero harsh cuts)
            Circle()
                .fill(
                    LinearGradient(
                        stops: [
                            .init(color: Color.white.opacity(0.045), location: 0.0),
                            .init(color: Color.cyan.opacity(0.020), location: 0.22),
                            .init(color: Color.clear, location: 0.52),
                            .init(color: Color.white.opacity(0.015), location: 0.82),
                            .init(color: Color.clear, location: 1.0)
                        ],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
                .frame(width: dialRadius * 2.0, height: dialRadius * 2.0)
                .position(x: center.x, y: center.y)

            // Ultra-subtle upper rim specular highlight
            Circle()
                .stroke(
                    LinearGradient(
                        stops: [
                            .init(color: Color.white.opacity(0.35), location: 0.0),
                            .init(color: Color.cyan.opacity(0.15), location: 0.15),
                            .init(color: Color.clear, location: 0.38)
                        ],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    ),
                    lineWidth: 1.2
                )
                .frame(width: dialRadius * 1.99, height: dialRadius * 1.99)
                .position(x: center.x, y: center.y)
        }
    }

    // MARK: - 8. Top-Right Floating Digital Clock HUD (Outside the Ring)
    private func topRightDigitalClock(width: CGFloat, height: CGFloat, minDim: CGFloat) -> some View {
        let pillHeight = max(40.0, minDim * 0.052)

        return VStack {
            HStack {
                Spacer()

                HStack(spacing: pillHeight * 0.20) {
                    // Floating Digital Time Capsule Pill
                    HStack(spacing: pillHeight * 0.065) {
                        // Hours with Slashed Zero
                        HStack(spacing: pillHeight * 0.02) {
                            ForEach(Array(model.hourString.enumerated()), id: \.offset) { _, char in
                                DigitGlyph(char: char, fontSize: pillHeight * 0.50, color: Color(red: 0.95, green: 0.96, blue: 0.98), isBold: false)
                            }
                        }

                        // Colon
                        Text(":")
                            .font(.system(size: pillHeight * 0.44, weight: .bold, design: .rounded))
                            .foregroundColor(Color(red: 0.55, green: 0.58, blue: 0.65))

                        // Minutes with Slashed Zero
                        HStack(spacing: pillHeight * 0.02) {
                            ForEach(Array(model.minuteString.enumerated()), id: \.offset) { _, char in
                                DigitGlyph(char: char, fontSize: pillHeight * 0.50, color: Color(red: 0.72, green: 0.75, blue: 0.82), isBold: true)
                            }
                        }

                        // Glowing Amber Dot
                        Circle()
                            .fill(palette.pillDot)
                            .frame(width: pillHeight * 0.18, height: pillHeight * 0.18)
                            .shadow(color: palette.pillDotGlow, radius: pillHeight * 0.16, x: 0, y: 0)
                            .padding(.horizontal, pillHeight * 0.03)

                        // Live Seconds in Electric Neon Lime
                        HStack(spacing: pillHeight * 0.02) {
                            ForEach(Array(model.secondString.enumerated()), id: \.offset) { _, char in
                                DigitGlyph(char: char, fontSize: pillHeight * 0.48, color: palette.pillSecondText, isBold: true)
                            }
                        }
                    }
                    .padding(.horizontal, pillHeight * 0.40)
                    .frame(height: pillHeight)
                    .background(
                        Capsule(style: .continuous)
                            .fill(
                                LinearGradient(
                                    colors: [
                                        Color(red: 0.13, green: 0.14, blue: 0.16),
                                        Color(red: 0.06, green: 0.065, blue: 0.075)
                                    ],
                                    startPoint: .topLeading,
                                    endPoint: .bottomTrailing
                                )
                            )
                    )
                    .overlay(
                        Capsule(style: .continuous)
                            .stroke(
                                LinearGradient(
                                    colors: [Color.white.opacity(0.24), Color.white.opacity(0.04)],
                                    startPoint: .topLeading,
                                    endPoint: .bottomTrailing
                                ),
                                lineWidth: 1.0
                            )
                    )
                    .shadow(color: Color.black.opacity(0.65), radius: pillHeight * 0.35, x: 0, y: pillHeight * 0.15)

                    // Companion AM/PM Circular Indicator Dish
                    ZStack {
                        Circle()
                            .fill(
                                LinearGradient(
                                    colors: [
                                        Color(red: 0.13, green: 0.14, blue: 0.16),
                                        Color(red: 0.06, green: 0.065, blue: 0.075)
                                    ],
                                    startPoint: .topLeading,
                                    endPoint: .bottomTrailing
                                )
                            )
                            .overlay(
                                Circle()
                                    .stroke(
                                        LinearGradient(
                                            colors: [Color.white.opacity(0.24), Color.white.opacity(0.04)],
                                            startPoint: .topLeading,
                                            endPoint: .bottomTrailing
                                        ),
                                        lineWidth: 1.0
                                    )
                            )
                            .shadow(color: Color.black.opacity(0.65), radius: pillHeight * 0.35, x: 0, y: pillHeight * 0.15)

                        Text(model.amPmLetter)
                            .font(.system(size: pillHeight * 0.48, weight: .medium, design: .default))
                            .foregroundColor(Color(red: 0.70, green: 0.74, blue: 0.82))
                    }
                    .frame(width: pillHeight, height: pillHeight)
                }
                .padding(.top, max(28, height * 0.042))
                .padding(.trailing, max(36, width * 0.042))
            }

            Spacer()
        }
    }
}

// MARK: - 3D Faceted Clock Hand Component (Illuminated & Shaded Longitudinal Flanks)
public struct FacetedClockHand: View {
    public let length: CGFloat
    public let baseWidth: CGFloat
    public let tipWidth: CGFloat
    public let palette: TimedColorPalette
    public let dialRadius: CGFloat

    public var body: some View {
        ZStack {
            // 3D Split Faceted Hand Body:
            // Left flank illuminated by 135° light source, right flank in shadow
            HStack(spacing: 0) {
                // Left Illuminated Flank
                HalfHandShape(baseHalf: baseWidth / 2.0, tipHalf: tipWidth / 2.0, length: length, isLeft: true)
                    .fill(
                        LinearGradient(
                            colors: [Color(red: 0.32, green: 0.34, blue: 0.38), Color(red: 0.20, green: 0.21, blue: 0.24)],
                            startPoint: .top,
                            endPoint: .bottom
                        )
                    )
                    .frame(width: baseWidth / 2.0, height: length)

                // Right Shaded Flank
                HalfHandShape(baseHalf: baseWidth / 2.0, tipHalf: tipWidth / 2.0, length: length, isLeft: false)
                    .fill(
                        LinearGradient(
                            colors: [Color(red: 0.15, green: 0.16, blue: 0.18), Color(red: 0.08, green: 0.09, blue: 0.10)],
                            startPoint: .top,
                            endPoint: .bottom
                        )
                    )
                    .frame(width: baseWidth / 2.0, height: length)
            }
            .frame(width: baseWidth, height: length)

            // Center Razor Ridge Line
            Rectangle()
                .fill(Color.white.opacity(0.35))
                .frame(width: 0.9, height: length * 0.88)
                .offset(y: -length * 0.04)

            // Recessed Luminous Super-LumiNova Lime Channel
            let grooveWidth = baseWidth * 0.22
            let grooveLength = length * 0.56
            Capsule(style: .continuous)
                .fill(
                    LinearGradient(
                        colors: [Color(red: 0.65, green: 1.0, blue: 0.22), Color(red: 0.48, green: 0.90, blue: 0.12)],
                        startPoint: .top,
                        endPoint: .bottom
                    )
                )
                .frame(width: grooveWidth, height: grooveLength)
                .shadow(color: palette.handNeonGlow, radius: dialRadius * 0.016, x: 0, y: 0)
                .offset(y: -length * 0.06)
        }
        .frame(width: baseWidth * 2.0, height: length)
    }
}

// MARK: - Longitudinal Half Hand Shape for 3D Facet Rendering
public struct HalfHandShape: Shape {
    public let baseHalf: CGFloat
    public let tipHalf: CGFloat
    public let length: CGFloat
    public let isLeft: Bool

    public func path(in rect: CGRect) -> Path {
        var path = Path()
        let tipPointHeight = tipHalf * 1.5

        if isLeft {
            path.move(to: CGPoint(x: rect.maxX, y: rect.maxY))
            path.addLine(to: CGPoint(x: rect.minX, y: rect.maxY))
            path.addLine(to: CGPoint(x: rect.maxX - tipHalf, y: rect.minY + tipPointHeight))
            path.addLine(to: CGPoint(x: rect.maxX, y: rect.minY))
            path.closeSubpath()
        } else {
            path.move(to: CGPoint(x: rect.minX, y: rect.maxY))
            path.addLine(to: CGPoint(x: rect.maxX, y: rect.maxY))
            path.addLine(to: CGPoint(x: rect.minX + tipHalf, y: rect.minY + tipPointHeight))
            path.addLine(to: CGPoint(x: rect.minX, y: rect.minY))
            path.closeSubpath()
        }

        return path
    }
}

// MARK: - 3D Live Sweeping Seconds Hand (High Precision Floating Needle)
public struct LiveSecondHand3D: View {
    public let dialRadius: CGFloat
    public let palette: TimedColorPalette

    public var body: some View {
        let needleLength = dialRadius * 0.94
        let tailLength = dialRadius * 0.22
        let totalHeight = (needleLength + tailLength) * 2.0

        Canvas { context, canvasSize in
            let midX = canvasSize.width / 2.0
            let centerY = canvasSize.height / 2.0

            // 1. Polished Chamfered Skeletonized Open-Ring Counterweight
            let tailCenter = CGPoint(x: midX, y: centerY + tailLength * 0.72)
            let ringRadius = dialRadius * 0.024
            let ringPath = Path(
                ellipseIn: CGRect(
                    x: tailCenter.x - ringRadius,
                    y: tailCenter.y - ringRadius,
                    width: ringRadius * 2.0,
                    height: ringRadius * 2.0
                )
            )
            context.stroke(ringPath, with: .color(Color(red: 0.85, green: 0.88, blue: 0.94)), style: StrokeStyle(lineWidth: dialRadius * 0.006))

            // 2. Titanium Silver Needle Shank
            var needlePath = Path()
            needlePath.move(to: CGPoint(x: midX, y: centerY + tailLength))
            needlePath.addLine(to: CGPoint(x: midX, y: centerY - needleLength * 0.65))
            context.stroke(
                needlePath,
                with: .color(Color(red: 0.82, green: 0.85, blue: 0.90)),
                style: StrokeStyle(lineWidth: dialRadius * 0.007, lineCap: .round)
            )

            // 3. Vibrant Neon Lime Active Needle Tip
            var tipPath = Path()
            tipPath.move(to: CGPoint(x: midX, y: centerY - needleLength * 0.65))
            tipPath.addLine(to: CGPoint(x: midX, y: centerY - needleLength))
            context.stroke(
                tipPath,
                with: .color(Color(red: 0.60, green: 1.0, blue: 0.18)),
                style: StrokeStyle(lineWidth: dialRadius * 0.009, lineCap: .round)
            )
        }
        .frame(width: dialRadius * 0.16, height: totalHeight)
    }
}
