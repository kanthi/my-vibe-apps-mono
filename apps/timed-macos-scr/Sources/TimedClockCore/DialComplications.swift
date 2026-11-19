import SwiftUI

/// Layout-specific printing and complications for the 3D dials:
/// pilot, chronograph, moon phase and GMT.
extension WatchPainter {

    // MARK: - Pilot

    func drawPilotDial(_ ctx: GraphicsContext) {
        // Minute track at the edge, heavier every five minutes.
        let r2 = dR * 0.995
        for m in 0..<60 where m != 0 {
            let major = m % 5 == 0
            let r1 = r2 - dR * (major ? 0.07 : 0.035)
            var line = Path()
            line.move(to: geo.polar(r1, Double(m) * 6))
            line.addLine(to: geo.polar(r2, Double(m) * 6))
            ctx.stroke(line, with: .color(theme.print.color(major ? 0.95 : 0.7)),
                       style: StrokeStyle(lineWidth: max(0.8, dR * (major ? 0.011 : 0.004)), lineCap: .butt))
        }

        let glow = theme.lume.color(theme.lumeGlow)

        // Triangle with two dots at 12.
        var tri = Path()
        tri.move(to: geo.polar(dR * 0.80, 0))
        tri.addLine(to: CGPoint(x: c.x - dR * 0.065, y: c.y - r2 + dR * 0.005))
        tri.addLine(to: CGPoint(x: c.x + dR * 0.065, y: c.y - r2 + dR * 0.005))
        tri.closeSubpath()
        var dots = geo.circle(dR * 0.020, at: geo.polar(dR * 0.955, -7.5))
        dots.addPath(geo.circle(dR * 0.020, at: geo.polar(dR * 0.955, 7.5)))
        ctx.drawLayer { l in
            l.addFilter(.shadow(color: .black.opacity(0.8), radius: u * 0.5, x: u * 0.3, y: u * 0.5))
            l.addFilter(.shadow(color: glow, radius: u * 1.4))
            l.fill(tri, with: .color(theme.numeral.color()))
            l.fill(dots, with: .color(theme.numeral.color()))
        }

        // Large printed numerals 1–11.
        for hour in 1...11 {
            let text = Text("\(hour)")
                .font(.system(size: dR * 0.17, weight: .medium, design: .default))
                .foregroundColor(theme.numeral.color())
            drawEmbossed(ctx, text, at: geo.polar(dR * 0.76, Double(hour) * 30), glow: glow)
        }
    }

    // MARK: - Chronograph

    /// Sub-dials: running seconds at 9, 30-minute counter at 3, 12-hour counter at 6.
    var subdials: [(deg: Double, count: Int, majorEvery: Int, labels: [(Int, Double)])] {
        [
            (270, 60, 5, [(20, 120), (40, 240), (60, 0)]),
            (90, 30, 5, [(10, 120), (20, 240), (30, 0)]),
            (180, 12, 1, [(3, 90), (6, 180), (9, 270), (12, 0)]),
        ]
    }

    var subdialRadius: CGFloat { dR * 0.19 }
    func subdialCenter(_ deg: Double) -> CGPoint { geo.polar(dR * 0.42, deg) }

