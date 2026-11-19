# Meter (GPUI edition) agent worklog

## 2026-10-28: first complete version

**Request:** "create a complete another version of this using gpui + rust + nsh". "nsh" was not defined; I assumed the native AppKit "NS" layer (NSStatusItem, NSMenu, plus UserNotifications and ServiceManagement) through `objc2`, because GPUI has no menu-bar-item API. If "nsh" meant something else, say so. This is a new app per the repo rules: `apps/meter-gpui/`, standalone, its own README, .gitignore, lockfile (`Cargo.lock`), .env.example and scripts. The Swift edition in `apps/meter/` is untouched.

**Standing rule (user, carried over from the Swift edition):** do not let testing affect the host macOS until the user says the app is final. Honored: nothing was launched, installed or registered; tests use temp folders and injected paths; the one real-data check was the read-only `meter-probe` (temp copies of databases, counts only). No Meter files exist in `~/Library/Application Support` or `~/Library/Preferences` afterwards (checked).

**Done**
- `crates/meter-core` (Rust port of the Swift core): `model`, `readers/{codex,grok,claude_code,cursor,agy}`, `snapshot` (readers open a temp copy of db + -wal + -shm via rusqlite, never the live file), `rollup` (+ JSON store), `accounts`, `insights` (pace, notes, out-of-pattern, idle quota, menu summary), `alerts` (engine with baseline + dedupe + reset-cycle tolerance, policy with quiet hours and snooze), `dashboard` (concurrent `refresh` with keep-last-good, `Dashboard::build`, stale tag, ended windows, token/turn fallback lines), `format`, `fakehome`, `bin/meter-probe`.
- `crates/meter-app`: GPUI 0.2.2 (`runtime_shaders`, so no Metal toolchain is needed to build). `state.rs` (AppState entity, background refresh, rollup merge, alerts), `ui.rs` (RootView: sidebar + Overview, Usage, Accounts, Settings), `widgets.rs`, `theme.rs` (light/dark from window appearance), `settings.rs` (JSON in `MeterGPUI`), `statusbar.rs` (NSStatusItem + native NSMenu via objc2, clicks over a futures channel), `system.rs` (UNUserNotificationCenter and SMAppService, both no-ops without a bundle id and only called when the user flips the setting), `main.rs` (actions, ⌘Q ⌘W ⌘R ⌘,, app menu, Dock menu, on_reopen, keep running after the last window closes, refresh timer).
- Scripts: `bundle.sh` (build/Meter.app, ad-hoc signed, bundle id `com.king.meter-gpui`), `make-vm-kit.sh` (+ `setup-fake-data.sh` that refuses outside a VM; verified it refuses on the host), `sandbox.sh` (HOME + METER_SUPPORT_DIR to a throwaway folder; not run).
- Docs: README, `docs/vm-test-guide.md`. Root README catalog row added.

**Verification**
- `cargo test --workspace`: 37 tests pass (core: 18 logic + 13 reader tests ported from the Swift suite with invented fixtures, including "readers leave the original DB untouched"; app: 5 unit tests + 1 headless GPUI test that lays out and paints all four pages with and without data, hidden account, quiet hours, metric/range switches). No compiler warnings from our code (a future-incompat note comes from the `block` and `proc-macro-error2` dependencies).
- `meter-probe` on the real home folder (read-only): all five readers work: Codex 7 sessions with quota windows, Grok 2 sessions with a weekly quota, Claude Code 4 sessions, Cursor 1 session (83 turns), agy 19 sessions.
- Release build and `scripts/bundle.sh` succeed; Info.plist lints; binary is arm64, 7.7 MB.

