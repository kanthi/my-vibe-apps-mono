# Sourced by the other tart scripts. Everything the scripts create lives in build/vm-tools/ (git-ignored).
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VT="$ROOT/build/vm-tools"
TART="${TART:-$HOME/.local/opt/tart/tart.app/Contents/MacOS/tart}"
VM="${METER_VM:-meter-test}"
# macOS restore image used to create the VM the first time (VirtualBuddy's download works).
IPSW="${METER_IPSW:-$HOME/Library/Application Support/VirtualBuddy/_Downloads/UniversalMac_27.0.1_26A434_Restore.ipsw}"
mkdir -p "$VT"

vm_ip() { "$TART" ip "$VM" 2>/dev/null; }
vm_running() { "$TART" list 2>/dev/null | awk -v n="$VM" '$2==n && $NF=="running" {f=1} END {exit !f}'; }

# ssh/scp into the VM with the throwaway key (never a password after the first install)
vmssh() { ssh -i "$VT/id_vm" -o IdentitiesOnly=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile="$VT/known_hosts" -o ConnectTimeout=15 -o LogLevel=ERROR admin@"$(vm_ip)" "$@"; }
vmscp() { scp -q -r -i "$VT/id_vm" -o IdentitiesOnly=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile="$VT/known_hosts" "$@"; }

# Screen over the VM's built-in VNC server. The port and password come from the tart run log and are never printed.
vnc() {
  URL="$(grep -o 'vnc://[^ ]*' "$VT/tart-run.log" | tail -1)"
  PASS="$(printf '%s' "$URL" | sed -E 's#vnc://:([^@]*)@.*#\1#')"
  HP="$(printf '%s' "$URL" | sed -E 's#.*@([^ ]*)#\1#' | sed 's#:#::#')"
  "$VT/venv/bin/vncdo" -s "$HP" -p "$PASS" "$@" 2>&1 | grep -v -i "CryptographyDeprecationWarning\|encryptor =" || true   # grep exits 1 when it filters everything
}