    func drawChronographDial(_ ctx: GraphicsContext) {
        // Tachymeter scale on the outer edge.
        let tachy: [Int] = [500, 400, 300, 250, 200, 180, 160, 150, 140, 130, 120, 110, 100, 90, 80, 75, 70, 65]
        let labelled: Set<Int> = [500, 400, 300, 250, 200, 150, 120, 100, 90, 80, 70]
        for v in tachy {
            let deg = 3600.0 / Double(v) * 6
            var tick = Path()
            tick.move(to: geo.polar(dR * 0.975, deg))
            tick.addLine(to: geo.polar(dR * 0.998, deg))
            ctx.stroke(tick, with: .color(theme.print.color(0.85)), lineWidth: max(0.6, dR * 0.003))
            if labelled.contains(v) {
                let p = geo.polar(dR * 0.948, deg)
                var tc = ctx
                tc.translateBy(x: p.x, y: p.y)
                tc.rotate(by: .degrees(readableRotation(deg)))
                tc.draw(Text("\(v)")
                    .font(.system(size: dR * 0.030, weight: .medium, design: .default))
                    .foregroundColor(theme.print.color(0.85)), at: .zero, anchor: .center)
            }
        }
        let tLabel = geo.polar(dR * 0.948, 340)
        var tc = ctx
        tc.translateBy(x: tLabel.x, y: tLabel.y)
        tc.rotate(by: .degrees(readableRotation(340)))
        tc.draw(Text("TACHYMETER")
            .font(.system(size: dR * 0.022, weight: .semibold))
            .tracking(dR * 0.004)
            .foregroundColor(theme.accent.color(0.9)), at: .zero, anchor: .center)
        ctx.stroke(geo.circle(dR * 0.918), with: .color(theme.print.color(0.35)), lineWidth: max(0.5, dR * 0.002))

        // Fine 1/5-second track.
        for i in 0..<300 {
            let deg = Double(i) * 1.2
            let len: CGFloat = i % 25 == 0 ? 0.045 : (i % 5 == 0 ? 0.028 : 0.014)
            var line = Path()
            line.move(to: geo.polar(dR * (0.905 - len), deg))
            line.addLine(to: geo.polar(dR * 0.905, deg))
            ctx.stroke(line, with: .color(theme.print.color(i % 5 == 0 ? 0.85 : 0.5)),
                       lineWidth: max(0.4, dR * (i % 5 == 0 ? 0.0035 : 0.0016)))
        }

        // Recessed sub-dials with azurage (concentric grooves).
        let sR = subdialRadius
        for s in subdials {
            let center = subdialCenter(s.deg)
            let disc = geo.circle(sR, at: center)
            ctx.fill(disc, with: .radialGradient(
                Gradient(colors: [theme.secondary.scaled(1.6).color(), theme.secondary.color()]),
                center: center, startRadius: 0, endRadius: sR))
            var cl = ctx
            cl.clip(to: disc)
            let grooves = 16
            for g in 1...grooves {
                let r = sR * CGFloat(g) / CGFloat(grooves)
                cl.stroke(geo.circle(r, at: CGPoint(x: center.x - 0.4, y: center.y - 0.4)),
                          with: .color(.white.opacity(0.06)), lineWidth: max(0.5, sR / CGFloat(grooves) * 0.35))
                cl.stroke(geo.circle(r - sR / CGFloat(grooves) * 0.5, at: center),
                          with: .color(.black.opacity(0.25)), lineWidth: max(0.5, sR / CGFloat(grooves) * 0.35))
            }
            cl.fill(disc, with: .conicGradient(WatchLight.sheen(.init(1, 1, 1), strength: 0.10), center: center))
            drawRecessShadow(ctx, center: center, radius: sR)

            // Scale.
            for k in 0..<s.count {
                let deg = Double(k) / Double(s.count) * 360
                let major = k % s.majorEvery == 0
                var line = Path()
                line.move(to: geo.polar(sR * (major ? 0.74 : 0.82), deg, around: center))
                line.addLine(to: geo.polar(sR * 0.93, deg, around: center))
                ctx.stroke(line, with: .color(theme.print.color(major ? 0.9 : 0.6)),
                           lineWidth: max(0.5, sR * (major ? 0.022 : 0.010)))
            }
            for (value, deg) in s.labels {
                ctx.draw(Text("\(value)")
                    .font(.system(size: sR * 0.17, weight: .medium))
                    .foregroundColor(theme.print.color(0.85)),
                         at: geo.polar(sR * 0.55, deg, around: center), anchor: .center)
            }
            drawBevelRing(ctx, center: center, radius: sR, width: sR * 0.045)
        }
    }

