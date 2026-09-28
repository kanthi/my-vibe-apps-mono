# timed-macos-scr

An aesthetic, borderless minimalist **Bauhaus Horological Clock** screen saver for macOS, faithfully replicating the acclaimed clock design by [Atilla Taşkıran (bauhausclock.com)](https://bauhausclock.com).

Engineered strictly as a native macOS Screen Saver (`.saver` bundle) conforming to Apple's official `ScreenSaverView` framework, featuring adaptive **Glacier White Light Mode** and **Swiss BGW9 Electric Cyan Dark Mode**, dynamic full-screen vector geometry, and real-time live clock animation with a continuous 60 FPS sweeping second hand.

![Bauhaus Clock Screen Saver Preview](Resources/preview_duo.png)

---

## Features

- **Version**: `0.008`
- **Strictly a Native macOS Screen Saver**:
  - Compliant with Apple's official `ScreenSaver` framework (`ScreenSaverView` lifecycle with `init(frame:isPreview:)`, `startAnimation()`, `stopAnimation()`, and 60 FPS `animateOneFrame()`).
  - Pure native `.saver` bundle with zero companion applications, helper daemons, or external runtime dependencies.
- **Adaptive Day & Night Horological Dial**:
  - **Light Mode (Glacier Ceramic White)**: Soothing off-white ceramic canvas (`#EAEFF2`), tactile 3D embossed ice-cyan pill batons with beveled highlights, crisp graphite Bauhaus typography, and brushed titanium hands with central lume channels.
  - **Dark Mode (Swiss BGW9 Super-LumiNova)**: Deep obsidian OLED pitch black (`#050709`), radiant electric cyan lume (`#00F2FF`) with ambient bloom on numerals, batons, and hands, alongside muted deep-slate minute ticks.
  - Automatically adapts to your macOS system appearance (`Dark Mode` / `Light Mode`).
- **4 Concentric Horological Tiers**:
  1. **Tier 1: Inner Upright Bauhaus Hour Numerals (1–12)**: Upright, centered numerals in rounded Bauhaus typography with circular bowls and balanced geometry.
  2. **Tier 2: 3D Embossed Hour Batons (12 Pills)**: Radial capsule batons at each hour mark, shaded with realistic directional lighting (white highlight on top-left, drop shadow on bottom-right) in light mode, and high-intensity luminous core with bloom in dark mode.
  3. **Tier 3: Precision Minute Track**: 4 thin radial tick marks neatly recessed between each hour baton.
  4. **Tier 4: Outer 5-Minute Numerals (60, 05, 10, ... 55)**: Tangentially aligned 5-minute numerals following the dial circumference with upright readability.
- **Sculpted Hands & Silky 60 FPS Sweep**:
  - **Hour & Minute Hands**: Tapered pencil shape with split 3D longitudinal chamfers (light flank vs shadow flank) and recessed luminescent central channels.
  - **Sweeping Second Hand**: Ultra-fine needle gliding continuously at 60 FPS, extending through the outer 5-minute ring with a rounded counterweight tail.
  - **Center Pinion**: Turned metal collet washer with an open dark center aperture and second hand pivot cap.
- **Interactive Full-Screen Preview**:
  - Instant ephemeral preview via `./scripts/preview.sh`.
  - Press `Space` or `T` while previewing to toggle between Light and Dark mode.
  - Press `ESC` or `Q` to exit.

---

## Usage & Installation

### Requirements
- macOS 14.0 (Sonoma), macOS 15.0 (Sequoia), macOS 27 or later.
- Apple Silicon (M1/M2/M3/M4) or Intel Mac.
- Swift 6.0+ / Xcode Command Line Tools.

### Quick Commands

```bash
# 1. Build the release screensaver bundle (dist/TimedClock.saver)
./scripts/build.sh

# 2. Test live in full-screen without installing (Press SPACE to toggle Light/Dark, ESC to exit)
./scripts/preview.sh

# 3. Install into macOS System Settings > Screen Savers
./scripts/install.sh
```

---

## Project Structure

```text
timed-macos-scr/
├── Package.swift               # Swift Package manifest (TimedClockCore)
├── Sources/
│   ├── TimedClockCore/         # Scalable vector views, themes, and live time model
│   │   ├── TimedTheme.swift    # BauhausDialTheme (Light & Dark color palettes & tokens)
│   │   ├── TimedClockModel.swift # 60 FPS high-precision time publisher (hours, minutes, seconds)
│   │   └── TimedClockMainView.swift # Full-screen Bauhaus clock orchestrator
│   └── TimedClockSaver/        # Native Apple ScreenSaverView host
│       └── TimedClockScreenSaverView.swift
├── Resources/                  # High-resolution screenshot renders
│   ├── preview_light.png       # Glacier White daylight preview
│   ├── preview_dark.png        # Swiss BGW9 cyan night preview
│   └── preview_duo.png         # Side-by-side comparison preview
├── scripts/
│   ├── build.sh                # Compiles TimedClock.saver Mach-O bundle
│   ├── install.sh              # 1-click installer into ~/Library/Screen Savers/
│   ├── preview.sh              # Non-intrusive full-screen live preview
│   ├── preview_runner.swift    # Ephemeral test runner with Spacebar Light/Dark toggle
│   └── render_preview.swift    # High-resolution vector renderer for documentation
└── README.md
```

---

## Credits

Replication of the **Bauhaus Clock** design by **[Atilla Taşkıran](https://bauhausclock.com)**.
Engineered and packaged natively for macOS by **Antigravity**.
