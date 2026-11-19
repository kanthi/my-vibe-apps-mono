import SwiftUI

/// A full-screen, dark, 3D-lit dial. The dial surface (sunburst, matte or guilloché,
/// with a corner vignette) covers the whole screen edge to edge; markers, numerals,
/// complications and hands are sized to the largest circle that fits, so it adapts to
/// any screen size and orientation.
///
/// All shading comes from one light source at the top left. Facets of the hands and
/// markers are lit by their current angle, and every cast shadow falls the same way on
/// screen, so the hands look raised above the dial as they turn.
///
/// Two layers: the static dial (Equatable, drawn once per size) and the moving parts
/// (hands, sub-dial hands, date, moon), which redraw at 60 FPS.
public struct RealisticWatchView: View {
    public let model: TimedClockModel
    public let theme: RealisticWatchTheme

    public init(model: TimedClockModel = .shared, theme: RealisticWatchTheme) {
        self.model = model
        self.theme = theme
    }

    public var body: some View {
        GeometryReader { proxy in
            let size = proxy.size
            ZStack {
                theme.dialEdge.color()
                WatchDialLayer(theme: theme, size: size).equatable()
                WatchHandsLayer(model: model, theme: theme, size: size)
            }
            .frame(width: size.width, height: size.height)
        }
        .ignoresSafeArea()
    }
}

// MARK: - Layers

struct WatchDialLayer: View, Equatable {
    let theme: RealisticWatchTheme
    let size: CGSize

    nonisolated static func == (a: WatchDialLayer, b: WatchDialLayer) -> Bool {
        a.theme == b.theme && a.size == b.size
    }

    var body: some View {
        let painter = WatchPainter(theme: theme, size: size)
        Canvas { ctx, _ in
            painter.drawDial(ctx)
        }
        .frame(width: size.width, height: size.height)
        .allowsHitTesting(false)
    }
}

struct WatchHandsLayer: View {
    @ObservedObject var model: TimedClockModel
    let theme: RealisticWatchTheme
    let size: CGSize

    var body: some View {
        let painter = WatchPainter(theme: theme, size: size)
        let time = WatchTime(
            hour: model.hourHandDegrees,
            minute: model.minuteHandDegrees,
            second: model.secondHandDegrees,
            date: model.currentDate
        )
        Canvas { ctx, _ in
            painter.drawMoving(ctx, time: time)
        }
        .frame(width: size.width, height: size.height)
        .allowsHitTesting(false)
    }
}

/// Hand angles (clock degrees) plus the date, for complications.
struct WatchTime {
    var hour: Double
    var minute: Double
    var second: Double
    var date: Date
}

// MARK: - Lighting

/// One light at the top left. `(x, y)` points from the dial toward the light, y down.
enum WatchLight {
    static let x: Double = -0.6
    static let y: Double = -0.8

    /// Color of one long facet of a radial part (marker or hand) pointing at `clockDeg`
    /// (0 = 12 o'clock, clockwise). The left facet faces left of the pointing direction.
    static func facet(_ light: RGB, _ dark: RGB, clockDeg: Double, left: Bool) -> RGB {
        let a = clockDeg * .pi / 180
        let nx = left ? -cos(a) : cos(a)
        let ny = left ? -sin(a) : sin(a)
        let d = nx * x + ny * y
        return RGB.mix(dark, light, 0.5 + 0.5 * d)
    }

    /// Conic gradient for a small domed or beveled round metal part:
    /// bright toward the light, a sharp specular, and a faint bounce opposite.
    /// `inward` flips it for a surface that slopes toward the center (a recess).
    static func metal(_ light: RGB, _ dark: RGB, gloss: Double = 0.6, inward: Bool = false, steps: Int = 96) -> Gradient {
        var stops: [Gradient.Stop] = []
        stops.reserveCapacity(steps + 1)
        for k in 0...steps {
            let s = Double(k) / Double(steps)
            let a = s * 2 * .pi
            var d = cos(a) * x + sin(a) * y
            if inward { d = -d }
            let base = 0.5 + 0.5 * d
            let spec = pow(max(0, d), 16) * gloss
            let bounce = pow(max(0, -d), 6) * 0.22 * gloss
            let t = min(1, base * 0.82 + spec + bounce)
            stops.append(.init(color: RGB.mix(dark, light, t).color(), location: s))
        }
        return Gradient(stops: stops)
    }

    /// Bow-tie sheen of brushed or engraved metal: bright along the light axis.
    static func sheen(_ tint: RGB, strength: Double) -> Gradient {
        var stops: [Gradient.Stop] = []
        for k in 0...96 {
            let a = Double(k) / 96 * 2 * .pi
            let d = abs(cos(a) * x + sin(a) * y)
            stops.append(.init(color: tint.color(pow(d, 5) * strength), location: Double(k) / 96))
        }
        return Gradient(stops: stops)
    }
}

// MARK: - Geometry

struct WatchGeometry {
    let size: CGSize
    let c: CGPoint
    /// Radius of the largest circle that fits the screen, with a small margin.
    let dialR: CGFloat
    /// Distance from the center to a screen corner.
    let cornerR: CGFloat