    func drawChronographHands(_ ctx: GraphicsContext, time: WatchTime) {
        let sR = subdialRadius
        let minutes = time.minute / 6 // 0..<60
        let values: [Double] = [
            time.second,                                  // running seconds
            minutes.truncatingRemainder(dividingBy: 30) / 30 * 360, // 30-minute counter
            time.hour,                                    // 12-hour counter
        ]
        for (i, s) in subdials.enumerated() {
            let center = subdialCenter(s.deg)
            let L = sR * 0.86
            let w = max(1.2, dR * 0.008)
            let shape = HandShape(
                full: polygon([CGPoint(x: 0, y: -L), CGPoint(x: w / 2, y: -L * 0.82), CGPoint(x: w * 0.7, y: sR * 0.18),
                               CGPoint(x: -w * 0.7, y: sR * 0.18), CGPoint(x: -w / 2, y: -L * 0.82)]),
                left: polygon([CGPoint(x: 0, y: -L), CGPoint(x: -w / 2, y: -L * 0.82), CGPoint(x: -w * 0.7, y: sR * 0.18), CGPoint(x: 0, y: sR * 0.18)]),
                right: polygon([CGPoint(x: 0, y: -L), CGPoint(x: 0, y: sR * 0.18), CGPoint(x: w * 0.7, y: sR * 0.18), CGPoint(x: w / 2, y: -L * 0.82)]),
                lume: nil, tip: CGPoint(x: 0, y: -L), base: CGPoint(x: 0, y: sR * 0.18), ring: nil)
            drawHand(ctx, shape, deg: values[i], height: 0.9, pivot: center)
            ctx.fill(geo.circle(sR * 0.075, at: center), with: .conicGradient(
                WatchLight.metal(theme.handLight, theme.handDark, gloss: 0.9), center: center))
        }
    }

    // MARK: - Lunar (moon phase)

    var moonCenter: CGPoint { geo.polar(dR * 0.42, 180) }
    var moonWindowRadius: CGFloat { dR * 0.15 }

    func drawLunarDial(_ ctx: GraphicsContext) {
        // Railroad minute track.
        let rIn = dR * 0.905, rOut = dR * 0.945
        ctx.stroke(geo.circle(rIn), with: .color(theme.print.color(0.75)), lineWidth: max(0.6, dR * 0.0025))
        ctx.stroke(geo.circle(rOut), with: .color(theme.print.color(0.75)), lineWidth: max(0.6, dR * 0.0025))
        for m in 0..<60 {
            var line = Path()
            line.move(to: geo.polar(rIn, Double(m) * 6))
            line.addLine(to: geo.polar(rOut, Double(m) * 6))
            ctx.stroke(line, with: .color(theme.print.color(0.7)), lineWidth: max(0.5, dR * (m % 5 == 0 ? 0.004 : 0.002)))
            if m % 5 == 0 {
                ctx.fill(geo.circle(dR * 0.008, at: geo.polar(dR * 0.97, Double(m) * 6)), with: .color(theme.print.color(0.85)))
            }
        }

        // Roman numerals, radial, tops toward the edge.
        let romans = ["XII", "I", "II", "III", "IIII", "V", "VI", "VII", "VIII", "IX", "X", "XI"]
        for h in 0..<12 {
            let deg = Double(h) * 30
            let text = Text(romans[h])
                .font(.system(size: dR * 0.095, weight: .regular, design: .serif))
                .foregroundColor(theme.numeral.color())
            drawEmbossed(ctx, text, at: geo.polar(dR * 0.80, deg), rotation: deg)
        }
    }

    /// Moon age in days since a known new moon, 0 ..< 29.53.
    static func moonAge(_ date: Date) -> Double {
        let synodic = 29.530588853
        let knownNewMoon = 947_182_440.0 // 2000-01-06 18:14 UTC
        let days = (date.timeIntervalSince1970 - knownNewMoon) / 86_400
        let age = days.truncatingRemainder(dividingBy: synodic)
        return age < 0 ? age + synodic : age
    }

