#!/usr/bin/env bash
# Validate every Tanux Kickstart and check DNF5 package resolution for all
# five editions. Run on macOS. Optional args: edition IDs (e.g. kde sway).
set -euo pipefail

cd "$(dirname "$0")/.."
APP="$PWD"
MACHINE="${TANUX_BUILDER:-tanux-builder}"

./scripts/build-rpms.sh >/dev/null

if [ -f /etc/fedora-release ] && command -v ksvalidator >/dev/null 2>&1; then
  python3 scripts/check_kickstarts.py "$@"
else
  orb -m "$MACHINE" bash -lc "cd '$APP' && python3 scripts/check_kickstarts.py $*"
fi
