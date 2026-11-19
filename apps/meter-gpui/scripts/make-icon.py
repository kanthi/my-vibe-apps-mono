"""Draws the Meter app icon (a gauge on a rounded square) as a 1024x1024 PNG.
Run through scripts/make-icon.sh, which also builds assets/AppIcon.icns."""
import math
import sys

from PIL import Image, ImageChops, ImageDraw, ImageFilter

SIZE = 1024
SS = 2  # supersampling for smooth edges
S = SIZE * SS


def lerp(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def gradient(top, bottom):
    img = Image.new("RGB", (S, S))
    px = img.load()
    for y in range(S):
        c = lerp(top, bottom, y / (S - 1))
        for x in range(S):
            px[x, y] = c
    return img


def rounded_mask(box, radius):
    m = Image.new("L", (S, S), 0)
    ImageDraw.Draw(m).rounded_rectangle(box, radius=radius, fill=255)
    return m


def main(out):
    # macOS icon grid: the body is 824 of 1024, centered.
    pad = int(100 * SS)
    box = (pad, pad, S - pad, S - pad)
    radius = int(185 * SS)
    body_mask = rounded_mask(box, radius)

    canvas = Image.new("RGBA", (S, S), (0, 0, 0, 0))

    # Soft drop shadow under the body.
    shadow = Image.new("L", (S, S), 0)
    ImageDraw.Draw(shadow).rounded_rectangle((pad, pad + 14 * SS, S - pad, S - pad + 14 * SS), radius=radius, fill=150)
    shadow = shadow.filter(ImageFilter.GaussianBlur(22 * SS))
    canvas.paste(Image.new("RGBA", (S, S), (0, 0, 0, 255)), (0, 0), shadow)

    # Body: deep blue to teal.
    body = gradient((52, 130, 255), (9, 38, 110)).convert("RGBA")
    canvas.paste(body, (0, 0), body_mask)

    # A faint highlight at the top that fades out smoothly (no hard edge).
    fade = Image.new("L", (S, S), 0)
    fp = fade.load()
    top, bottom = pad, pad + int(420 * SS)
    for y in range(top, bottom):
        v = int(70 * (1 - (y - top) / (bottom - top)) ** 2)
        for x in range(S):
            fp[x, y] = v
    hi = ImageChops.multiply(fade, body_mask)
    canvas.paste(Image.new("RGBA", (S, S), (255, 255, 255, 255)), (0, 0), hi)

    def layer():
        return Image.new("RGBA", (S, S), (0, 0, 0, 0))

    cx, cy = S // 2, int(S * 0.55)
    r = int(250 * SS)
    stroke = int(58 * SS)
    box_arc = (cx - r, cy - r, cx + r, cy + r)
    start, sweep, value = 135, 270, 0.62  # PIL angles: 0 = 3 o'clock, clockwise
    end = start + sweep * value
    rr = r - stroke / 2

    def cap(d, ang, fill):
        x = cx + rr * math.cos(math.radians(ang))
        y = cy + rr * math.sin(math.radians(ang))
        d.ellipse((x - stroke / 2, y - stroke / 2, x + stroke / 2, y + stroke / 2), fill=fill)

    # Unused part of the dial: translucent white on its own layer, then blended.
    track = layer()
    td = ImageDraw.Draw(track)
    faint = (255, 255, 255, 80)
    td.arc(box_arc, start, start + sweep, fill=faint, width=stroke)
    cap(td, start, faint)
    cap(td, start + sweep, faint)
    canvas = Image.alpha_composite(canvas, track)

    # Used part of the dial, the needle and the hub: solid white.
    solid = layer()
    sd = ImageDraw.Draw(solid)
    white = (255, 255, 255, 255)
    sd.arc(box_arc, start, end, fill=white, width=stroke)
    cap(sd, start, white)
    cap(sd, end, white)
    ang = math.radians(end)
    nl = r * 0.78
    nx, ny = cx + nl * math.cos(ang), cy + nl * math.sin(ang)
    sd.line((cx, cy, nx, ny), fill=white, width=int(34 * SS))
    sd.ellipse((nx - 17 * SS, ny - 17 * SS, nx + 17 * SS, ny + 17 * SS), fill=white)
    hub = int(46 * SS)
    sd.ellipse((cx - hub, cy - hub, cx + hub, cy + hub), fill=white)
    sd.ellipse((cx - hub // 2, cy - hub // 2, cx + hub // 2, cy + hub // 2), fill=(30, 80, 190, 255))
    canvas = Image.alpha_composite(canvas, solid)

    canvas.resize((SIZE, SIZE), Image.LANCZOS).save(out)


if __name__ == "__main__":
    main(sys.argv[1])
