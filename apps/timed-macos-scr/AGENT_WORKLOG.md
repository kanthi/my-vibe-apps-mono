# timed-macos-scr agent worklog

## 2026-11-14: preview run, stale build found

**Request:** "preview it for me".

**Found**
- `scripts/preview.sh` failed: `.build/release` held a `TimedClockCore` module built with SDK macosx27.0, and the installed SDK is now macosx27.2. The script skips `swift build` whenever `TimedClockCore.o` exists, so it never rebuilds after an SDK update.
- Fixed locally by running `swift build -c release` and recompiling `.build/preview_runner`. No source or script changes.
- The preview then ran fine (v0.008 Bauhaus clock). It quits on any key or click other than Space/T, not only Esc/Q as the README says.

**Status:** builds and previews on macOS 27.2. Not installed by the agent. Nothing committed.

**Suggested next steps (not done, not yet approved)**
1. `scripts/preview.sh`: always run `swift build -c release` (it is incremental, a few seconds) so SDK updates cannot break the preview.
2. `scripts/preview_runner.swift`: quit only on Esc/Q, ignore other keys and clicks.
3. Root README catalog row still describes v0.007 ("top-right digital HUD"); v0.008 is the Bauhaus clock.

## 2026-11-14: v0.009, realistic dark watch faces

**Request:** "add more variations with dark theme, more 3d effect, like realistic watch".

**Done**
- `Sources/TimedClockCore/RealisticWatchTheme.swift`: `RGB` helper, `RealisticWatchTheme` with four dark faces (Obsidian, Midnight, Forest, Abyss diver), and `WatchFace` (the Bauhaus clock plus the four).
- `Sources/TimedClockCore/RealisticWatchView.swift`: Canvas renderer. One top-left light shades everything: hand/marker facets by angle, screen-space cast shadows scaled by hand height, conic metal gradients for case/bezel/rehaut/hub, sunburst rays plus bow-tie sheen, rehaut shadow crescent, applied markers with lume glow, date window, crystal glare. Three layers (static case/dial and crystal are `Equatable` and do not redraw; only hands + date redraw at 60 FPS). `TimedClockFaceView` switches faces.
- Saver: Options… sheet with a face popup, stored in `ScreenSaverDefaults` (key `face`, default Bauhaus, so existing installs look the same).
- Preview: arrows / 1–5 switch faces, Space/T toggles Bauhaus light/dark, only Esc/Q quit (clicks and other keys ignored). `preview.sh` now always runs `swift build` (fixes the stale-SDK failure). `render_preview.swift` renders every face and takes an output folder.
- Version 0.009 in `build.sh`; README updated. Faces are original designs (generic watch vocabulary), not copies of real brands.

**Verified:** `swift build -c release`, `scripts/build.sh` (dist/TimedClock.saver) and the preview build all succeed. Rendered all faces to PNG and inspected them; shortened lugs and hand tails after the first look. Preview launched for the user.

**Not verified:** the Options sheet inside System Settings (needs the saver installed; not installed by the agent). Resources/ preview PNGs not regenerated for the new faces (would add ~1–2 MB per image).

**Still open:** root README catalog row still describes v0.007.

## 2026-11-14: preview tour mode

- `preview_runner.swift` takes `--face <name>` (start on a face) and `--cycle <secs>` (tour every face in order, looping). `preview.sh` passes its arguments through.
- Ran `./scripts/preview.sh --cycle 10` for the user; it built, ran and exited normally (Esc/Q). Nothing committed.

## 2026-11-14: full-screen dials, Bauhaus Lume face

**Request:** "do not add watch here, just dials that cover full screen to max, whichever screen they run, dynamically. also create something close to original that was there before you started".

**Done**
- Removed the watch body (case, lugs, strap, crown, bezel, rehaut, crystal). The dial surface (gradient, sunburst rays, bow-tie sheen or matte highlight, corner vignette) now fills the whole screen; markers, numerals and hands scale to `min(width, height) / 2 * 0.96`, so any screen size or orientation works (checked 1440×900 and 900×1440 renders).
- New face **Bauhaus Lume (3D)** (`bauhausLume`): the original layout (upright 1–12, pill batons, 4 ticks between, tangential 05–55 ring, pencil hands with lume groove, original proportions) rendered with 3D facets, cast shadows and cyan glow. No date, no brand text, like the original.
- The original Bauhaus face is untouched and still the default. Faces are now 6; preview keys 1–6.
- Theme model simplified (no case/strap/bezel fields); added `HandStyle` (sword / pencil), `showsDate`, `brand`, `numeral`.

**Verified:** library, saver bundle and preview build; all faces rendered and inspected.

## 2026-11-14: v0.010, structurally different dials

**Request:** "create complete different watch faces (just like existing ones, only dials) instead of just color changes, make them more realistic".

**Done**
- Replaced the color variants Midnight and Forest with four dials that differ in layout and complications. Faces are now 8: Bauhaus (original), Bauhaus Lume, Obsidian (dress), Pilot, Chronograph, Lunar (moon phase), Meridian (GMT), Abyss (diver). A saved face of `midnight`/`forest` falls back to Bauhaus.
- Theme model: `Layout` (bauhaus, dress, diver, pilot, chronograph, lunar, gmt), `Surface` (sunburst, matte, guilloche), `HandStyle` (sword, pencil, stick, breguet), plus `secondary`/`tertiary` colors and `dateRadius`.
- New `Sources/TimedClockCore/DialComplications.swift`: pilot printing (triangle + dots, numerals), chronograph (tachymeter, 1/5 s track, recessed azurage sub-dials with bevel rings, live sub-dial hands: running seconds, 30-min counter, 12-h counter), moon phase (live phase from a known new moon + synodic month, sky, craters, dark-side earthshine, framed recess), GMT (two-tone 24 h ring, UTC hand).
- Core renderer: guilloché surface, recess shadows, bevel rings, arrowhead markers, stick and Breguet hands (open ring), hands can pivot on a sub-dial.
- Fixes after inspection: GMT 6/18 labels on the day/night boundary printed in the accent color; guilloché contrast raised.
- Version 0.010; README and preview keys (1–8) updated.

**Verified:** library, saver bundle and preview build; all new faces rendered at 1440×900 and inspected. Nothing committed.
