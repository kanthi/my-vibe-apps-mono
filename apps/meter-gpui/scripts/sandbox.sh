#!/bin/sh
# Runs Meter against invented data with every file it writes kept in a throwaway folder.
# HOME points the readers at the fake tool folders; METER_SUPPORT_DIR keeps Meter's own files there.
# The bare binary has no bundle id, so notifications and the login item are no-ops. Not covered:
# the Dock icon and menu bar item appear while it runs. Press Ctrl-C or quit Meter to clean up.
set -e
cd "$(dirname "$0")/.."
cargo build -q -p meter-core -p meter-app
BOX="$(mktemp -d "${TMPDIR:-/tmp}/meter-gpui-sandbox.XXXXXX")"
trap 'rm -rf "$BOX"' EXIT INT TERM
target/debug/meter-probe --fake-home "$BOX/home"
echo "Running Meter in a sandbox at $BOX"
HOME="$BOX/home" METER_SUPPORT_DIR="$BOX/support" target/debug/meter
