#!/usr/bin/env python3
"""Click and type in the VMware Fusion test VM from the command line.

Talks to the VM's built-in remote display (VNC), which scripts/fusion-vm.sh
enables on port 5901 without a password. Coordinates are screen pixels, as
seen in a screenshot.

  vnc_input.py size
  vnc_input.py move X Y
  vnc_input.py click X Y
  vnc_input.py type "some text"
  vnc_input.py key enter|tab|space|esc|up|down ...
  vnc_input.py combo super+alt+space     (modifiers: super ctrl alt shift)

Set TANIX_VM_VNC_PORT for a VM on another port.
"""
import os, socket, struct, sys, time
s = socket.create_connection(("127.0.0.1", int(os.environ.get("TANIX_VM_VNC_PORT", "5901"))), timeout=10)
def rd(n):
    b = b""
    while len(b) < n: b += s.recv(n - len(b))
    return b
rd(12); s.sendall(b"RFB 003.008\n")
n = rd(1)[0]; types = rd(n)
assert 1 in types, types
s.sendall(b"\x01"); assert rd(4) == b"\0\0\0\0"
s.sendall(b"\x01")  # shared
w, h = struct.unpack(">HH", rd(4)); rd(16); rd(struct.unpack(">I", rd(4))[0])
KEYS = {"enter": 0xFF0D, "tab": 0xFF09, "space": 0x20, "esc": 0xFF1B, "down": 0xFF54, "up": 0xFF52,
        "left": 0xFF51, "right": 0xFF53, "backspace": 0xFF08, "return": 0xFF0D}
KEYS.update({"f%d" % i: 0xFFBD + i for i in range(1, 13)})
MODS = {"super": 0xFFEB, "ctrl": 0xFFE3, "alt": 0xFFE9, "shift": 0xFFE1,
        "superr": 0xFFEC, "meta": 0xFFE7}
SHIFTED = set('~!@#$%^&*()_+{}|:"<>?ABCDEFGHIJKLMNOPQRSTUVWXYZ')
def raw(sym, down): s.sendall(struct.pack(">BBHI", 4, down, 0, sym)); time.sleep(0.04)
def key(sym):
    sh = sym < 256 and chr(sym) in SHIFTED
    if sh: raw(0xFFE1, 1)
    raw(sym, 1); raw(sym, 0)
    if sh: raw(0xFFE1, 0)
    time.sleep(0.04)
def ptr(x, y, b): s.sendall(struct.pack(">BBHH", 5, b, x, y)); time.sleep(0.12)
a = sys.argv[1:]
if a[0] == "size": print(w, h)
elif a[0] == "move":
    ptr(int(a[1]), int(a[2]), 0)
elif a[0] == "click":
    x, y = int(a[1]), int(a[2]); ptr(x, y, 0); ptr(x, y, 1); ptr(x, y, 0)
elif a[0] == "type":
    for c in a[1]: key(ord(c))
elif a[0] == "key":
    for k in a[1:]: key(KEYS[k])
elif a[0] == "combo":
    *mods, last = a[1].lower().split("+")
    for m in mods: raw(MODS[m], 1)
    sym = KEYS.get(last, ord(last) if len(last) == 1 else None)
    raw(sym, 1); raw(sym, 0)
    for m in reversed(mods): raw(MODS[m], 0)
time.sleep(0.3)