    init(size: CGSize) {
        self.size = size
        c = CGPoint(x: size.width / 2, y: size.height / 2)
        dialR = min(size.width, size.height) / 2 * 0.96
        cornerR = hypot(size.width, size.height) / 2
    }

    var screen: Path { Path(CGRect(origin: .zero, size: size)) }

    /// Point at radius `r` and clock angle `deg` (0 = 12 o'clock, clockwise) around `o`.
    func polar(_ r: CGFloat, _ deg: Double, around o: CGPoint? = nil) -> CGPoint {
        let o = o ?? c
        let a = deg * .pi / 180
        return CGPoint(x: o.x + r * sin(a), y: o.y - r * cos(a))
    }

    /// Maps local coordinates (origin at the pivot, 12 o'clock = -y) onto the screen.
    func transform(_ deg: Double, around o: CGPoint? = nil) -> CGAffineTransform {
        let o = o ?? c
        return CGAffineTransform(translationX: o.x, y: o.y).rotated(by: deg * .pi / 180)
    }

    func circle(_ r: CGFloat, at p: CGPoint? = nil) -> Path {
        let o = p ?? c
        return Path(ellipseIn: CGRect(x: o.x - r, y: o.y - r, width: r * 2, height: r * 2))
    }
}

/// Rotation that keeps text tangent to a circle and never upside down.
func readableRotation(_ deg: Double) -> Double {
    if deg > 90 && deg < 270 { return deg - 180 }
    if deg >= 270 { return deg - 360 }
    return deg
}

// MARK: - Painter

struct WatchPainter {
    let theme: RealisticWatchTheme
    let geo: WatchGeometry

    init(theme: RealisticWatchTheme, size: CGSize) {
        self.theme = theme
        self.geo = WatchGeometry(size: size)
    }

    var c: CGPoint { geo.c }
    var dR: CGFloat { geo.dialR }
    var u: CGFloat { geo.dialR * 0.01 }
    /// The Bauhaus layout's reference radius (its outer numeral ring sits at 0.965 O).
    var O: CGFloat { geo.dialR * 0.995 }

    // MARK: Static dial

    func drawDial(_ ctx: GraphicsContext) {
        drawSurface(ctx)
        switch theme.layout {
        case .bauhaus:
            drawBauhausTrack(ctx)
            drawMarkers(ctx)
            drawBauhausNumerals(ctx)
        case .dress, .diver:
            drawMinuteTrack(ctx)
            drawDialPrint(ctx)
            drawMarkers(ctx)
        case .pilot:
            drawPilotDial(ctx)
        case .chronograph:
            drawChronographDial(ctx)
            drawDialPrint(ctx)
            drawMarkers(ctx)
        case .lunar:
            drawLunarDial(ctx)
            drawDialPrint(ctx)
        case .gmt:
            drawGMTRing(ctx)
            drawDialPrint(ctx)
            drawMarkers(ctx)
        }
    }

    /// The dial material, edge to edge.
    func drawSurface(_ ctx: GraphicsContext) {
        let screen = geo.screen
        ctx.fill(screen, with: .radialGradient(
            Gradient(colors: [theme.dialCenter.color(), theme.dialEdge.color()]),
            center: CGPoint(x: c.x - dR * 0.08, y: c.y - dR * 0.10),
            startRadius: 0, endRadius: geo.cornerR * 1.05))

        switch theme.surface {
        case .sunburst:
            // Fine radial brushing.
            let rays = 360
            var stops: [Gradient.Stop] = []
            stops.reserveCapacity(rays + 1)
            for i in 0...rays {
                let color: Color = i % 2 == 0 ? .white.opacity(0.05) : .black.opacity(0.07)
                stops.append(.init(color: color, location: Double(i) / Double(rays)))
            }
            ctx.fill(screen, with: .conicGradient(Gradient(stops: stops), center: c))
            ctx.fill(screen, with: .conicGradient(WatchLight.sheen(theme.dialCenter.scaled(3.2), strength: 0.40), center: c))
            ctx.fill(screen, with: .radialGradient(
                Gradient(colors: [theme.dialCenter.color(0.85), theme.dialCenter.color(0)]),
                center: c, startRadius: 0, endRadius: dR * 0.30))
        case .matte:
            // A soft, wide highlight toward the light, and a faint grain-like falloff.
            ctx.fill(screen, with: .radialGradient(
                Gradient(colors: [theme.dialCenter.scaled(1.8).color(0.35), .clear]),
                center: CGPoint(x: c.x - dR * 0.45, y: c.y - dR * 0.55),
                startRadius: 0, endRadius: dR * 1.1))
        case .guilloche:
            drawGuilloche(ctx)
        }

        // Vignette toward the screen corners, so the eye stays on the dial.
        ctx.fill(screen, with: .radialGradient(
            Gradient(stops: [
                .init(color: .clear, location: 0.55),
                .init(color: .black.opacity(0.55), location: 1),
            ]),
            center: c, startRadius: 0, endRadius: geo.cornerR))
    }

