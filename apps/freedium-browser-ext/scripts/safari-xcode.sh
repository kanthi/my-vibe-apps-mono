#!/bin/sh
# Wrap safari/ in a Safari app. Requires full Xcode, not just the Command Line Tools.
set -eu
root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)

if ! xcrun --find safari-web-extension-converter >/dev/null 2>&1; then
  echo "safari-web-extension-converter was not found." >&2
  echo "Install Xcode, then run this script again from apps/freedium-browser-ext." >&2
  exit 1
fi

xcrun safari-web-extension-converter "$root/safari" \
  --project-location "$root/safari-xcode" \
  --app-name Freedium \
  --bundle-identifier com.kanthi.freedium-browser-ext \
  --swift \
  --macos-only \
  --force \
  --no-open \
  --no-prompt
