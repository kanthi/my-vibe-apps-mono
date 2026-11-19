#!/bin/sh
# Stops the app and the VM. The VM's disk stays; `tart delete meter-test` removes it.
. "$(dirname "$0")/env.sh"
vmssh 'pkill -f "MeterTest/Meter.app/Contents/MacOS/Meter"' 2>/dev/null
"$TART" stop "$VM" 2>&1 | tail -1
echo "VM stopped."
