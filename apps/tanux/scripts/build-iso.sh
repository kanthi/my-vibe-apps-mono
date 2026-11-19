#!/usr/bin/env bash
# Build the Tanux installer ISO (Fedora Everything base + Tanux RPMs, Kickstarts,
# Anaconda branding product.img, and white-on-black GRUB boot menu).
# Run on macOS (inside OrbStack builder automatically) or directly on Fedora.
set -euo pipefail

cd "$(dirname "$0")/.."
APP="$PWD"
MACHINE="${TANUX_BUILDER:-tanux-builder}"
FEDORA_VER="${TANUX_FEDORA_VERSION:-45}"
ARCH="${TANUX_ARCH:-aarch64}"
UPSTREAM_VER="${TANUX_UPSTREAM_BUILD:-45-1.4}"
UPSTREAM_ISO="build/upstream/Fedora-Everything-netinst-${ARCH}-${UPSTREAM_VER}.iso"
UPSTREAM_CHECKSUM="build/upstream/Fedora-Everything-${UPSTREAM_VER}-${ARCH}-CHECKSUM"
BASE_URL="https://dl.fedoraproject.org/pub/fedora/linux/releases/${FEDORA_VER}/Everything/${ARCH}/iso"
ISO_LABEL="TANUX_${FEDORA_VER}"
OUT_ISO="build/tanux-${FEDORA_VER}-${ARCH}.iso"

mkdir -p build/upstream

if [ ! -f "$UPSTREAM_CHECKSUM" ]; then
  curl -fsSL -o "$UPSTREAM_CHECKSUM" \
    "${BASE_URL}/Fedora-Everything-${UPSTREAM_VER}-${ARCH}-CHECKSUM"
fi

if [ ! -f "$UPSTREAM_ISO" ]; then
  echo "==> Downloading $UPSTREAM_ISO"
  curl -fL -C - --retry 5 --retry-delay 2 -o "$UPSTREAM_ISO" \
    "${BASE_URL}/Fedora-Everything-netinst-${ARCH}-${UPSTREAM_VER}.iso"
fi

echo "==> Verifying upstream ISO checksum"
(
  cd build/upstream
  expected=$(awk "/Fedora-Everything-netinst-${ARCH}-${UPSTREAM_VER}\.iso/ && /^[0-9a-f]{64}/ {print \$1}" "Fedora-Everything-${UPSTREAM_VER}-${ARCH}-CHECKSUM" || true)
  if [ -z "$expected" ]; then
    expected=$(grep -E "SHA256 \(Fedora-Everything-netinst-${ARCH}-${UPSTREAM_VER}\.iso\)" "Fedora-Everything-${UPSTREAM_VER}-${ARCH}-CHECKSUM" | awk '{print $NF}')
  fi
  if command -v sha256sum >/dev/null 2>&1; then
    echo "$expected  Fedora-Everything-netinst-${ARCH}-${UPSTREAM_VER}.iso" | sha256sum -c -
  else
    echo "$expected  Fedora-Everything-netinst-${ARCH}-${UPSTREAM_VER}.iso" | shasum -a 256 -c -
  fi
)

./scripts/build-rpms.sh

build_inside_fedora="
  set -euo pipefail
  cd '$APP'
  WORK=/tmp/tanux-iso-work
  rm -rf \"\$WORK\"
  mkdir -p \"\$WORK\"/{ks,grub/theme,product/usr/share/anaconda/pixmaps,product/usr/share/pixmaps,product/usr/share/plymouth/themes/spinner}

  # 1. Generate Kickstarts, grub.cfg, and white-on-black GRUB theme assets
  python3 installer/build_configs.py \
    --ks-dir \"\$WORK/ks\" \
    --grub-cfg \"\$WORK/grub/grub.cfg\" \
    --theme-dir \"\$WORK/grub/theme\" \
    --label '$ISO_LABEL'

  for ks in \"\$WORK\"/ks/*.ks; do
    ksvalidator -v 'F${FEDORA_VER}' \"\$ks\"
  done

  # 3. Build Anaconda branding product.img (cpio.gz unpacked to /updates at boot)
  cat > \"\$WORK/product/.buildstamp\" <<EOF
