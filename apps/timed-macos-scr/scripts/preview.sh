#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Find build products directory
BUILD_DIR=""
for dir in ".build/release" ".build/arm64-apple-macosx/release" ".build/out/Products/Release"; do
    if [ -f "$dir/TimedClockCore.o" ]; then
        BUILD_DIR="$dir"
        break
    fi
done

if [ -z "$BUILD_DIR" ]; then
    echo "==> Building TimedClockCore..."
    swift build -c release
    for dir in ".build/release" ".build/arm64-apple-macosx/release" ".build/out/Products/Release"; do
        if [ -f "$dir/TimedClockCore.o" ]; then
            BUILD_DIR="$dir"
            break
        fi
    done
fi

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

echo "==> Launching fullscreen preview (Press ESC or Q to exit)..."
.build/preview_runner
