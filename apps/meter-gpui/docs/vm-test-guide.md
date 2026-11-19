# Testing Meter (GPUI edition) in a macOS VM by hand

> There is now an automated route that needs no clicking: see [vm-automation.md](vm-automation.md). This guide is for trying the app yourself in a VM you already have.

The Rust app compiles, its logic has 37 passing tests, and a headless GPUI test draws every page without panicking. But it has now been run in a Tart VM (see vm-automation.md), but not on a real Mac or by you. A VM lets you run it, and see it, without touching your real Mac.

## 1. Build the kit (on your Mac)

```sh
cd apps/meter-gpui
./scripts/make-vm-kit.sh
```

This writes `build/vm-kit/` (git-ignored) with `Meter.app` (ad-hoc signed, arm64, not notarized), `meter-probe`, `setup-fake-data.sh` and a short README. It only creates files in this repo. The first build downloads and compiles GPUI and takes a few minutes.

## 2. In the VM

Copy the `vm-kit` folder in, then:

```sh
cd <path to vm-kit>
xattr -dr com.apple.quarantine .
./setup-fake-data.sh
```

`setup-fake-data.sh` checks `sysctl kern.hv_vmm_present` and refuses on a real Mac (tested: it refuses here). The fake data is relative to the moment you run it, so re-run it before each session.

| Tool | Fake data |
|---|---|
| Codex | Session 34%, Weekly 62%, two threads, model `gpt-demo` |
| Grok | Weekly 18%, one session, model `grok-demo` |
| Claude Code | one session, about 24k tokens, no quota |
| Cursor | one conversation, 6 turns, model `grok-demo` |
| agy | one conversation, 64 steps |

Open `Meter.app` (right-click > Open if macOS warns).

## 3. Checklist

Launch
- [ ] The window opens and nothing crashes. GPUI compiles its Metal shaders on the first launch, so the first frame may take a moment.
- [ ] A menu bar item appears with a gauge icon and a percent (Codex Weekly 62% should lead).
- [ ] Text uses the system font and is crisp on a Retina display.

Menu bar and Dock
- [ ] The status menu shows up to 3 lines (for example Codex Weekly 62%, Codex Session 34%, Grok Weekly 18%), then Open Meter…, Refresh Now, Settings…, Quit Meter. ⌘R, ⌘, and ⌘Q work while the menu is open.
- [ ] A warning line appears only if a pace note is true.
- [ ] Close the window (red button or ⌘W). The app stays in the menu bar and the Dock. Open Meter… from the menu, the Dock icon click, and the Dock menu each bring the window back.
- [ ] The app menu bar has Meter (Settings…, Services, Quit), View (Refresh Now) and Window (Close Window).

Window
- [ ] Overview: cards for the installed tools, bars colored by pace and always with words (✓ On course, ⚠︎ Tight, ✕ Runs out…). Claude Code, Cursor and agy say no limit is recorded. "Not installed" never shows a zero.
- [ ] Usage: the chart draws stacked bars; Tokens and Turns, and 7, 14 and 30 days, switch correctly; day labels are readable.
- [ ] Accounts: the Shown/Hidden switch hides an account and the menu percent changes. "Choose data folder…" opens the native folder picker; the new account appears with the folder name; Remove deletes it.
- [ ] Settings: toggles, refresh interval, thresholds and quiet hours respond; snooze shows its end time.
- [ ] Light and dark mode both read well (switch the VM's appearance while the window is open; it should follow).
- [ ] Window resizes down to its minimum without clipping.

System changes (only when you ask for them)
- [ ] Nothing asks for permission at launch.
- [ ] Turn an alert on: macOS asks for notification permission at that moment.
- [ ] Turn "Open Meter at login" on: an entry appears under System Settings > General > Login Items. Turn it off: it disappears.
- [ ] Quit and relaunch: settings and hidden accounts persist.

Files
- [ ] `~/Library/Application Support/MeterGPUI/` has `settings.json`, `accounts.json`, `rollup.json`.
- [ ] No `com.king.meter-gpui.plist` in `~/Library/Preferences` (GPUI may still create one for window state; note it if it does).

## 4. What to tell me afterwards

- Anything that failed, with a screenshot if it is visual. GPUI layout is the likeliest place for surprises (wrapping, overflow, text clipping, hover and click targets).
- Whether macOS asked for any permission you did not expect.
- Whether the menu bar icon and percent looked right.
- Whether a refresh felt slow, or the app used noticeable CPU while idle.

## Known limits (not bugs to report)

- No text field: accounts are labeled by their folder name.
- VoiceOver support in the window is limited by GPUI.
- No chart hover tooltips. The Dock menu has no status lines.
- Claude Code, Cursor and agy have no quota percent. agy tokens are not read.

## Cleanup

Delete the VM. On a real Mac, `build/vm-kit/` can be deleted any time.
