#!/bin/sh
# Builds Meter and wraps it as build/Meter.app (Dock app that also lives in the menu bar, ad-hoc signed).
# This only builds; it does not launch or install anything.
set -e
cd "$(dirname "$0")/.."
swift build -c release
APP=build/Meter.app
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
cp "$(swift build -c release --show-bin-path)/Meter" "$APP/Contents/MacOS/Meter"
cat > "$APP/Contents/Info.plist" <<P
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>com.king.meter</string>
<key>CFBundleName</key><string>Meter</string>
<key>CFBundleExecutable</key><string>Meter</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.001</string>
<key>LSMinimumSystemVersion</key><string>14.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>NSHumanReadableCopyright</key><string>Meter reads usage records on this Mac only.</string>
</dict></plist>
P
codesign --force --sign - "$APP"
echo "Built $APP"