    /// Engine-turned "barleycorn" pattern: concentric rings with a wave that shifts
    /// from ring to ring, each cut catching the light as a pair of light/dark lines.
    func drawGuilloche(_ ctx: GraphicsContext) {
        let spacing = dR * 0.018
        let rings = Int(geo.cornerR / spacing) + 1
        let waves = 60.0
        let samples = 900
        for i in 1...rings {
            let r0 = CGFloat(i) * spacing
            let amp = spacing * 0.55
            let phase = Double(i) * 0.55
            var p = Path()
            for k in 0...samples {
                let a = Double(k) / Double(samples) * 2 * .pi
                let r = r0 + amp * sin(waves * a + phase)
                let pt = CGPoint(x: c.x + r * cos(a), y: c.y + r * sin(a))
                if k == 0 { p.move(to: pt) } else { p.addLine(to: pt) }
            }
            ctx.stroke(p.offsetBy(dx: -0.4, dy: -0.4), with: .color(.white.opacity(0.11)), lineWidth: max(0.5, spacing * 0.22))
            ctx.stroke(p, with: .color(.black.opacity(0.24)), lineWidth: max(0.5, spacing * 0.30))
        }
        ctx.fill(geo.screen, with: .conicGradient(WatchLight.sheen(theme.dialCenter.scaled(3.0), strength: 0.30), center: c))
    }

    func drawMinuteTrack(_ ctx: GraphicsContext) {
        let r2 = dR * 0.995
        for m in 0..<60 {
            let major = m % 5 == 0
            let r1 = r2 - dR * (major ? 0.055 : 0.03)
            var line = Path()
            line.move(to: geo.polar(r1, Double(m) * 6))
            line.addLine(to: geo.polar(r2, Double(m) * 6))
            ctx.stroke(line, with: .color(theme.print.color(major ? 0.9 : 0.55)),
                       style: StrokeStyle(lineWidth: max(0.6, dR * (major ? 0.0065 : 0.0032)), lineCap: .round))
        }
    }

    func drawDialPrint(_ ctx: GraphicsContext) {
        // Layouts with something at 6 print the subtitle under the brand instead.
        let subtitleUnderBrand = theme.layout == .chronograph || theme.layout == .lunar
        let brandY: CGFloat = -dR * 0.40
        if !theme.brand.isEmpty {
            var text = Text(theme.brand)
                .font(.system(size: dR * 0.075, weight: .semibold, design: .serif))
                .tracking(dR * 0.02)
            if theme.layout == .lunar { text = text.italic() }
            ctx.draw(text.foregroundColor(theme.print.color(0.94)),
                     at: CGPoint(x: c.x, y: c.y + brandY), anchor: .center)
        }
        if !theme.subtitle.isEmpty {
            ctx.draw(
                Text(theme.subtitle)
                    .font(.system(size: dR * 0.032, weight: .medium))
                    .tracking(dR * 0.012)
                    .foregroundColor((theme.layout == .gmt ? theme.accent : theme.print).color(0.8)),
                at: CGPoint(x: c.x, y: subtitleUnderBrand ? c.y + brandY + dR * 0.075 : c.y + dR * 0.34),
                anchor: .center
            )
        }
        if !theme.depthRating.isEmpty {
            ctx.draw(
                Text(theme.depthRating)
                    .font(.system(size: dR * 0.040, weight: .semibold))
                    .tracking(dR * 0.01)
                    .foregroundColor(theme.accent.color(0.95)),
                at: CGPoint(x: c.x, y: c.y + dR * 0.41),
                anchor: .center
            )
        }
    }

    /// Printed text with a little thickness (shadow) and optional glow.
    func drawEmbossed(_ ctx: GraphicsContext, _ text: Text, at p: CGPoint, rotation: Double = 0, glow: Color? = nil) {
        ctx.drawLayer { l in
            l.addFilter(.shadow(color: .black.opacity(0.8), radius: u * 0.5, x: u * 0.3, y: u * 0.5))
            if let glow { l.addFilter(.shadow(color: glow, radius: u * 1.4)) }
            l.translateBy(x: p.x, y: p.y)
            l.rotate(by: .degrees(rotation))
            l.draw(text, at: .zero, anchor: .center)
        }
    }

    // MARK: Bauhaus layout

    /// Four fine ticks between each pill baton, as on the original.
    func drawBauhausTrack(_ ctx: GraphicsContext) {
        let outer = O * 0.885
        let inner = outer - O * 0.145 * 0.44
        for m in 0..<60 where m % 5 != 0 {
            var line = Path()
            line.move(to: geo.polar(inner, Double(m) * 6))
            line.addLine(to: geo.polar(outer, Double(m) * 6))
            ctx.stroke(line, with: .color(theme.print.color(0.85)),
                       style: StrokeStyle(lineWidth: max(1.0, O * 0.0055), lineCap: .round))
        }
    }

    func drawBauhausNumerals(_ ctx: GraphicsContext) {
        for hour in 1...12 {
            let text = Text("\(hour)")
                .font(.system(size: O * 0.106, weight: .regular, design: .rounded))
                .foregroundColor(theme.numeral.color())
            drawEmbossed(ctx, text, at: geo.polar(O * 0.53, Double(hour) * 30),
                         glow: theme.numeral.color(theme.lumeGlow * 0.7))
        }
        let labels = ["60", "05", "10", "15", "20", "25", "30", "35", "40", "45", "50", "55"]
        for i in 0..<12 {
            let deg = Double(i) * 30
            let p = geo.polar(O * 0.965, deg)
            var tc = ctx
            tc.translateBy(x: p.x, y: p.y)
            tc.rotate(by: .degrees(readableRotation(deg)))
            tc.draw(
                Text(labels[i])
                    .font(.system(size: O * 0.054, weight: .regular, design: .rounded))
                    .foregroundColor(theme.print.color(0.9)),
                at: .zero, anchor: .center)
        }
    }

