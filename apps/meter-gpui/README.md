# Meter (GPUI edition)

A Rust rewrite of [Meter](../meter/): a Mac window plus a menu bar item showing how much of your AI tools' limits you have used. It reads the records Codex, Grok, Claude Code, Cursor and agy already keep on this Mac. Nothing leaves the Mac, no provider is called, and no prompt or reply text is kept.

Status: `wip`. Feature-complete for a first version, but **never launched** (see [AGENT_WORKLOG.md](AGENT_WORKLOG.md)). Test it in a macOS VM: [docs/vm-test-guide.md](docs/vm-test-guide.md).

## Stack

- **Rust** workspace: `meter-core` (readers, rollups, accounts, insights, alerts; no UI) and `meter-app` (the app).
- **GPUI** (Zed's UI framework, `gpui` 0.2.2 from crates.io) draws the window. The `runtime_shaders` feature compiles Metal shaders when the app starts, so building needs no Metal toolchain.
- **AppKit through `objc2`** supplies what GPUI does not: the menu bar item (`NSStatusItem` + `NSMenu`), notifications (`UNUserNotificationCenter`) and the login item (`SMAppService`).

## What it shows

| Tool | From this Mac |
|---|---|
| Codex | quota percent and reset (Session, Weekly), tokens, model, project |
| Grok | weekly credit percent and reset, tokens, model, project |
| Claude Code | tokens by model and project (no limit is recorded) |
| Cursor (including Grok Bot) | turns by model (no tokens or limit recorded) |
| agy | steps per conversation, subagents folded into their parent |

- **Window:** Overview (a card per tool with limits, pace and today's usage), Usage (daily stacked chart, tokens or turns, 7/14/30 days), Accounts (add, hide, remove), Settings (general and alerts).
- **Menu bar:** gauge icon plus the tightest limit percent; a native menu with up to three lines, a warning only when one is true, Open Meter…, Refresh Now, Settings…, Quit. With no recorded limit it shows tokens (or turns) today. Closing the window keeps Meter running; the Dock icon and Dock menu reopen it.
- **Alerts** (all off until you turn one on): filling up (80% and 95% by default), used up, reset, may run out early. Quiet hours and snooze. Each fires once per window, and a limit already high at launch stays quiet.
- **Accounts:** several per tool, each with a data folder; hidden accounts keep their history but leave the menu, notes and alerts.

## Build and test

macOS 14+, Rust 1.85+ (tested with 1.99).

```sh
cargo test --workspace            # 43 tests; fixtures only, never reads your real tool folders
cargo run -p meter-core --bin meter-probe   # read-only check of the readers on your real data; counts only
./scripts/bundle.sh               # builds build/Meter.app (does not launch or install it)
./scripts/make-vm-kit.sh          # builds build/vm-kit/ to copy into a test VM by hand
```

`meter-probe --fake-home <dir>` writes invented tool data for demos and tests.

## What running Meter changes on your Mac

- Files: `~/Library/Application Support/MeterGPUI/{settings,accounts,rollup}.json` (settings, your account labels, per-day usage totals; no text). Meter itself uses no UserDefaults, but GPUI/AppKit write their own window-state preferences file (observed: `~/Library/Preferences/meter.plist` for the bare `cargo run` binary; `com.king.meter-gpui.plist` is expected for `Meter.app`). Short-lived `meter-snap-*` folders in the temp directory while reading a database, deleted right after.
- Only if you ask: a notification permission entry (when you turn an alert on) and a Login Items entry (when you turn on "Open Meter at login"). Both are off by default and do nothing in a bare binary.
- It shows a Dock icon and a menu bar item while running. macOS may also record the app in Launch Services.
- It does not touch the Keychain, install helpers or launch agents, use the network, or write to the other tools' folders. Their files are only read, and SQLite databases are read from a temporary copy.
- To remove it: quit, delete `Meter.app`, `~/Library/Application Support/MeterGPUI` and the Meter plist in `~/Library/Preferences`, and remove any Login Items or Notifications entry.

## Trying it without touching your Mac

- **macOS VM (recommended):** `./scripts/make-vm-kit.sh`, then follow the guide.
- **Automated VM runs (Tart):** [docs/vm-automation.md](docs/vm-automation.md). `./scripts/tart/up.sh` then `./scripts/tart/suite.sh` boots a throwaway macOS VM (from a restore image you already have), runs the app in it, clicks through it and brings screenshots back.
- **Sandbox run:** `./scripts/sandbox.sh` runs the binary against invented data with `HOME` and `METER_SUPPORT_DIR` pointed at a throwaway folder. The Dock icon and menu bar item still appear while it runs.

## Differences from the Swift edition

Same readers, rollups, insights and alert rules (ported and tested the same way). Different UI layer:

- The window opens at about four fifths of the screen it is on (within limits, leaving room for the menu bar and Dock) and its cards reflow when you resize it.
- Settings is a page in the window (⌘, opens it), not a separate Settings window.
- Settings are a JSON file in `MeterGPUI`, not UserDefaults.
- Accounts are added by choosing a folder (label = folder name); there is no text field yet.
- The Dock menu has Open Meter and Refresh Now but not the status lines (GPUI Dock menus hold actions only).
- GPUI has little accessibility support, so VoiceOver reads the window far less well than the SwiftUI version. The menu bar item and its menu are native AppKit and are labeled.
- The usage chart has no hover tooltips; day totals are printed above the bars.

## Layout

- `crates/meter-core`: `readers/`, `rollup`, `accounts`, `insights`, `alerts`, `dashboard`, `fakehome`, `bin/meter-probe`. Tests in `tests/`.
- `crates/meter-app`: `main.rs` (actions, windows, menus, refresh loop), `state.rs`, `ui.rs`, `widgets.rs`, `theme.rs`, `settings.rs`, `statusbar.rs` (AppKit), `system.rs` (AppKit notifications and login item).
- `assets/`: `AppIcon.icns` and `AppIcon.png` (rebuild with `scripts/make-icon.sh`).
- `docs/`: [vm-automation.md](docs/vm-automation.md), [vm-test-guide.md](docs/vm-test-guide.md).

## Privacy rules

Readers never select titles, prompts, replies or log bodies, and never open credential files. Account labels are yours; Meter never reads an email or token.

## Limits

Claude Code, Cursor and agy record no quota locally, so there is no percent for them. agy token counts live in an undocumented binary format and are not read. Refresh is a timer (default 1 minute), not a file watcher.
