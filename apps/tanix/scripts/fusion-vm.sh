#!/usr/bin/env bash
# Create and start a VMware Fusion test VM for the newest ISO in $TANIX_OUT
# (default ~/Workspace/FusionVMs/tanix). Run on macOS (Apple Silicon). The VM
# lives in $TANIX_OUT/vm, outside the app folder (see build-iso.sh for why).
#
#   ./scripts/fusion-vm.sh          # new empty disk, boot the newest ISO
#                                   # (or TANIX_ISO=/path/to.iso)
#   ./scripts/fusion-vm.sh disk     # keep the disk, boot the installed system
#   ./scripts/fusion-vm.sh stop     # power the VM off
#
# Three settings here were found by testing and matter:
#   - The ISO is attached as a disk (like a USB stick), not as a CD. As a CD,
#     the live system hit read errors partway through an install.
#   - Network is "bridged". With Fusion's NAT the installer reported no
#     internet on this Mac.
#   - The network card is vmxnet3. With e1000e the guest had no network.
set -euo pipefail

cd "$(dirname "$0")/.."
FUSION="/Applications/VMware Fusion.app/Contents/Library"
# Set TANIX_VM_SUFFIX to run a second VM next to the first (it gets its own
# folder), and TANIX_VM_VNC_PORT to give it its own remote display port.
OUT="${TANIX_OUT:-$HOME/Workspace/FusionVMs/tanix}"
VM_DIR="$OUT/vm${TANIX_VM_SUFFIX:+-$TANIX_VM_SUFFIX}"
VNC_PORT="${TANIX_VM_VNC_PORT:-5901}"
VMX="$VM_DIR/tanix-test.vmx"
MODE="${1:-iso}"

if [ "$MODE" = stop ]; then
  "$FUSION/vmrun" stop "$VMX" hard
  exit 0
fi

if "$FUSION/vmrun" list | grep -qF "$VMX"; then
  "$FUSION/vmrun" stop "$VMX" hard
fi

if [ "$MODE" = iso ]; then
  ISO="${TANIX_ISO:-$(ls -t "$OUT"/tanix-*.iso | head -1)}"
  rm -rf "$VM_DIR"
  mkdir -p "$VM_DIR"

  "$FUSION/vmware-vdiskmanager" -c -s 30GB -a lsilogic -t 0 "$VM_DIR/tanix-test.vmdk" >/dev/null

  # A descriptor that presents the ISO file as a raw disk.
  SECTORS=$(( $(stat -f %z "$ISO") / 512 ))
  cat > "$VM_DIR/tanix-iso.vmdk" <<DESC
# Disk DescriptorFile
version=1
CID=fffffffe
parentCID=ffffffff
createType="monolithicFlat"

RW $SECTORS FLAT "$ISO" 0

ddb.adapterType = "lsilogic"
ddb.virtualHWVersion = "21"
DESC

  cat > "$VMX" <<CONF
.encoding = "UTF-8"
config.version = "8"
virtualHW.version = "22"
displayName = "Tanix test${TANIX_VM_SUFFIX:+ $TANIX_VM_SUFFIX}"
guestOS = "arm-other6xlinux-64"
firmware = "efi"
numvcpus = "4"
memsize = "4096"
nvram = "tanix-test.nvram"
svga.present = "TRUE"
mks.enable3d = "TRUE"
pciBridge0.present = "TRUE"
pciBridge4.present = "TRUE"
pciBridge4.virtualDev = "pcieRootPort"
pciBridge4.functions = "8"
pciBridge5.present = "TRUE"
pciBridge5.virtualDev = "pcieRootPort"
pciBridge5.functions = "8"
pciBridge6.present = "TRUE"
pciBridge6.virtualDev = "pcieRootPort"
pciBridge6.functions = "8"
pciBridge7.present = "TRUE"
pciBridge7.virtualDev = "pcieRootPort"
pciBridge7.functions = "8"
vmci0.present = "TRUE"
hpet0.present = "TRUE"
nvme0.present = "TRUE"
nvme0:0.present = "TRUE"
nvme0:0.fileName = "tanix-test.vmdk"
sata0.present = "TRUE"
sata0:1.present = "TRUE"
sata0:1.deviceType = "disk"
sata0:1.fileName = "tanix-iso.vmdk"
sata0:1.mode = "independent-nonpersistent"
ethernet0.present = "TRUE"
ethernet0.connectionType = "bridged"
ethernet0.virtualDev = "vmxnet3"
ethernet0.addressType = "generated"
usb.present = "TRUE"
ehci.present = "TRUE"
usb_xhci.present = "TRUE"
# Lets scripts/vnc_input.py click and type in the VM (localhost only use).
RemoteDisplay.vnc.enabled = "TRUE"
RemoteDisplay.vnc.port = "$VNC_PORT"
CONF
elif [ "$MODE" = disk ]; then
  # Detach the ISO so the firmware boots the installed system.
  sed -i '' 's/sata0:1.present = "TRUE"/sata0:1.present = "FALSE"/' "$VMX"
else
  echo "usage: $0 [iso|disk|stop]" >&2
  exit 1
fi

"$FUSION/vmrun" start "$VMX"
echo "Started. Screenshot: \"$FUSION/vmcli\" \"$VMX\" MKS captureScreenshot out.png"
