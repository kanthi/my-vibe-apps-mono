#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Always build: it is incremental (a few seconds when nothing changed), and it keeps
# the preview in step with the sources and with the installed macOS SDK.
echo "==> Building TimedClockCore..."
swift build -c release

BUILD_DIR=""
for dir in ".build/release" ".build/arm64-apple-macosx/release" ".build/out/Products/Release"; do
    if [ -f "$dir/TimedClockCore.o" ]; then
        BUILD_DIR="$dir"
        break
    fi
done

mkdir -p .build

echo "==> Preparing fullscreen screensaver preview..."
swiftc -parse-as-library -O \
  scripts/preview_runner.swift \
  "$BUILD_DIR/TimedClockCore.o" \
  -I "$BUILD_DIR" \
  -framework AppKit \
  -framework ScreenSaver \
  -framework SwiftUI \
  -o .build/preview_runner

echo "==> Launching fullscreen preview (arrows or 1-8 switch face, Space toggles Bauhaus light/dark, Esc or Q quits)..."
.build/preview_runner "$@"
