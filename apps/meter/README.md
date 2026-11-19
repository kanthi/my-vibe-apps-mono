# Meter

A Mac app and menu-bar item that show how much of your AI tools' limits you have used. It reads the records Codex, Grok, Claude Code, Cursor and agy already keep on this Mac. Nothing leaves the Mac, no provider is called, and no prompt or reply text is kept.

Status: `wip`. Feature-complete for the first version; not yet launched or installed on the author's Mac by design (see AGENT_WORKLOG.md).

## What it shows

| Tool | From this Mac |
|---|---|
| Codex | quota percent and reset (Session, Weekly), tokens, model, project |
| Grok | weekly credit percent and reset, tokens, model, project |
| Claude Code | tokens by model and project (no limit is recorded) |
| Cursor (including Grok Bot) | turns by model (no tokens or limit recorded) |
| agy | steps per conversation, subagents folded into their parent |

- **Window:** Overview (a card per tool with limits, pace and today's usage), Usage (daily chart, tokens or turns), Accounts (add, hide, remove).
- **Menu bar:** the tightest limit as an icon plus percent, up to three lines, a warning only when one is true. If no tool records a limit it shows tokens (or turns) today. Closing the window keeps Meter running; there is also a Dock menu.
- **Alerts** (all off until you turn one on): filling up (80% and 95% by default), used up, reset, may run out early. Quiet hours and snooze. Each fires once per window.
- **Accounts:** several per tool, each with your own label and data folder, hidden accounts keep their history.

## Build and test

macOS 14+, Xcode toolchain, no dependencies.

```sh
swift test                  # fixtures only; never reads your real tool folders
swift run meter-probe       # read-only check of the readers on your real data; prints counts only
./scripts/bundle.sh         # builds build/Meter.app (does not launch or install it)
```

`METER_SNAPSHOT_DIR=<dir> swift test` also writes PNG renders of each screen (drawn in a window that is never shown).

## Trying it without touching your Mac

- **macOS VM (recommended):** `./scripts/make-vm-kit.sh` builds `build/vm-kit/` with the app, a README checklist and a script that fills the VM with invented tool data (it refuses to run outside a VM).
- **Sandbox run:** `./scripts/sandbox.sh` runs the app against invented data in a throwaway folder. Not a full guarantee: the Dock icon and menu bar item appear and AppKit may write one window-position preferences file, which the script deletes on exit.

## What running Meter changes on your Mac

- Files: `~/Library/Application Support/Meter/accounts.json` and `rollup.json` (usage totals per day, no text), and `~/Library/Preferences/<bundle id>.plist` for settings and window positions. Short-lived `meter-snap-*` copies in the temp folder while reading a database, deleted right after.
- Only if you ask: a notification permission entry (when you turn an alert on) and a Login Items entry (when you turn on "Open Meter at login"). Both are off by default.
- It shows a Dock icon and a menu bar item while running. macOS may also record the app in its Launch Services list.
- It does not touch the Keychain, install LaunchAgents or helpers, use the network, or write to the other tools' folders. Other tools' files are only read.
- To remove it: quit, delete `Meter.app`, delete `~/Library/Application Support/Meter`, run `defaults delete <bundle id>`, and remove any Login Items or Notifications entry.
- macOS might ask permission the first time Meter reads another app's data folder; I could not test that without launching it.

## Layout

- `Sources/MeterCore`: readers, rollup store, accounts, insights, alert engine, dashboard (no UI).
- `Sources/MeterUI`: observable app model and SwiftUI views.
- `Sources/Meter`: the `@main` app. `Sources/meter-probe`: read-only probe.
- Docs: [VM test guide](docs/vm-test-guide.md), [product](docs/product.md), [design](docs/design.md), [reference study](docs/reference-study.md), [AGENT_WORKLOG.md](AGENT_WORKLOG.md).

## Privacy rules

Readers never select titles, prompts, replies or log bodies, and never open credential files. SQLite databases that other apps keep open are read from a temporary copy, never the live file. Account labels are yours; Meter never reads an email or token.

## Limits

Claude Code, Cursor and agy record no quota locally, so Meter shows no percent for them (a live-quota mode that reuses their logins was considered and deferred). agy token counts live in an undocumented binary format and are not read. Refresh is a timer (default 1 minute), not a file watcher.
