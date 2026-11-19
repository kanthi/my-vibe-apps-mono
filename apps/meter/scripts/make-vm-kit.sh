#!/bin/sh
# Builds build/vm-kit/ to copy into a macOS virtual machine for testing the UI:
#   Meter.app            the app (ad-hoc signed, Apple silicon)
#   meter-probe          writes invented tool data into the VM's home folder
#   setup-fake-data.sh   refuses to run outside a VM, then runs meter-probe --fake-home ~
#   README.txt           what to try
# This only builds files inside this repo's build/ folder.
set -e
cd "$(dirname "$0")/.."
./scripts/bundle.sh >/dev/null
KIT=build/vm-kit
rm -rf "$KIT"
mkdir -p "$KIT"
cp -R build/Meter.app "$KIT/Meter.app"
cp "$(swift build -c release --show-bin-path)/meter-probe" "$KIT/meter-probe"
cat > "$KIT/setup-fake-data.sh" <<'S'
#!/bin/sh
# Writes invented Codex, Grok, Claude Code, Cursor and agy data into THIS user's home folder.
# Only run it inside the test VM. It refuses on a real Mac.
if [ "$(sysctl -n kern.hv_vmm_present 2>/dev/null)" != "1" ]; then
  echo "This does not look like a virtual machine, so I will not write fake data into your home folder."
  echo "Set FORCE=1 to override (not recommended)."
  [ "$FORCE" = "1" ] || exit 1
fi
cd "$(dirname "$0")"
chmod +x ./meter-probe
./meter-probe --fake-home "$HOME"
echo "Done. Now open Meter.app. Re-run this script to refresh the invented timestamps."
S
chmod +x "$KIT/setup-fake-data.sh" "$KIT/meter-probe"
cat > "$KIT/README.txt" <<'S'
Meter VM test kit
=================
1. In the VM: copy this folder in (shared folder, AirDrop, or drag and drop).
2. Terminal:  cd <this folder> && xattr -dr com.apple.quarantine . && ./setup-fake-data.sh
3. Open Meter.app (right-click > Open if macOS warns; it is ad-hoc signed, not notarized).

What to check
- Menu bar item appears: icon plus a percent; the menu shows up to 3 lines, Open Meter, Refresh Now, Settings, Quit.
- Close the window: the app keeps running in the menu bar and the Dock icon stays. Dock menu > Open Meter reopens it.
- Overview shows Codex (Session 34%, Weekly 62%), Grok (Weekly 18%), Claude Code tokens, Cursor turns, agy; "Not installed" never shows zero.
- Accounts: hide Codex, the menu percent changes; add an account with a folder; remove it.
- Settings > Alerts: turn one on; macOS asks for notification permission at that moment. Try quiet hours and snooze.
- Settings > General: "Open Meter at login" adds a Login Items entry; turn it off again.
- Quit from the menu bar; relaunch and confirm settings and hidden accounts persisted.
- Appearance: light and dark, Increase Contrast, Reduce Motion, VoiceOver reading of the gauges.
- Where Meter writes: ~/Library/Application Support/Meter and ~/Library/Preferences.
S
echo "Kit ready: $KIT"
ls "$KIT"
