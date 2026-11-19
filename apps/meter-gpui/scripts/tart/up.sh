#!/bin/sh
# Creates the test VM on first use (from a macOS restore image you already have), then boots it
# headless with minimal resources and waits until SSH works. Safe to re-run.
set -e
. "$(dirname "$0")/env.sh"

if ! "$TART" list 2>/dev/null | awk -v n="$VM" '$2==n {f=1} END {exit !f}'; then
  [ -f "$IPSW" ] || { echo "No restore image at: $IPSW (set METER_IPSW)"; exit 1; }
  echo "Creating VM $VM from the restore image (10-20 minutes, once)..."
  "$TART" create "$VM" --from-ipsw "$IPSW" --disk-size 50 --disk-format asif
  "$TART" set "$VM" --cpu 2 --memory 4096 --display 1280x800
fi

if [ ! -x "$VT/venv/bin/vncdo" ]; then
  python3 -m venv "$VT/venv" && "$VT/venv/bin/pip" install --quiet vncdotool
fi

if ! vm_running; then
  echo "Booting $VM (2 CPUs, 4 GB, no window)..."
  nohup "$TART" run "$VM" --no-graphics --no-audio --no-clipboard --no-usb-accessories --vnc-experimental \
    --provisioning-opts="fullName=Meter Test,username=admin,password=admin,logsInAutomatically=true,enablesRemoteLogin=true" \
    > "$VT/tart-run.log" 2>&1 &
fi
"$TART" ip "$VM" --wait 240 >/dev/null

# One-time: install a throwaway SSH key using the first-boot password.
[ -f "$VT/id_vm" ] || ssh-keygen -q -t ed25519 -N "" -C meter-vm-test -f "$VT/id_vm"
printf '#!/bin/sh\necho admin\n' > "$VT/askpass.sh"; chmod +x "$VT/askpass.sh"
i=0
until vmssh true 2>/dev/null; do
  SSH_ASKPASS="$VT/askpass.sh" SSH_ASKPASS_REQUIRE=force DISPLAY=:0 ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile="$VT/known_hosts" \
    -o PreferredAuthentications=password,keyboard-interactive -o PubkeyAuthentication=no -o ConnectTimeout=15 -o LogLevel=ERROR \
    admin@"$(vm_ip)" 'mkdir -p ~/.ssh && chmod 700 ~/.ssh && cat >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys' < "$VT/id_vm.pub" 2>/dev/null || true
  i=$((i+1)); [ $i -gt 30 ] && { echo "SSH never came up"; exit 1; }; sleep 5
done
echo "VM ready: $(vm_ip). Guest: $(vmssh 'sw_vers -productVersion') ($(vmssh 'sysctl -n hw.ncpu') CPUs)"
