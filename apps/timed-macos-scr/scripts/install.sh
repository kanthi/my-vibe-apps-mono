#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

SAVER_NAME="TimedClock.saver"
DEST_DIR="$HOME/Library/Screen Savers"

# If not built yet, build it
if [ ! -d "dist/$SAVER_NAME" ]; then
    echo "==> Saver bundle not found in dist/. Building now..."
    ./scripts/build.sh
fi

echo "==> Installing $SAVER_NAME to $DEST_DIR..."
mkdir -p "$DEST_DIR"

# Remove existing installation if present
rm -rf "$DEST_DIR/$SAVER_NAME"

# Copy new bundle
cp -R "dist/$SAVER_NAME" "$DEST_DIR/$SAVER_NAME"

# Remove quarantine flags if applied by macOS Gatekeeper
xattr -r -d com.apple.quarantine "$DEST_DIR/$SAVER_NAME" 2>/dev/null || true

echo "==> Installation complete!"
echo "    Installed to: $DEST_DIR/$SAVER_NAME"
echo ""
echo "==> Opening System Settings > Screen Saver..."
open "x-apple.systempreferences:com.apple.ScreenSaver-Settings.extension" 2>/dev/null || \
open "x-apple.systempreferences:com.apple.preference.desktopscreeneffect" 2>/dev/null || \
open "/System/Library/PreferencePanes/DesktopScreenEffectsPref.prefPane" 2>/dev/null || true

echo "Select 'TimedClock' in the list to activate."
