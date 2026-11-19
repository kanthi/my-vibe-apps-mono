#!/bin/sh
# Builds Meter and wraps it as build/Meter.app (a Dock app that also lives in the menu bar,
# ad-hoc signed). This only builds; it does not launch or install anything.
set -e
cd "$(dirname "$0")/.."
cargo build --release -p meter-app -p meter-core
APP=build/Meter.app
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp assets/AppIcon.icns "$APP/Contents/Resources/AppIcon.icns"
cp target/release/meter "$APP/Contents/MacOS/Meter"
cat > "$APP/Contents/Info.plist" <<P
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>com.king.meter-gpui</string>
<key>CFBundleName</key><string>Meter</string>
<key>CFBundleDisplayName</key><string>Meter</string>
<key>CFBundleExecutable</key><string>Meter</string>
<key>CFBundleIconFile</key><string>AppIcon</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>LSMinimumSystemVersion</key><string>14.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>NSHumanReadableCopyright</key><string>Meter reads usage records on this Mac only.</string>
</dict></plist>
P
codesign --force --sign - "$APP"
echo "Built $APP"