[Main]
Product=Tanux
Version=${FEDORA_VER}
BugURL=your distribution provided bug reporting tool
IsFinal=True
UUID=\$(date -u +%Y%m%d%H%M).${ARCH}
Variant=Everything
[Compose]
Lorax=45.4-1
EOF
  cp installer/anaconda/fedora.css \"\$WORK/product/usr/share/anaconda/pixmaps/fedora.css\"
  rsvg-convert -w 406 -h 1200 installer/anaconda/sidebar-bg.svg -o \"\$WORK/product/usr/share/anaconda/pixmaps/sidebar-bg.png\"
  rsvg-convert -w 150 -h 69 installer/anaconda/sidebar-logo.svg -o \"\$WORK/product/usr/share/anaconda/pixmaps/sidebar-logo.png\"
  rsvg-convert -w 1040 -h 132 installer/anaconda/topbar-bg.svg -o \"\$WORK/product/usr/share/anaconda/pixmaps/topbar-bg.png\"
  rsvg-convert -w 119 -h 36 installer/anaconda/anaconda_header.svg -o \"\$WORK/product/usr/share/anaconda/pixmaps/anaconda_header.png\"
  rsvg-convert -w 160 -h 160 assets/tanux-mark.svg -o \"\$WORK/product/usr/share/pixmaps/system-logo-white.png\"
  cp \"\$WORK/product/usr/share/pixmaps/system-logo-white.png\" \"\$WORK/product/usr/share/plymouth/themes/spinner/watermark.png\"
  (cd \"\$WORK/product\" && find . | sort | cpio -c -o --quiet | pigz -9c > \"\$WORK/product.img\")

  # 4. Update EFI boot image (images/efiboot.img) with Tanux grub.cfg + theme
  xorriso -osirrox on -indev '$UPSTREAM_ISO' -extract /images/efiboot.img \"\$WORK/efiboot.img\" 2>/dev/null
  chmod u+w \"\$WORK/efiboot.img\"
  mmd -D s -i \"\$WORK/efiboot.img\" ::/EFI/BOOT/theme 2>/dev/null || true
  mcopy -o -i \"\$WORK/efiboot.img\" \"\$WORK/grub/grub.cfg\" ::/EFI/BOOT/grub.cfg
  mcopy -o -i \"\$WORK/efiboot.img\" \"\$WORK/grub/theme/\"* ::/EFI/BOOT/theme/

  # 5. Repack ISO cleanly with xorriso, adding ks/, tanux-repo/, product.img, grub.cfg/theme
  rm -f '$OUT_ISO'
  xorriso -indev '$UPSTREAM_ISO' -outdev '$OUT_ISO' \
    -overwrite on \
    -volid '$ISO_LABEL' \
    -map \"\$WORK/ks\" /ks \
    -map build/repo /tanux-repo \
    -map \"\$WORK/product.img\" /images/product.img \
    -map \"\$WORK/efiboot.img\" /images/efiboot.img \
    -map \"\$WORK/grub/grub.cfg\" /EFI/BOOT/grub.cfg \
    -map \"\$WORK/grub/theme\" /EFI/BOOT/theme \
    -boot_image any cat_path='/boot.catalog' \
    -boot_image any efi_path='/images/efiboot.img' \
    -boot_image any platform_id=0xef \
    -boot_image any emul_type=no_emulation
  implantisomd5 '$OUT_ISO'
"

if [ -f /etc/fedora-release ] && command -v xorriso >/dev/null 2>&1; then
  bash -lc "$build_inside_fedora"
else
  orb -m "$MACHINE" bash -lc "$build_inside_fedora"
fi

ls -lh "$OUT_ISO"