    // MARK: Applied markers

    struct Marker {
        var full: Path
        var left: Path
        var right: Path
        var lume: Path?
        var deg: Double
        var tip: CGPoint
        var base: CGPoint
        var roundCenter: CGPoint?
    }

    func polygon(_ pts: [CGPoint]) -> Path {
        var p = Path()
        p.addLines(pts)
        p.closeSubpath()
        return p
    }

    func bar(_ deg: Double, r1: CGFloat, r2: CGFloat, w: CGFloat, xOff: CGFloat = 0, pill: Bool = false, lume: Bool = true) -> Marker {
        let t = geo.transform(deg)
        let rect = CGRect(x: xOff - w / 2, y: -r2, width: w, height: r2 - r1)
        let lumeRect = pill ? rect.insetBy(dx: w * 0.22, dy: w * 0.22) : rect.insetBy(dx: w * 0.27, dy: w * 0.34)
        let corner = pill ? w / 2 : w * 0.12
        return Marker(
            full: Path(roundedRect: rect, cornerRadius: corner).applying(t),
            left: Path(CGRect(x: rect.minX, y: rect.minY, width: w / 2, height: rect.height)).applying(t),
            right: Path(CGRect(x: rect.midX, y: rect.minY, width: w / 2, height: rect.height)).applying(t),
            lume: lume ? Path(roundedRect: lumeRect, cornerRadius: pill ? lumeRect.width / 2 : w * 0.08).applying(t) : nil,
            deg: deg,
            tip: CGPoint(x: xOff, y: -r2).applying(t),
            base: CGPoint(x: xOff, y: -r1).applying(t),
            roundCenter: nil
        )
    }

    /// Wide at the outside, narrowing toward the center.
    func arrowhead(_ deg: Double, r1: CGFloat, r2: CGFloat, w: CGFloat) -> Marker {
        let t = geo.transform(deg)
        let a = CGPoint(x: -w / 2, y: -r2), b = CGPoint(x: w / 2, y: -r2)
        let cR = CGPoint(x: w * 0.16, y: -r1), cL = CGPoint(x: -w * 0.16, y: -r1)
        let inset: (CGPoint) -> CGPoint = { p in
            let g = CGPoint(x: 0, y: -(r1 + r2) / 2)
            return CGPoint(x: g.x + (p.x - g.x) * 0.55, y: g.y + (p.y - g.y) * 0.72)
        }
        return Marker(
            full: polygon([a, b, cR, cL]).applying(t),
            left: polygon([a, CGPoint(x: 0, y: -r2), CGPoint(x: 0, y: -r1), cL]).applying(t),
            right: polygon([CGPoint(x: 0, y: -r2), b, cR, CGPoint(x: 0, y: -r1)]).applying(t),
            lume: polygon([inset(a), inset(b), inset(cR), inset(cL)]).applying(t),
            deg: deg,
            tip: CGPoint(x: 0, y: -r2).applying(t),
            base: CGPoint(x: 0, y: -r1).applying(t),
            roundCenter: nil
        )
    }

    func triangle(_ deg: Double, r1: CGFloat, r2: CGFloat, half: CGFloat) -> Marker {
        let t = geo.transform(deg)
        let apex = CGPoint(x: 0, y: -r1)
        let a = CGPoint(x: -half, y: -r2), b = CGPoint(x: half, y: -r2)
        let g = CGPoint(x: 0, y: -(r1 + 2 * r2) / 3)
        func toward(_ p: CGPoint) -> CGPoint {
            CGPoint(x: g.x + (p.x - g.x) * 0.58, y: g.y + (p.y - g.y) * 0.58)
        }
        return Marker(
            full: polygon([apex, a, b]).applying(t),
            left: polygon([apex, a, CGPoint(x: 0, y: -r2)]).applying(t),
            right: polygon([apex, CGPoint(x: 0, y: -r2), b]).applying(t),
            lume: polygon([toward(apex), toward(a), toward(b)]).applying(t),
            deg: deg,
            tip: CGPoint(x: 0, y: -r2).applying(t),
            base: apex.applying(t),
            roundCenter: nil
        )
    }

    func dot(_ deg: Double, rc: CGFloat, rd: CGFloat) -> Marker {
        let center = geo.polar(rc, deg)
        let full = geo.circle(rd, at: center)
        return Marker(
            full: full, left: full, right: full,
            lume: geo.circle(rd * 0.74, at: center),
            deg: deg, tip: center, base: center,
            roundCenter: center
        )
    }

