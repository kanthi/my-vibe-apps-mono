#!/usr/bin/env bash
# Check that every desktop in the installer's list produces a configuration
# that evaluates. Run on macOS. Optional arguments: desktop ids to check
# (for example: hyprland niri server).
set -euo pipefail

cd "$(dirname "$0")/.."
APP="$PWD"
MACHINE="${TANIX_BUILDER:-tanix-builder}"
# Limit how hard the build pushes the Mac: at most this many CPU cores.
# Raise it for speed (the builder has all cores available), lower it to keep
# the machine cool.
CORES="${TANIX_BUILD_CORES:-4}"
NIX="nix --extra-experimental-features 'nix-command flakes' --max-jobs 1 --cores $CORES"

orb -m "$MACHINE" bash -lc "
  set -euo pipefail
  cd '$APP'
  $NIX build path:.#installer-extensions -o /tmp/tanix-installer-extensions
  $NIX shell --inputs-from path:. nixpkgs#python3 -c python3 scripts/check_desktops.py \
    /tmp/tanix-installer-extensions/lib/calamares/modules/nixos/main.py . $*
"
