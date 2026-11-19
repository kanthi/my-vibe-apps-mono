#!/usr/bin/env bash
# Build all Tanux noarch RPMs inside the OrbStack builder and generate DNF
# repository metadata in build/repo/. Run on macOS or inside the builder.
set -euo pipefail

cd "$(dirname "$0")/.."
APP="$PWD"
MACHINE="${TANUX_BUILDER:-tanux-builder}"

run_in_builder() {
  if [ -f /etc/fedora-release ] && command -v rpmbuild >/dev/null 2>&1; then
    bash -lc "$1"
  else
    orb -m "$MACHINE" bash -lc "$1"
  fi
}

run_in_builder "
  set -euo pipefail
  cd '$APP'
  TOP=/tmp/tanux-rpmbuild
  rm -rf \"\$TOP\" build/repo
  mkdir -p \"\$TOP\"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS} build/repo

  cp assets/tanux-logo.svg \"\$TOP/SOURCES/\"
  cp assets/tanux-mark.svg \"\$TOP/SOURCES/\"
  cp assets/wallpapers/tanux-dark.svg \"\$TOP/SOURCES/\"
  cp assets/wallpapers/tanux-light.svg \"\$TOP/SOURCES/\"

  for spec in rpm/*.spec; do
    echo \"==> Building \$spec\"
    rpmbuild -bb \"\$spec\" \
      --define \"_topdir \$TOP\" \
      --define 'dist .fc45' \
      --define 'fedora 45'
  done

  cp \"\$TOP\"/RPMS/noarch/*.rpm build/repo/
  createrepo_c build/repo
  ls -lh build/repo/*.rpm
"
