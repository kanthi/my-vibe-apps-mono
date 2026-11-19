# Automated testing in a macOS VM (Tart)

Meter is tested on a throwaway macOS VM that scripts can start, drive, screenshot and stop. Nothing runs on your real Mac except Tart itself and the VM disk. Verified end to end on 2026-10-28 (macOS 27.0.1 guest, 2 CPUs, 4 GB).

## How it works

- **Tart** is a small command-line wrapper around Apple's Virtualization framework (the same technology VirtualBuddy uses). A VM is a folder in `~/.tart/vms/<name>/`. There is no background service and no kernel extension. A stopped VM costs only disk space; `tart delete meter-test` removes it.
- **First boot without Setup Assistant:** macOS 27 supports provisioning options (`username`, `password`, automatic login, Remote Login), so the VM comes up logged in with SSH on and nothing to click.
- **Control:** SSH (a throwaway key in `build/vm-tools/`) copies the app in and launches it. The VM's built-in VNC server gives screenshots and mouse and keyboard input from the host (`vncdotool`), so no screen-recording or accessibility permission is needed inside the VM.

## Requirements

- Apple silicon Mac on macOS 27 (first-boot provisioning needs macOS 27 on host and guest), about 40 GB free.
- Tart 2.33 or newer. The Homebrew tap's formula was broken here, so the official release (checksum-verified, notarized) was installed to `~/.local/opt/tart/`. Override with `TART=/path/to/tart`.
- A macOS restore image (`.ipsw`). VirtualBuddy's download is used by default (`~/Library/Application Support/VirtualBuddy/_Downloads/…27.0.1…ipsw`); override with `METER_IPSW`. The file is only read.
- Python 3 (a virtual environment with `vncdotool` is created in `build/vm-tools/venv`).

## Scripts (`scripts/tart/`)

| Script | What it does |
|---|---|
| `up.sh` | First time: creates the VM from the restore image (10 to 20 minutes) and sets 2 CPUs, 4 GB, 1280x800. Boots it headless with no audio or clipboard, waits for SSH, installs the throwaway key. Safe to re-run. |
| `deploy.sh [page]` | Builds the kit, copies the app into the VM, fills the VM's home with invented tool data (the generator refuses to run on a real home outside a VM), launches Meter on `overview`, `usage`, `accounts` or `settings`. |
| `suite.sh` | Runs the checks: every page in light and dark mode (it really switches the VM's appearance), the menu bar menu, closing the window (the app must keep running), reopening it from the menu bar. Screenshots and `result.txt` go to `build/vm-tools/shots/<time>/`. |
| `down.sh` | Quits the app and stops the VM. |

Typical loop: `./scripts/tart/up.sh && ./scripts/tart/suite.sh`, look at the screenshots, fix, run `suite.sh` again, then `./scripts/tart/down.sh`.

## What the first run found

- The Accounts page overflowed the window (page container could not shrink below its longest text). Fixed with `min_w_0`.
- Confirmed working: the menu bar item (gauge icon and percent), its native menu, closing the window keeps the app running, the menu reopens it, the window follows the system's dark mode live, idle CPU about 0.2% and about 60 MB.
- The bundled app writes `~/Library/Preferences/com.king.meter-gpui.plist` (GPUI window state).
- Not tested: notification permission and delivery, the login item, the folder picker (they need clicks through macOS prompts), and VoiceOver.
- A fake-data generator bug was found and fixed: it failed when re-run in the same folder, and it could have overwritten real tool files if pointed at a real home. It now refuses a real home outside a VM and can be re-run.

## Limits and costs

- Disk: the VM disk is capped at 50 GB and was 35 GB after first use. Memory and CPU are used only while it runs.
- Clicks use fixed pixel coordinates for the 1280x800 display (`suite.sh`); a layout change that moves the sidebar needs those updated.
- The VM is separate from your VirtualBuddy VM, which is never touched.

## Cleanup

`./scripts/tart/down.sh`, then `~/.local/opt/tart/tart.app/Contents/MacOS/tart delete meter-test` (frees about 35 GB). To remove Tart: delete `~/.local/opt/tart` and `~/.tart`.
