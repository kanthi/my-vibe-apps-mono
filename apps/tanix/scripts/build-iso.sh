#!/usr/bin/env bash
# Build a Tanix ISO inside the OrbStack builder and copy it to $TANIX_OUT
# (default ~/Workspace/FusionVMs/tanix). Run on macOS from anywhere.
#
#   ./scripts/build-iso.sh          # desktop live/installer ISO
#   ./scripts/build-iso.sh server   # Tanix Server ISO (text installer)
#
# The ISO is kept outside the app folder on purpose: `path:.` copies the
# whole app folder into the Nix store on every build, so a 3 GB ISO (or a
# test VM) inside it would be copied again each time.
set -euo pipefail

cd "$(dirname "$0")/.."
APP="$PWD"
MACHINE="${TANIX_BUILDER:-tanix-builder}"
# Limit how hard the build pushes the Mac: at most this many CPU cores.
# Raise it for speed (the builder has all cores available), lower it to keep
# the machine cool.
CORES="${TANIX_BUILD_CORES:-4}"
NIX="nix --extra-experimental-features 'nix-command flakes' --max-jobs 1 --cores $CORES"

case "${1:-desktop}" in
  desktop) TARGET=iso ;;
  server) TARGET=iso-server ;;
  *) echo "usage: $0 [desktop|server]" >&2; exit 1 ;;
esac

OUT="${TANIX_OUT:-$HOME/Workspace/FusionVMs/tanix}"
mkdir -p "$OUT"

# `path:.` makes Nix read the folder as it is on disk, so untracked or
# uncommitted files are included.
# The result link points into the builder's own /nix/store, which macOS
# cannot see, so the ISO is copied out next to it.
orb -m "$MACHINE" bash -lc "
  set -euo pipefail
  cd '$APP'
  result=\$($NIX build path:.#$TARGET --no-link --print-out-paths -L)
  cp -fL \$result/iso/*.iso '$OUT/'
  chmod u+w '$OUT'/*.iso
"

ls -lht "$OUT"/*.iso
