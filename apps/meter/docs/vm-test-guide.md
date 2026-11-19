# Testing Meter in a macOS VM

Written 2026-10-24 for a test session on 2026-10-25. Nothing in this guide touches your real Mac except building files inside this repo.

## Why a VM

Meter has never been launched. Its logic is covered by 38 passing tests, but the parts that need a real desktop have not run: the menu bar item, the Dock menu, notifications, the login item, and settings persistence. A VM lets you run all of those without changing your own macOS. VMware Fusion is installed on this Mac; whether it can run a macOS guest depends on its version, which has not been checked. Any other macOS VM tool works too.

## 1. Build the kit (on your Mac)

```sh
cd apps/meter
./scripts/make-vm-kit.sh
```

This writes `build/vm-kit/` (git-ignored, inside the repo) with:

| File | Purpose |
|---|---|
| `Meter.app` | The app, ad-hoc signed, for Apple silicon. Not notarized. |
| `meter-probe` | Writes invented tool data into the home folder it runs in. |
| `setup-fake-data.sh` | Runs `meter-probe --fake-home "$HOME"`. Refuses to run outside a VM. |
| `README.txt` | Short checklist, same as below. |

Build requirements: macOS 14+, Xcode toolchain. If the kit is missing a file, re-run the script.

## 2. Get the kit into the VM

Use a shared folder, AirDrop, or drag and drop. Copy the whole `vm-kit` folder.

## 3. Set up inside the VM

```sh
cd <path to vm-kit>
xattr -dr com.apple.quarantine .
./setup-fake-data.sh
```

- `xattr` removes the download quarantine flag so the app can open.
- `setup-fake-data.sh` checks `sysctl kern.hv_vmm_present` and stops if it is not 1. I tested this on the real Mac: it refused and wrote nothing.
- The fake data covers Codex, Grok, Claude Code, Cursor and agy. Quotas and timestamps are relative to the moment you run it, so re-run it before each session. If you test across several days, the quota windows will drift.

Fake data you should see:

| Tool | What the fake data holds |
|---|---|
| Codex | Session 34%, Weekly 62%, two threads, model `gpt-demo` |
| Grok | Weekly 18%, one session, model `grok-demo` |
| Claude Code | One session, about 24k tokens, no quota |
| Cursor | One conversation, 6 turns, model `grok-demo`, no quota |
| agy | One conversation, 64 steps, no quota |

## 4. Open the app

Right-click `Meter.app` and choose Open (macOS warns because it is not notarized). The first launch is the one I have not been able to watch, so note anything odd.

## 5. Checklist

Menu bar and Dock
- [ ] A menu bar item appears: gauge icon plus the tightest percent.
- [ ] Click it. The menu shows up to 3 lines, then Open Meter…, Refresh Now, Settings…, Quit Meter.
- [ ] A warning line appears only when a pace or out-of-pattern note is true.
- [ ] Close the main window. The app keeps running: menu bar item and Dock icon stay.
- [ ] Dock menu (right-click or hold the Dock icon) shows the same lines plus Open Meter and Refresh Now. Open Meter reopens the window.
- [ ] Settings… opens with ⌘, and Quit works from the menu bar and the Dock.

Window
- [ ] Overview: a card per tool. Codex has Session and Weekly bars, Grok has Weekly. Claude Code, Cursor and agy say no limit is recorded and show tokens or turns. Nothing shows a made-up zero.
- [ ] Bar colors always come with words and a symbol (On course, Tight, Runs out in…).
- [ ] Usage: chart switches between Tokens and Turns, and between 7, 14 and 30 days. Day labels are readable.
- [ ] Accounts: hide Codex, and the menu percent changes. Show it again.
- [ ] Add an account with a name and a data folder (the Choose… panel). It appears under its tool. Remove it.
- [ ] Refresh button works, and a failed read keeps the last values with an Outdated tag after about 10 minutes.

Settings
- [ ] General: turn off "Show percent in the menu bar" and the percent disappears. Refresh interval changes.
- [ ] Alerts: all four are off by default. Turn one on. macOS should ask for notification permission at that moment, not before.
- [ ] Quiet hours and Snooze appear and can be set.
- [ ] "Open Meter at login": turning it on should add an entry under System Settings > General > Login Items. Turn it off and confirm it is removed.

Persistence
- [ ] Quit, relaunch. Settings and hidden accounts are still as you left them.

Accessibility and appearance
- [ ] Light and dark mode both read well.
- [ ] Increase Contrast and Reduce Motion do not break anything.
- [ ] VoiceOver reads the gauges (for example "Codex Weekly, 62 percent used") and the menu bar item.

Where Meter writes (check these in the VM)
- `~/Library/Application Support/Meter/accounts.json` and `rollup.json`
- `~/Library/Preferences/` for settings and window positions
- Short-lived `meter-snap-*` folders in the temp directory while it reads a database

## 6. What to tell me afterwards

Please report:
- Anything that failed from the checklist, with a screenshot if it is visual.
- Whether macOS asked for any permission I did not expect, for example "Meter would like to access data from other apps".
- Whether the menu bar label looked right (icon plus percent), since I could only reason about it from the code.
- Whether notifications arrived once and only once per window, if you triggered an alert.

## Known limits (not bugs to report)

- Claude Code, Cursor and agy record no quota locally, so there is no percent for them.
- agy token counts are not read. Codex shows one token total per thread.
- Refresh runs on a timer (default 1 minute), not a file watcher.
- Alert text is English only.

## Lighter option: sandbox script (not recommended now)

`./scripts/sandbox.sh` runs the app against fake data in a throwaway folder. It is not fully isolated: the Dock icon and menu bar item appear, and macOS may write one window-position settings file (`~/Library/Preferences/Meter.plist`), which the script deletes on exit. I have not run it. The VM is the safer choice.

## Cleanup after testing

- Delete the VM, or inside it: quit Meter, delete `Meter.app`, `~/Library/Application Support/Meter`, run `defaults delete <bundle id>`, and remove any Login Items or Notifications entry.
- On your real Mac, `build/vm-kit/` can be deleted any time. It is rebuilt by the script.

## Other docs

- `README.md` for build, test and privacy notes.
- `docs/design.md` for the interface rules.
- `AGENT_WORKLOG.md` for the full history and open items.