    func markers() -> [Marker] {
        var out: [Marker] = []
        for h in 0..<12 {
            let deg = Double(h) * 30
            let dateHere = h == 3 && theme.showsDate
            switch theme.layout {
            case .bauhaus:
                let len = O * 0.145
                let outer = O * 0.885
                out.append(bar(deg, r1: outer - len, r2: outer, w: O * 0.034, pill: true))
            case .dress:
                if dateHere { continue }
                let w = dR * 0.045
                let r1 = dR * 0.74, r2 = dR * 0.93
                if h == 0 {
                    out.append(bar(deg, r1: r1, r2: r2, w: w, xOff: -w * 0.8))
                    out.append(bar(deg, r1: r1, r2: r2, w: w, xOff: w * 0.8))
                } else {
                    out.append(bar(deg, r1: r1, r2: r2, w: w))
                }
            case .diver:
                if dateHere { continue }
                if h == 0 {
                    out.append(triangle(deg, r1: dR * 0.74, r2: dR * 0.93, half: dR * 0.10))
                } else if h == 6 || h == 9 {
                    out.append(bar(deg, r1: dR * 0.74, r2: dR * 0.93, w: dR * 0.085))
                } else {
                    out.append(dot(deg, rc: dR * 0.85, rd: dR * 0.062))
                }
            case .chronograph:
                let w = dR * 0.030
                let r1 = dR * 0.70, r2 = dR * 0.835
                if h == 0 {
                    out.append(bar(deg, r1: r1, r2: r2, w: w, xOff: -w * 0.8))
                    out.append(bar(deg, r1: r1, r2: r2, w: w, xOff: w * 0.8))
                } else {
                    out.append(bar(deg, r1: r1, r2: r2, w: w))
                }
            case .gmt:
                if dateHere { continue }
                let w = dR * (h == 0 ? 0.11 : 0.075)
                out.append(arrowhead(deg, r1: dR * 0.60, r2: dR * 0.82, w: w))
            case .pilot, .lunar:
                break // printed, not applied
            }
        }
        return out
    }

    func drawMarkers(_ ctx: GraphicsContext) {
        let ms = markers()
        // Cast shadows, all in one direction on screen.
        ctx.drawLayer { l in
            l.addFilter(.blur(radius: u * 1.0))
            for m in ms {
                l.fill(m.full.offsetBy(dx: u * 0.7, dy: u * 1.1), with: .color(.black.opacity(0.7)))
            }
        }
        for m in ms {
            if let center = m.roundCenter {
                ctx.fill(m.full, with: .conicGradient(
                    WatchLight.metal(theme.indexLight, theme.indexDark, gloss: 0.9), center: center))
            } else {
                var cl = ctx
                cl.clip(to: m.full)
                for left in [true, false] {
                    let col = WatchLight.facet(theme.indexLight, theme.indexDark, clockDeg: m.deg, left: left)
                    cl.fill(left ? m.left : m.right, with: .linearGradient(
                        Gradient(colors: [col.scaled(1.12).color(), col.scaled(0.78).color()]),
                        startPoint: m.tip, endPoint: m.base))
                }
            }
            ctx.stroke(m.full, with: .color(.white.opacity(0.22)), lineWidth: max(0.5, u * 0.18))
            if let lume = m.lume { drawLume(ctx, lume) }
        }
    }

    func drawLume(_ ctx: GraphicsContext, _ path: Path) {
        ctx.drawLayer { l in
            l.addFilter(.shadow(color: theme.lume.color(theme.lumeGlow), radius: u * 2.0))
            l.fill(path, with: .color(theme.lume.color()))
        }
        ctx.stroke(path, with: .color(.black.opacity(0.3)), lineWidth: max(0.4, u * 0.14))
    }

    /// Sunken area (sub-dial, aperture): the rim shades its top-left inside edge.
    func drawRecessShadow(_ ctx: GraphicsContext, center: CGPoint, radius r: CGFloat) {
        var cl = ctx
        cl.clip(to: geo.circle(r, at: center))
        cl.drawLayer { l in
            l.addFilter(.blur(radius: r * 0.06))
            var crescent = geo.circle(r * 1.6, at: center)
            crescent.addPath(geo.circle(r, at: CGPoint(x: center.x + r * 0.07, y: center.y + r * 0.10)))
            l.fill(crescent, with: .color(.black.opacity(0.7)), style: FillStyle(eoFill: true))
        }
    }

    /// Thin polished ring around a recess.
    func drawBevelRing(_ ctx: GraphicsContext, center: CGPoint, radius r: CGFloat, width w: CGFloat) {
        var ring = geo.circle(r + w / 2, at: center)
        ring.addPath(geo.circle(r - w / 2, at: center))
        ctx.fill(ring, with: .conicGradient(
            WatchLight.metal(theme.indexLight, theme.indexDark, gloss: 1.0, inward: true), center: center),
                 style: FillStyle(eoFill: true))
    }

    // MARK: Moving parts

    func drawMoving(_ ctx: GraphicsContext, time: WatchTime) {
        if theme.showsDate {
            drawDate(ctx, day: Calendar.current.component(.day, from: time.date))
        }
        switch theme.layout {
        case .chronograph: drawChronographHands(ctx, time: time)
        case .lunar: drawMoonPhase(ctx, date: time.date)
        default: break
        }
        drawHands(ctx, time: time)
    }