**Not verified**
- **The app has never been launched.** Menu bar item, Dock menu, app menu, window reopen, notification permission and delivery, login item, and the folder picker are untested at runtime.
- **No one has seen the UI.** GPUI has no off-screen image rendering here, so layout is checked only for "does not panic". Expect cosmetic issues (wrapping, spacing, click targets).
- Metal shaders compile at first launch (`runtime_shaders`); first-frame delay and any shader error are unobserved.
- GPUI default window behavior when the last window closes is assumed not to quit (matches the `on_window_close_quit` example, which quits explicitly); unobserved.
- Settings changes while the Settings page is open rebuild everything; no known problem, not profiled.

**Known limits and differences from the Swift edition** (also in README)
- Accounts: added by picking a folder (label = folder name); no text input widget.
- Settings is a page, not a separate window. Settings live in JSON, not UserDefaults.
- Dock menu has no status lines (GPUI dock menus are action-only).
- VoiceOver on the window is limited by GPUI; the menu bar item/menu are native and labeled.
- Refresh is a timer (default 60 s), no file watching. Grok output tokens = completion + reasoning (unverified). Codex gives one token total per thread. agy tokens unread. Claude Code, Cursor and agy have no quota percent.
- GPUI may still write window-state defaults of its own; unverified (the VM guide asks to check).

**Next steps**
1. User tests in a macOS VM (`./scripts/make-vm-kit.sh`, then `docs/vm-test-guide.md`) and reports findings.
2. Fix layout issues found. Likely candidates: card widths on narrow windows, text clipping in rows, scroll behavior, hover states.
3. Optional: a simple text input for account labels, status lines in the Dock menu, a file watcher, per-tool alert switches.

## 2026-10-28: first launch (user asked "cargo run for me")
The user explicitly asked for `cargo run`, so the standing "do not launch" rule was lifted for this one run. Ran `cargo run -p meter-app` in the background (debug binary, real home folder, read-only reads).
- Result: process stays up, no panic or error output after ~15 s. First refresh wrote `~/Library/Application Support/MeterGPUI/rollup.json` (31 rows, 17 days, all five tools). The bare binary has no bundle id, so notifications and the login item were not exercised.
- **Host footprint observed:** `~/Library/Preferences/meter.plist` appeared (GPUI/AppKit window state; Meter does not write it itself). Left in place while the app runs. The README now says so. Other plists there (`gpui-hello-world`, `gpui-nsh-test-agy`, `so.pen.herdr-gpui`) pre-date this run and are not ours; `gpui-nsh-test-agy` hints that "nsh" may be the user's own project; unresolved.
- Not observed: the UI itself (no screenshot taken, the agent cannot see the screen). Awaiting the user's description of the menu bar item and window.
- Cleanup when the user is done: quit Meter, delete `~/Library/Application Support/MeterGPUI` and `~/Library/Preferences/meter.plist` (ask first).

## 2026-10-28: cleanup after the first launch
User approved deleting this run's files. Deleted `~/Library/Application Support/MeterGPUI` (only `rollup.json`) and `~/Library/Preferences/meter.plist`. Verified both are gone. The other plists in `~/Library/Preferences` (`gpui-hello-world`, `gpui-nsh-test-agy`, `so.pen.herdr-gpui`) were not touched. The first run exited with code 0; why it exited (menu Quit or closing the window) was not confirmed by the user.

