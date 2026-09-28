#!/usr/bin/env python3
"""Draw the toolbar icons. Stdlib only."""

import struct
import zlib
from pathlib import Path

INK = (28, 25, 21, 255)
PAPER = (246, 241, 231, 255)
CLEAR = (0, 0, 0, 0)


def chunk(tag: bytes, data: bytes) -> bytes:
    return (
        struct.pack(">I", len(data))
        + tag
        + data
        + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
    )


def write_png(path: Path, size: int, pixels: list[tuple[int, int, int, int]]) -> None:
    rows = []
    for y in range(size):
        row = bytearray([0])
        for x in range(size):
            row.extend(pixels[y * size + x])
        rows.append(bytes(row))
    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(b"".join(rows), 9))
        + chunk(b"IEND", b"")
    )
    path.write_bytes(png)


def rounded(size: int) -> list[tuple[int, int, int, int]]:
    radius = max(2, round(size * 0.22))
    pixels: list[tuple[int, int, int, int]] = []
    for y in range(size):
        for x in range(size):
            dx = 0 if radius <= x < size - radius else (radius - x if x < radius else x - (size - 1 - radius))
            dy = 0 if radius <= y < size - radius else (radius - y if y < radius else y - (size - 1 - radius))
            inside = dx * dx + dy * dy <= radius * radius
            pixels.append(INK if inside else CLEAR)
    return pixels


def fill_rect(pixels: list, size: int, x0: int, y0: int, x1: int, y1: int, color) -> None:
    for y in range(max(0, y0), min(size, y1)):
        for x in range(max(0, x0), min(size, x1)):
            pixels[y * size + x] = color


def draw(size: int) -> list[tuple[int, int, int, int]]:
    pixels = rounded(size)
    # A paper "F": stem plus two bars. Coordinates are fractions of the icon.
    stem_x = round(size * 0.30)
    stem_w = max(2, round(size * 0.14))
    top = round(size * 0.24)
    bottom = round(size * 0.76)
    bar_h = max(2, round(size * 0.13))
    fill_rect(pixels, size, stem_x, top, stem_x + stem_w, bottom, PAPER)
    fill_rect(pixels, size, stem_x, top, round(size * 0.74), top + bar_h, PAPER)
    mid = round(size * 0.46)
    fill_rect(pixels, size, stem_x, mid, round(size * 0.64), mid + bar_h, PAPER)
    return pixels


def main() -> None:
    out = Path(__file__).resolve().parents[1] / "shared" / "icons"
    out.mkdir(exist_ok=True)
    for size in (16, 32, 48, 128):
        write_png(out / f"icon{size}.png", size, draw(size))


if __name__ == "__main__":
    main()