    func drawDate(_ ctx: GraphicsContext, day: Int) {
        let center = geo.polar(dR * theme.dateRadius, 90)
        let w = dR * 0.15, h = dR * 0.115
        let win = CGRect(x: center.x - w / 2, y: center.y - h / 2, width: w, height: h)
        let frame = win.insetBy(dx: -dR * 0.018, dy: -dR * 0.018)
        let framePath = Path(roundedRect: frame, cornerRadius: dR * 0.018)
        let winPath = Path(roundedRect: win, cornerRadius: dR * 0.010)

        ctx.drawLayer { l in
            l.addFilter(.blur(radius: u * 1.0))
            l.fill(framePath.offsetBy(dx: u * 0.7, dy: u * 1.1), with: .color(.black.opacity(0.7)))
        }
        ctx.fill(framePath, with: .linearGradient(
            Gradient(colors: [theme.indexLight.color(), theme.indexDark.color()]),
            startPoint: CGPoint(x: frame.minX, y: frame.minY),
            endPoint: CGPoint(x: frame.maxX, y: frame.maxY)))
        ctx.fill(winPath, with: .color(theme.dateWheel.color()))
        ctx.draw(
            Text("\(day)")
                .font(.system(size: h * 0.78, weight: .semibold, design: .default))
                .foregroundColor(theme.dateText.color()),
            at: CGPoint(x: win.midX, y: win.midY),
            anchor: .center
        )
        var wc = ctx
        wc.clip(to: winPath)
        wc.fill(winPath, with: .linearGradient(
            Gradient(stops: [.init(color: .black.opacity(0.6), location: 0), .init(color: .clear, location: 0.4)]),
            startPoint: CGPoint(x: win.midX, y: win.minY), endPoint: CGPoint(x: win.midX, y: win.maxY)))
        wc.fill(winPath, with: .linearGradient(
            Gradient(stops: [.init(color: .black.opacity(0.4), location: 0), .init(color: .clear, location: 0.25)]),
            startPoint: CGPoint(x: win.minX, y: win.midY), endPoint: CGPoint(x: win.maxX, y: win.midY)))
    }

    // MARK: Hands

    struct HandShape {
        var full: Path
        var left: Path
        var right: Path
        var lume: Path?
        var tip: CGPoint
        var base: CGPoint
        /// Open ring (Breguet): center (local) and outer / inner radius.
        var ring: (center: CGPoint, outer: CGFloat, inner: CGFloat)?
    }

    /// Faceted sword hand pointing up (-y) from the pivot at the origin.
    func sword(length L: CGFloat, width bw: CGFloat) -> HandShape {
        let tail = bw * 0.55
        let tip = CGPoint(x: 0, y: -L)
        let shoulderL = CGPoint(x: -bw, y: -L * 0.30), shoulderR = CGPoint(x: bw, y: -L * 0.30)
        let baseL = CGPoint(x: -bw * 0.55, y: tail), baseR = CGPoint(x: bw * 0.55, y: tail)
        return HandShape(
            full: polygon([tip, shoulderR, baseR, baseL, shoulderL]),
            left: polygon([tip, shoulderL, baseL, CGPoint(x: 0, y: tail)]),
            right: polygon([tip, CGPoint(x: 0, y: tail), baseR, shoulderR]),
            lume: polygon([
                CGPoint(x: 0, y: -L * 0.88),
                CGPoint(x: bw * 0.52, y: -L * 0.32),
                CGPoint(x: bw * 0.30, y: -L * 0.10),
                CGPoint(x: -bw * 0.30, y: -L * 0.10),
                CGPoint(x: -bw * 0.52, y: -L * 0.32),
            ]),
            tip: tip,
            base: CGPoint(x: 0, y: tail),
            ring: nil
        )
    }

    /// Tapered pencil hand with a chisel tip and a lume groove (the Bauhaus hands).
    func pencil(length L: CGFloat, width bw: CGFloat) -> HandShape {
        let hb = bw / 2 * 0.8, ht = bw * 0.65 / 2
        let chisel = bw * 0.65 * 0.75
        let tail = hb
        let tip = CGPoint(x: 0, y: -L)
        let tl = CGPoint(x: -ht, y: -L + chisel), tr = CGPoint(x: ht, y: -L + chisel)
        let bl = CGPoint(x: -hb, y: tail), br = CGPoint(x: hb, y: tail)
        let groove = CGRect(x: -bw * 0.19, y: -L * 0.85, width: bw * 0.38, height: L * 0.70)
        return HandShape(
            full: polygon([tip, tr, br, bl, tl]),
            left: polygon([tip, tl, bl, CGPoint(x: 0, y: tail)]),
            right: polygon([tip, CGPoint(x: 0, y: tail), br, tr]),
            lume: Path(roundedRect: groove, cornerRadius: groove.width / 2),
            tip: tip,
            base: CGPoint(x: 0, y: tail),
            ring: nil
        )
    }

