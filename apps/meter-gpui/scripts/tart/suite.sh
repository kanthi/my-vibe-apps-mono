#!/bin/sh
# Runs the app in the VM and checks it: every page in light and dark, the menu bar menu, closing the
# window (the app must keep running) and reopening it. Screenshots go to build/vm-tools/shots/<time>/.
# Clicks use VNC coordinates for the 1280x800 display (framebuffer pixels = 2x points).
set -e
. "$(dirname "$0")/env.sh"
OUT="$VT/shots/$(date +%Y%m%d-%H%M%S)"; mkdir -p "$OUT"
S() { vnc capture "$OUT/$1.png"; echo "  shot: $1"; }
alive() { vmssh 'pgrep -f "MeterTest/Meter.app/Contents/MacOS/Meter" >/dev/null && echo yes || echo no'; }

set_dark() { vmssh "osascript -e 'tell application \"System Events\" to tell appearance preferences to set dark mode to $1'" >/dev/null 2>&1 || echo "  (could not switch appearance: allow the System Events prompt in the VM once)"; sleep 3; }

"$(dirname "$0")/deploy.sh" overview
RESULTS="$OUT/result.txt"; : > "$RESULTS"
for mode in light dark; do
  [ "$mode" = dark ] && set_dark true || set_dark false
  vnc move 421 250 click 1 pause 1.5 >/dev/null; S "$mode-overview"
  vnc move 421 333 click 1 pause 1.5 >/dev/null; S "$mode-usage"
  vnc move 421 418 click 1 pause 1.5 >/dev/null; S "$mode-accounts"
  vnc move 421 503 click 1 pause 1.5 >/dev/null; S "$mode-settings"
done
set_dark false

# the menu bar item opens a native menu
vnc move 2022 28 click 1 pause 1.5 >/dev/null; S menu-open; vnc key esc pause 1 >/dev/null

# closing the window must not quit the app; the status menu reopens it
vnc move 311 152 click 1 pause 3 >/dev/null; S window-closed
echo "app alive after closing the window: $(alive)" | tee -a "$RESULTS"
vnc move 2022 28 click 1 pause 1.5 move 1638 311 click 1 pause 2.5 >/dev/null; S window-reopened
echo "app alive after reopening: $(alive)" | tee -a "$RESULTS"

# what the app computed, and how much it costs at idle
vmscp admin@"$(vm_ip)":MeterTest/summary.json "$OUT/summary.json" 2>/dev/null || true
echo "idle cpu%, rss KB: $(vmssh 'ps -o %cpu=,rss= -p $(pgrep -f "MeterTest/Meter.app/Contents/MacOS/Meter" | head -1)')" | tee -a "$RESULTS"
echo "preferences the app added: $(vmssh 'ls ~/Library/Preferences | grep -i "meter" || echo none')" | tee -a "$RESULTS"
echo "Results in $OUT"
