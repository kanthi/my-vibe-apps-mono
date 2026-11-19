#!/bin/sh
# Builds the app, copies it into the VM, fills the VM's home with invented tool data, and launches it.
#   deploy.sh [page]     page = overview (default) | usage | accounts | settings
set -e
. "$(dirname "$0")/env.sh"
PAGE="${1:-overview}"
vm_running || { echo "VM is not running. Run scripts/tart/up.sh first."; exit 1; }
[ "$(vmssh 'sysctl -n kern.hv_vmm_present')" = "1" ] || { echo "Refusing: the target does not look like a VM"; exit 1; }
"$ROOT/scripts/make-vm-kit.sh" >/dev/null
vmssh 'pkill -f "MeterTest/Meter.app/Contents/MacOS/Meter" 2>/dev/null; rm -rf ~/MeterTest; mkdir -p ~/MeterTest/support'
vmscp "$ROOT/build/vm-kit/Meter.app" "$ROOT/build/vm-kit/meter-probe" admin@"$(vm_ip)":MeterTest/
vmssh 'chmod +x ~/MeterTest/meter-probe && ~/MeterTest/meter-probe --fake-home ~ >/dev/null'
vmssh "open -n --env METER_SUPPORT_DIR=\"\$HOME/MeterTest/support\" --env METER_START_PAGE=$PAGE --env METER_AUTOTEST_LOG=\"\$HOME/MeterTest/summary.json\" \"\$HOME/MeterTest/Meter.app\""
sleep 10
echo "Meter launched on the $PAGE page."
