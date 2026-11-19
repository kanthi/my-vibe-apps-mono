#!/bin/sh
# Rebuilds assets/AppIcon.png and assets/AppIcon.icns. The finished icon is committed, so you only
# need this to change the design. Needs Python 3 (Pillow is installed into build/icon-venv) and the
# macOS tools sips and iconutil.
set -e
cd "$(dirname "$0")/.."
[ -x build/icon-venv/bin/python ] || { python3 -m venv build/icon-venv && build/icon-venv/bin/pip install --quiet pillow; }
mkdir -p assets build/AppIcon.iconset
build/icon-venv/bin/python scripts/make-icon.py assets/AppIcon.png
for s in 16 32 128 256 512; do
  sips -z $s $s assets/AppIcon.png --out build/AppIcon.iconset/icon_${s}x${s}.png >/dev/null
  sips -z $((s*2)) $((s*2)) assets/AppIcon.png --out build/AppIcon.iconset/icon_${s}x${s}@2x.png >/dev/null
done
iconutil -c icns build/AppIcon.iconset -o assets/AppIcon.icns
echo "Wrote assets/AppIcon.png and assets/AppIcon.icns"