    /// Straight stick hand with a short pointed tip and a lume stripe.
    func stick(length L: CGFloat, width bw: CGFloat, lume: Bool = true) -> HandShape {
        let hw = bw / 2
        let tail = bw * 1.2
        let tip = CGPoint(x: 0, y: -L)
        let sl = CGPoint(x: -hw, y: -L + bw * 1.4), sr = CGPoint(x: hw, y: -L + bw * 1.4)
        let bl = CGPoint(x: -hw, y: tail), br = CGPoint(x: hw, y: tail)
        let stripe = CGRect(x: -hw * 0.45, y: -L * 0.86, width: hw * 0.9, height: L * 0.62)
        return HandShape(
            full: polygon([tip, sr, br, bl, sl]),
            left: polygon([tip, sl, bl, CGPoint(x: 0, y: tail)]),
            right: polygon([tip, CGPoint(x: 0, y: tail), br, sr]),
            lume: lume ? Path(roundedRect: stripe, cornerRadius: stripe.width / 2) : nil,
            tip: tip,
            base: CGPoint(x: 0, y: tail),
            ring: nil
        )
    }

    /// Thin hand with an open ring near the tip and a fine tapered point.
    func breguet(length L: CGFloat, width w: CGFloat, ringAt f: CGFloat) -> HandShape {
        let ro = w * 3.0, ri = w * 1.9
        let ringC = CGPoint(x: 0, y: -L * f)
        let tail = w * 3
        // Shaft: from the tail up to the bottom of the ring.
        let shaftTop = ringC.y + ro * 0.92
        let shaftL = [CGPoint(x: -w * 0.75, y: tail), CGPoint(x: -w * 0.5, y: shaftTop), CGPoint(x: 0, y: shaftTop), CGPoint(x: 0, y: tail)]
        let shaftR = [CGPoint(x: 0, y: tail), CGPoint(x: 0, y: shaftTop), CGPoint(x: w * 0.5, y: shaftTop), CGPoint(x: w * 0.75, y: tail)]
        // Point: from the top of the ring to the tip.
        let pointBase = ringC.y - ro * 0.92
        let tipPt = CGPoint(x: 0, y: -L)
        let pointL = [CGPoint(x: -w * 0.6, y: pointBase), tipPt, CGPoint(x: 0, y: pointBase)]
        let pointR = [CGPoint(x: 0, y: pointBase), tipPt, CGPoint(x: w * 0.6, y: pointBase)]
        var left = polygon(shaftL); left.addPath(polygon(pointL))
        var right = polygon(shaftR); right.addPath(polygon(pointR))
        var full = polygon(shaftL + shaftR.reversed())
        full.addPath(polygon([CGPoint(x: -w * 0.6, y: pointBase), tipPt, CGPoint(x: w * 0.6, y: pointBase)]))
        return HandShape(
            full: full, left: left, right: right, lume: nil,
            tip: tipPt, base: CGPoint(x: 0, y: tail),
            ring: (ringC, ro, ri)
        )
    }

    func castShadow(_ ctx: GraphicsContext, _ path: Path, height h: CGFloat) {
        ctx.drawLayer { l in
            l.addFilter(.blur(radius: u * h * 0.75))
            l.fill(path.offsetBy(dx: u * h * 0.6, dy: u * h * 1.0), with: .color(.black.opacity(0.62)))
        }
    }

    func drawHand(_ ctx: GraphicsContext, _ shape: HandShape, deg: Double, height: CGFloat,
                  light: RGB? = nil, dark: RGB? = nil, pivot: CGPoint? = nil) {
        let light = light ?? theme.handLight
        let dark = dark ?? theme.handDark
        let t = geo.transform(deg, around: pivot)
        let full = shape.full.applying(t)

        var ringPath: Path?
        var ringCenter = CGPoint.zero
        if let ring = shape.ring {
            ringCenter = ring.center.applying(t)
            var p = geo.circle(ring.outer, at: ringCenter)
            p.addPath(geo.circle(ring.inner, at: ringCenter))
            ringPath = p
        }

        // Shadow (hand plus ring).
        ctx.drawLayer { l in
            l.addFilter(.blur(radius: u * height * 0.75))
            let dx = u * height * 0.6, dy = u * height * 1.0
            l.fill(full.offsetBy(dx: dx, dy: dy), with: .color(.black.opacity(0.62)))
            if let ringPath {
                l.fill(ringPath.offsetBy(dx: dx, dy: dy), with: .color(.black.opacity(0.62)), style: FillStyle(eoFill: true))
            }
        }

        var cl = ctx
        cl.clip(to: full)
        let tip = shape.tip.applying(t), base = shape.base.applying(t)
        for left in [true, false] {
            let col = WatchLight.facet(light, dark, clockDeg: deg, left: left)
            cl.fill((left ? shape.left : shape.right).applying(t), with: .linearGradient(
                Gradient(colors: [col.scaled(1.12).color(), col.scaled(0.8).color()]),
                startPoint: tip, endPoint: base))
        }
        ctx.stroke(full, with: .color(dark.scaled(0.45).color(0.85)), lineWidth: max(0.5, u * 0.15))
        if let ringPath {
            ctx.fill(ringPath, with: .conicGradient(WatchLight.metal(light, dark, gloss: 1.0), center: ringCenter),
                     style: FillStyle(eoFill: true))
        }
        if let lume = shape.lume { drawLume(ctx, lume.applying(t)) }
    }