    func drawMoonPhase(_ ctx: GraphicsContext, date: Date) {
        let P = moonCenter, W = moonWindowRadius
        let window = geo.circle(W, at: P)

        // Polished frame and its shadow.
        ctx.drawLayer { l in
            l.addFilter(.blur(radius: u * 1.0))
            l.fill(geo.circle(W * 1.10, at: CGPoint(x: P.x + u * 0.7, y: P.y + u * 1.1)), with: .color(.black.opacity(0.7)))
        }
        drawBevelRing(ctx, center: P, radius: W * 1.04, width: W * 0.10)

        // Night sky with a few stars.
        ctx.fill(window, with: .radialGradient(
            Gradient(colors: [theme.secondary.scaled(1.8).color(), theme.secondary.scaled(0.5).color()]),
            center: P, startRadius: 0, endRadius: W))
        let stars: [(CGFloat, CGFloat, CGFloat)] = [
            (-0.62, -0.30, 0.020), (-0.48, 0.42, 0.014), (0.55, -0.48, 0.017), (0.68, 0.22, 0.013),
            (-0.20, -0.72, 0.012), (0.30, 0.66, 0.015), (-0.78, 0.05, 0.011), (0.12, -0.80, 0.010),
        ]
        for (sx, sy, sr) in stars {
            ctx.fill(geo.circle(W * sr * 2, at: CGPoint(x: P.x + W * sx, y: P.y + W * sy)),
                     with: .color(theme.tertiary.color(0.85)))
        }

        // The moon: lit part from today's phase, dark part faintly visible.
        let mr = W * 0.62
        let disc = geo.circle(mr, at: P)
        ctx.fill(disc, with: .color(theme.tertiary.scaled(0.18).color(0.9)))

        let phase = Self.moonAge(date) / 29.530588853 // 0 new, 0.5 full
        let k = cos(2 * .pi * phase)
        let waxing = phase < 0.5
        var lit = Path()
        let n = 48
        for i in 0...n {
            let y = -mr + 2 * mr * CGFloat(i) / CGFloat(n)
            let limb = sqrt(max(0, mr * mr - y * y))
            let x = waxing ? limb : -limb
            let pt = CGPoint(x: P.x + x, y: P.y + y)
            if i == 0 { lit.move(to: pt) } else { lit.addLine(to: pt) }
        }
        for i in stride(from: n, through: 0, by: -1) {
            let y = -mr + 2 * mr * CGFloat(i) / CGFloat(n)
            let limb = sqrt(max(0, mr * mr - y * y))
            let x = waxing ? k * limb : -k * limb
            lit.addLine(to: CGPoint(x: P.x + x, y: P.y + y))
        }
        lit.closeSubpath()
        ctx.drawLayer { l in
            l.addFilter(.shadow(color: theme.tertiary.color(0.35), radius: u * 1.6))
            l.fill(lit, with: .radialGradient(
                Gradient(colors: [theme.tertiary.scaled(1.1).color(), theme.tertiary.scaled(0.72).color()]),
                center: CGPoint(x: P.x - mr * 0.3, y: P.y - mr * 0.35), startRadius: 0, endRadius: mr * 1.5))
        }
        // Craters, only where lit.
        var lc = ctx
        lc.clip(to: lit)
        let craters: [(CGFloat, CGFloat, CGFloat)] = [(-0.30, -0.25, 0.20), (0.25, 0.10, 0.14), (-0.05, 0.40, 0.16), (0.35, -0.40, 0.10), (-0.45, 0.20, 0.09)]
        for (cx, cy, cr) in craters {
            lc.fill(geo.circle(mr * cr, at: CGPoint(x: P.x + mr * cx, y: P.y + mr * cy)),
                    with: .color(theme.tertiary.scaled(0.6).color(0.35)))
        }

        drawRecessShadow(ctx, center: P, radius: W)
    }

    // MARK: - GMT