## 2026-10-28: automated testing in the user's VirtualBuddy VM
**Why:** the user wanted a better way to test than the host. Fusion 26 does not run macOS guests on Apple silicon (Broadcom KB 301109 and 315602; unconfirmed for 26.x specifically). The user then said macOS was already downloaded by VirtualBuddy, so I dropped the Tart plan.
**Found (read-only):** VirtualBuddy 2.2.1 has one VM, "Multiple Turkey" (`~/Library/Application Support/VirtualBuddy/Multiple Turkey.vbvm`): macOS 27.0.1 installed, 4 CPUs, 6 GB, 4480x2520 display, guest additions on, **no shared folders**, a saved session. `vctool` only manages the software catalog; there is no CLI to start the VM or run commands in it. The VM's disk, config and saved session were not touched.
**Approach:** file-based RPC over a VirtualBuddy shared folder. Host stages `build/vm-share/` (kit + guest scripts + jobs + results). A LaunchAgent in the VM runs `scripts/vm/runner.sh`, which runs `run-suite.sh` per job: copies the app out of the share, fills the VM home with invented data (`meter-probe --fake-home`), launches Meter in each page x light/dark, the menu-open state, and the close-window state, takes `screencapture` screenshots, writes JSON summaries, CPU/RSS, a footprint listing and `result.json`. Host: `scripts/vm-stage.sh`, `scripts/vm-run.sh`. All guest scripts refuse outside a VM (`kern.hv_vmm_present`); verified on the host that they refuse.
**App test hooks added (env vars only, inert otherwise):** `METER_START_PAGE`, `METER_AUTOTEST_OPEN_MENU_MS` (calls `performClick` on the status item), `METER_AUTOTEST_CLOSE_WINDOW_MS`, `METER_AUTOTEST_LOG` (JSON summary after each refresh), plus the existing `METER_SUPPORT_DIR`. `cargo test --workspace` still 37 passing, no warnings from our code.
**Pending on the user (one-time):** add `build/vm-share` as a shared folder in VirtualBuddy (VM off), start the VM, run `install-runner.sh` inside it, click Allow for screen recording. Steps in `docs/vm-automation.md`. Not yet run end to end, so the runner, screenshots and permission behavior are unverified.
**Tart leftovers (mine, from the abandoned plan):** `~/.local/opt/tart/tart.app` (official 2.41.1, checksum-verified) and about 306 MB of a partial image download in `~/.tart`. Not removed; ask the user.

## 2026-10-28: automated testing in a Tart VM (replaces the shared-folder idea)
**Correction:** the user meant the macOS *installer* VirtualBuddy downloaded (`UniversalMac_27.0.1_26A434_Restore.ipsw`, 25 GB), not their installed VM. The VirtualBuddy VM ("Multiple Turkey") and the ipsw were never modified (the ipsw was only read). Fusion 26 does not run macOS guests on Apple silicon (Broadcom KB 301109/315602).
**Done**
- Installed Tart 2.41.1 (official release, sha256 verified, Cirrus Labs notarized) to `~/.local/opt/tart/` (the Homebrew tap formula is broken on this Homebrew). Created VM `meter-test` from the ipsw with `tart create --from-ipsw` (asif disk, 50 GB cap, 35 GB used), set to 2 CPUs / 4 GB / 1280x800 per the user's "minimal resources" request.
- First boot used macOS 27 provisioning options (admin/admin, auto-login, Remote Login): no Setup Assistant. A throwaway ssh key and a `vncdotool` venv live in `build/vm-tools/` (git-ignored). Screenshots and clicks go over the VM's built-in VNC (`--vnc-experimental`), so the guest needs no screen-recording permission.
- Scripts: `scripts/tart/{env,up,deploy,suite,down}.sh`. Removed the obsolete shared-folder runner (`scripts/vm/`, `vm-run.sh`, `vm-stage.sh`, `build/vm-share`). New docs: `docs/vm-automation.md`; README and vm-test-guide updated.
- App test hooks (env vars, inert otherwise): `METER_START_PAGE`, `METER_AUTOTEST_OPEN_MENU_MS`, `METER_AUTOTEST_CLOSE_WINDOW_MS`, `METER_AUTOTEST_LOG`.
**Verified in the VM (macOS 27.0.1, screenshots inspected)**
- Menu bar item: gauge icon + "62%"; native menu with 3 limit lines, ⚠︎ pace warning, Open Meter…, Refresh Now ⌘R, Settings… ⌘,, Quit Meter ⌘Q.
- All four pages render; Overview cards with pace-colored bars and words; Usage chart; Settings toggles.
- Closing the window keeps the app running (menu bar item stays); the status menu's Open Meter… reopens it. Answers the earlier open question about whether closing quits: it does not.
- Dark mode: the app follows the system's real appearance change live (no relaunch). Setting `AppleInterfaceStyle` alone does not change the VM's appearance; the suite uses System Events (one TCC "Allow" click in the VM was needed once).
- Idle: 0.2% CPU, ~60 MB RSS. The bundled app writes `~/Library/Preferences/com.king.meter-gpui.plist` (GPUI window state).
**Bugs found and fixed**
1. Accounts page overflowed the window (flex item could not shrink below its longest text): `min_w_0` on the page container and cards.
2. `fakehome` failed when re-run in the same folder, and could have overwritten real tool files (for example the real Grok log) if pointed at a real home. It now recreates its files and refuses a real home outside a VM (`is_unsafe_target`, `running_in_vm`); 2 new tests. 39 tests pass. The same hazard exists in the Swift edition's `meter-probe --fake-home` (only the kit script guards it); not changed.
3. suite.sh: `set -e` plus a grep that exits 1 when it filters everything ended the run silently; fixed with `|| true`.
**Observed, not fixed**
- No app icon (generic Dock icon).
- Dock overlaps the lower part of the 1000x680 window on a 1280x800 display (window is not clamped to the visible area).
- Fake data has only two days of history, so the 14/30-day charts look sparse.
**Not tested:** notification permission and delivery, login item, folder picker (need clicks through macOS prompts), VoiceOver.
**State now:** VM stopped (`down.sh`). `~/.tart` is 33 GB; `~/.tart/cache` is empty. Cleanup commands are in `docs/vm-automation.md`.

