#!/usr/bin/env bash
# Create the OrbStack Fedora machine that builds Tanux and install the lean
# packaging/ISO tools inside it. Run on macOS. Safe to re-run.
set -euo pipefail

MACHINE="${TANUX_BUILDER:-tanux-builder}"

if ! command -v orb >/dev/null; then
  echo "OrbStack is not installed (https://orbstack.dev)." >&2
  exit 1
fi

if orb list | awk '{print $1}' | grep -qx "$MACHINE"; then
  echo "Builder '$MACHINE' already exists."
else
  orb create fedora:44 "$MACHINE"
fi

orb -m "$MACHINE" bash -lc '
  set -euo pipefail
  sudo dnf install -y --setopt=install_weak_deps=False \
    rpm-build createrepo_c xorriso mtools dosfstools squashfs-tools \
    cpio pigz librsvg2-tools pykickstart isomd5sum
  rpmbuild --version
  xorriso --version | head -n 1
  uname -m
'
