#!/bin/sh
# Runs Meter against invented data in a throwaway home, so nothing on this Mac is read or changed
# beyond Meter's own temp folder. Press Ctrl-C (or quit Meter) and the folder is deleted.
#
#   CFFIXED_USER_HOME  makes Foundation treat the throwaway folder as the home folder, so the readers
#                      look there instead of ~/.codex, ~/.grok, ~/.claude, Cursor and agy.
#   METER_SANDBOX      makes Meter keep its files and settings there, and turns off notifications
#                      and launch at login.
# Not covered: the Dock icon and menu bar item appear while it runs, and AppKit may write window
# positions to ~/Library/Preferences/Meter.plist; this script deletes that file on exit.
set -e
cd "$(dirname "$0")/.."
BOX="$(mktemp -d "${TMPDIR:-/tmp}/meter-sandbox.XXXXXX")"
trap 'rm -rf "$BOX"; defaults delete Meter >/dev/null 2>&1 || true' EXIT INT TERM
swift build >/dev/null
"$(swift build --show-bin-path)/meter-probe" --fake-home "$BOX"
echo "Running Meter in a sandbox at $BOX"
CFFIXED_USER_HOME="$BOX" METER_SANDBOX="$BOX" "$(swift build --show-bin-path)/Meter"