## 2026-10-28: app icon and dynamic window size (user: "do not test this yet")
**Done**
- **App icon.** `scripts/make-icon.py` (Pillow) draws a 1024 px gauge on a rounded blue square: used part of the dial solid white, unused part translucent, needle at 62%. `scripts/make-icon.sh` builds the iconset with `sips` and `assets/AppIcon.icns` with `iconutil` (Pillow goes into `build/icon-venv`). The finished `assets/AppIcon.icns` and `AppIcon.png` are committed, so builds do not need Python. `scripts/bundle.sh` copies the icns to `Contents/Resources/` and sets `CFBundleIconFile`. I viewed the PNG (first draft had a flaw: the translucent track was written without blending, so the whole dial looked solid; fixed with separate blended layers and a smooth highlight).
- **Window size from the screen.** New `crates/meter-app/src/window.rs`: `initial_size(display_w, display_h)` is about 78% of the display, clamped to 780..1200 wide and 480..820 tall, never taller than the display minus 170 pt (menu bar + Dock) and never more than 95% of a small display; `min_size` never exceeds the starting size. `main.rs` reads `cx.primary_display()` each time the window opens and falls back to 1000x680. 4 unit tests (1280x800 gives 998x624, 1440x900 gives 1123x702, large displays cap at 1200x820, tiny displays never overflow).
- **Responsive cards.** Overview tool cards are `flex_1` with min 300 and max 560 pt instead of a fixed 360, so they share the row and reflow when the window is resized.
**Verified:** `cargo test --workspace` passes, 43 tests (10 app incl. the headless render of all four pages, 18 + 15 core). `sh -n` and a plist lint on the new `bundle.sh` text.
**NOT verified (by request, not tested):** the icon in the real Dock, Finder and the `.app` (bundle.sh was not run, so the icns copy and Info.plist were checked by lint only); the window size on a real display; card reflow while dragging the window edge. No launch, no VM run. Next: run `./scripts/tart/up.sh && ./scripts/tart/suite.sh` and check the Dock icon and window fit in the screenshots. The suite's click coordinates assume the sidebar position, which has not changed.
**Housekeeping:** `cargo clean` earlier freed 7 GB; this build regrew `target/` (see `du -sh target`).

## 2026-11-14: .gitignore hardened

Added ignore rules so the app stays clean after extraction from the incubator (repo-wide ignore pass). Added `.env` secrets rules, `.tart/` and `*.ipsw`. No code changes.