    func drawHands(_ ctx: GraphicsContext, time: WatchTime) {
        let diver = theme.layout == .diver
        var hubR: CGFloat = dR * 0.040
        var Ls: CGFloat = dR * 0.95
        var tail: CGFloat = dR * 0.20

        switch theme.hands {
        case .pencil:
            drawHand(ctx, pencil(length: O * 0.52, width: O * 0.038), deg: time.hour, height: 1.4)
            drawHand(ctx, pencil(length: O * 0.735, width: O * 0.028), deg: time.minute, height: 2.0)
            hubR = O * 0.036
            Ls = O * 0.885 * 0.985
            tail = O * 0.175
        case .sword:
            let wide = diver || theme.layout == .pilot
            drawHand(ctx, sword(length: dR * 0.55, width: dR * (wide ? 0.062 : 0.048)), deg: time.hour, height: 1.4)
            if theme.layout == .gmt { drawGMTHand(ctx, time: time) }
            drawHand(ctx, sword(length: dR * (theme.layout == .pilot ? 0.88 : 0.84),
                                width: dR * (wide ? 0.050 : 0.038)), deg: time.minute, height: 2.0)
        case .stick:
            drawHand(ctx, stick(length: dR * 0.55, width: dR * 0.040), deg: time.hour, height: 1.4)
            drawHand(ctx, stick(length: dR * 0.86, width: dR * 0.030), deg: time.minute, height: 2.0)
            hubR = dR * 0.034
            Ls = dR * 0.90
        case .breguet:
            drawHand(ctx, breguet(length: dR * 0.52, width: dR * 0.012, ringAt: 0.70), deg: time.hour, height: 1.4)
            drawHand(ctx, breguet(length: dR * 0.80, width: dR * 0.0095, ringAt: 0.78), deg: time.minute, height: 2.0)
            hubR = dR * 0.030
            Ls = dR * 0.88
            tail = dR * 0.16
        }

        // Hub.
        let hub = geo.circle(hubR)
        castShadow(ctx, hub, height: 2.0)
        ctx.fill(hub, with: .conicGradient(WatchLight.metal(theme.handLight, theme.handDark, gloss: 0.9), center: c))

        // Central seconds.
        let t = geo.transform(time.second)
        let sw = max(1.3, dR * (theme.hands == .breguet ? 0.0055 : 0.008))
        var needle = Path(roundedRect: CGRect(x: -sw / 2, y: -Ls, width: sw, height: Ls + tail), cornerRadius: sw / 2)
        let weightR = dR * (theme.hands == .breguet ? 0.018 : 0.026)
        needle.addEllipse(in: CGRect(x: -weightR, y: tail * 0.72 - weightR, width: weightR * 2, height: weightR * 2))
        let lollipopR = dR * 0.030
        let lollipopY = -Ls * 0.70
        if diver {
            needle.addEllipse(in: CGRect(x: -lollipopR, y: lollipopY - lollipopR, width: lollipopR * 2, height: lollipopR * 2))
        }
        let screenNeedle = needle.applying(t)
        castShadow(ctx, screenNeedle, height: 2.8)
        let shade = WatchLight.facet(theme.second.scaled(1.15), theme.second.scaled(0.7), clockDeg: time.second, left: true)
        if theme.hands == .pencil {
            ctx.drawLayer { l in
                l.addFilter(.shadow(color: theme.second.color(theme.lumeGlow * 0.8), radius: u * 1.2))
                l.fill(screenNeedle, with: .color(shade.color()))
            }
        } else {
            ctx.fill(screenNeedle, with: .color(shade.color()))
        }
        let tipPath = Path(CGRect(x: -sw / 2, y: -Ls, width: sw, height: Ls * 0.14)).applying(t)
        ctx.fill(tipPath, with: .color(theme.secondTip.color()))
        if diver {
            drawLume(ctx, Path(ellipseIn: CGRect(
                x: -lollipopR * 0.7, y: lollipopY - lollipopR * 0.7,
                width: lollipopR * 1.4, height: lollipopR * 1.4)).applying(t))
        }

        // Cap.
        let capR = hubR * 0.5
        ctx.fill(geo.circle(capR), with: .conicGradient(
            WatchLight.metal(theme.second.scaled(1.3), theme.second.scaled(0.55), gloss: 0.8), center: c))
        ctx.fill(geo.circle(capR * 0.35, at: CGPoint(x: c.x - capR * 0.3, y: c.y - capR * 0.3)),
                 with: .color(.white.opacity(0.45)))
    }
}

// MARK: - Face switcher

/// Shows the chosen face: the original Bauhaus clock or one of the 3D dials.
public struct TimedClockFaceView: View {
    public let model: TimedClockModel
    public let face: WatchFace
    public let bauhausTheme: BauhausDialTheme?

    public init(model: TimedClockModel = .shared, face: WatchFace, bauhausTheme: BauhausDialTheme? = nil) {
        self.model = model
        self.face = face
        self.bauhausTheme = bauhausTheme
    }

    public var body: some View {
        if let theme = face.realisticTheme {
            RealisticWatchView(model: model, theme: theme)
        } else {
            TimedClockMainView(model: model, theme: bauhausTheme)
        }
    }
}
