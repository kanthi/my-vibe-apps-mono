#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Building Timed Clock Screen Saver (TimedClock.saver)..."

# 1. Build Swift Package (TimedClockCore library)
swift build -c release

# Find build products directory
BUILD_DIR=""
for dir in ".build/release" ".build/arm64-apple-macosx/release" ".build/out/Products/Release"; do
    if [ -f "$dir/TimedClockCore.o" ]; then
        BUILD_DIR="$dir"
        break
    fi
done

mkdir -p dist

# 2. Build Native ScreenSaver Bundle: dist/TimedClock.saver
SAVER_DIR="dist/TimedClock.saver"
rm -rf "$SAVER_DIR"
mkdir -p "$SAVER_DIR/Contents/MacOS"
mkdir -p "$SAVER_DIR/Contents/Resources"

cat << 'EOF' > "$SAVER_DIR/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>TimedClock</string>
    <key>CFBundleIdentifier</key>
    <string>com.vibe.timed-macos-scr</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>TimedClock</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.007</string>
    <key>CFBundleVersion</key>
    <string>0.007</string>
    <key>NSPrincipalClass</key>
    <string>TimedClockScreenSaverView</string>
</dict>
</plist>
EOF

swiftc -O \
  Sources/TimedClockSaver/TimedClockScreenSaverView.swift \
  "$BUILD_DIR/TimedClockCore.o" \
  -I "$BUILD_DIR" \
  -framework AppKit \
  -framework ScreenSaver \
  -framework SwiftUI \
  -Xlinker -bundle \
  -o "$SAVER_DIR/Contents/MacOS/TimedClock"

echo "==> Screen saver bundle built successfully!"
echo "    Location: $(pwd)/dist/TimedClock.saver"