    func drawGMTRing(_ ctx: GraphicsContext) {
        let r1 = dR * 0.86, r2 = dR * 0.985
        var ring = geo.circle(r2)
        ring.addPath(geo.circle(r1))
        // Day (06–18) is the lower half: 24 sits at the top.
        let gradient = Gradient(stops: [
            .init(color: theme.secondary.color(), location: 0),
            .init(color: theme.secondary.color(), location: 0.5),
            .init(color: theme.tertiary.color(), location: 0.5),
            .init(color: theme.tertiary.color(), location: 1),
        ])
        ctx.fill(ring, with: .conicGradient(gradient, center: c), style: FillStyle(eoFill: true))
        // Printed-ring gloss.
        ctx.fill(ring, with: .conicGradient(WatchLight.sheen(.init(1, 1, 1), strength: 0.12), center: c),
                 style: FillStyle(eoFill: true))
        ctx.stroke(geo.circle(r1), with: .color(.black.opacity(0.5)), lineWidth: max(0.6, dR * 0.003))
        ctx.stroke(geo.circle(r2), with: .color(.black.opacity(0.4)), lineWidth: max(0.6, dR * 0.003))

        let mid = (r1 + r2) / 2
        for h in 1...24 {
            let deg = Double(h) / 24 * 360
            let day = h > 6 && h < 18
            // 6 and 18 sit on the day/night boundary: print them in the accent color.
            let ink = (h == 6 || h == 18) ? theme.accent : (day ? theme.tertiary : theme.print)
            if h % 2 == 0 {
                let p = geo.polar(mid, deg)
                var tc = ctx
                tc.translateBy(x: p.x, y: p.y)
                tc.rotate(by: .degrees(readableRotation(deg)))
                tc.draw(Text(h == 24 ? "24" : "\(h)")
                    .font(.system(size: dR * 0.050, weight: .semibold, design: .rounded))
                    .foregroundColor(ink.color(0.95)), at: .zero, anchor: .center)
            } else {
                ctx.fill(geo.circle(dR * 0.010, at: geo.polar(mid, deg)), with: .color(ink.color(0.9)))
            }
        }
    }

    /// 24-hour hand showing UTC, with an arrow tip on the 24-hour ring.
    func drawGMTHand(_ ctx: GraphicsContext, time: WatchTime) {
        var utc = Calendar(identifier: .gregorian)
        utc.timeZone = TimeZone(identifier: "UTC") ?? .current
        let comps = utc.dateComponents([.hour, .minute, .second], from: time.date)
        let hours = Double(comps.hour ?? 0) + Double(comps.minute ?? 0) / 60 + Double(comps.second ?? 0) / 3600
        let deg = hours / 24 * 360

        let L = dR * 0.85
        let w = dR * 0.010
        let head = dR * 0.055
        let shaftTop = -L + head * 0.9
        let shape = HandShape(
            full: polygon([
                CGPoint(x: 0, y: -L), CGPoint(x: head * 0.55, y: shaftTop), CGPoint(x: w / 2, y: shaftTop),
                CGPoint(x: w / 2, y: w * 3), CGPoint(x: -w / 2, y: w * 3), CGPoint(x: -w / 2, y: shaftTop),
                CGPoint(x: -head * 0.55, y: shaftTop),
            ]),
            left: polygon([CGPoint(x: 0, y: -L), CGPoint(x: -head * 0.55, y: shaftTop), CGPoint(x: -w / 2, y: shaftTop),
                           CGPoint(x: -w / 2, y: w * 3), CGPoint(x: 0, y: w * 3)]),
            right: polygon([CGPoint(x: 0, y: -L), CGPoint(x: 0, y: w * 3), CGPoint(x: w / 2, y: w * 3),
                            CGPoint(x: w / 2, y: shaftTop), CGPoint(x: head * 0.55, y: shaftTop)]),
            lume: nil, tip: CGPoint(x: 0, y: -L), base: CGPoint(x: 0, y: w * 3), ring: nil)
        drawHand(ctx, shape, deg: deg, height: 1.7,
                 light: theme.accent.scaled(1.25), dark: theme.accent.scaled(0.55))
    }
}
