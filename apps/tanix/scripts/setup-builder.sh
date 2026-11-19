#!/usr/bin/env bash
# Create the OrbStack NixOS machine that builds Tanix. Run on macOS. Safe to re-run.
set -euo pipefail

MACHINE="${TANIX_BUILDER:-tanix-builder}"

if ! command -v orb >/dev/null; then
  echo "OrbStack is not installed (https://orbstack.dev)." >&2
  exit 1
fi

if orb list | awk '{print $1}' | grep -qx "$MACHINE"; then
  echo "Builder '$MACHINE' already exists."
else
  orb create nixos "$MACHINE"
fi

orb -m "$MACHINE" bash -lc 'nix --version && uname -m'
