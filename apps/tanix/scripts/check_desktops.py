#!/usr/bin/env python3
"""Check that every desktop offered by the Tanix installer evaluates.

For each desktop id in installer/config/packagechooser.conf this script puts
together the same configuration.nix the installer would write (using the text
snippets from the patched installer module), then asks Nix to evaluate the
whole system against the nixpkgs revision pinned in flake.lock.

It catches renamed or removed NixOS options and typos in the snippets. It does
not build or boot anything.

Usage: check_desktops.py <patched main.py> <app folder> [desktop id ...]
"""
import ast
import os
import re
import subprocess
import sys
import tempfile

main_py, app_dir = sys.argv[1], os.path.abspath(sys.argv[2])
only = sys.argv[3:]

# Module-level string constants named cfg* are the configuration snippets.
snippets = {}
for node in ast.parse(open(main_py).read()).body:
    if isinstance(node, ast.Assign) and isinstance(node.value, ast.Constant):
        name = node.targets[0].id
        if name.startswith("cfg") and isinstance(node.value.value, str):
            snippets[name] = node.value.value

chooser = open(os.path.join(app_dir, "installer/config/packagechooser.conf")).read()
ids = re.findall(r"^\s*- id: (\S+)\s*$", chooser, flags=re.M)
ids = [i.strip('"') for i in ids]

HARDWARE = """\
{ ... }:
{
  fileSystems."/" = { device = "/dev/disk/by-label/tanix"; fsType = "ext4"; };
  fileSystems."/boot" = { device = "/dev/disk/by-label/BOOT"; fsType = "vfat"; };
  nixpkgs.hostPlatform = builtins.currentSystem;
}
"""

EXPR = """\
let
  flake = builtins.getFlake "path:%(app)s";
  system = flake.inputs.nixpkgs.lib.nixosSystem { modules = [ %(cfg)s ]; };
in
  "${system.config.system.nixos.distroName} ${system.config.system.build.toplevel.drvPath}"
"""

# Must match TANIX_SERVER_IDS in the installer patch.
SERVER_IDS = ("", "server-web", "server-containers", "server-database")

failed = []
for desktop in ids:
    if only and (desktop or "server") not in only:
        continue
    label = desktop or "server"
    cfg = snippets["cfghead"] + snippets["cfgbootefi"] + snippets["cfgnetwork"]
    cfg += snippets["cfgconnman" if desktop == "enlightenment" else "cfgnetworkmanager"]
    if desktop in ("mate", "lxqt"):
        cfg += snippets["cfgnmapplet"]
    is_desktop = desktop not in SERVER_IDS
    if is_desktop:
        misc = snippets["cfgmisc"]
        if desktop == "security":
            misc = misc.replace(
                "services.printing.enable = true;", "services.printing.enable = false;"
            )
        cfg += snippets["cfg" + desktop] + misc
    else:
        cfg += snippets["cfgserver"]
        if desktop:
            cfg += snippets["cfg" + desktop.replace("-", "")]
    cfg += snippets["cfgtanixupdates"]
    cfg += snippets["cfgusers"]
    if is_desktop:
        cfg += snippets["cfgautologin"] + snippets["cfgfirefox"]
    else:
        cfg += snippets["cfgautologintty"]
    cfg += snippets["cfgpkgs"] + snippets["cfgtail"]
    for key, value in {
        "hostname": "tanix",
        "username": "tester",
        "fullname": "Tanix Tester",
        "groups": '"networkmanager" "wheel"',
        "pkgs": "",
        "nixosversion": "26.05",
        "tanixautoupgrade": "false" if is_desktop else "true",
    }.items():
        cfg = cfg.replace("@@%s@@" % key, value)

    with tempfile.TemporaryDirectory(prefix="tanix-check-") as tmp:
        open(os.path.join(tmp, "configuration.nix"), "w").write(cfg)
        open(os.path.join(tmp, "hardware-configuration.nix"), "w").write(HARDWARE)
        os.symlink(os.path.join(app_dir, "modules/tanix"), os.path.join(tmp, "tanix"))
        expr = EXPR % {"app": app_dir, "cfg": os.path.join(tmp, "configuration.nix")}
        result = subprocess.run(
            [
                "nix",
                "--extra-experimental-features",
                "nix-command flakes",
                "eval",
                "--impure",
                "--raw",
                "--expr",
                expr,
            ],
            capture_output=True,
            text=True,
        )
    if result.returncode == 0 and result.stdout.startswith("Tanix /nix/store/"):
        print("ok    %-14s %s" % (label, result.stdout.split()[1]), flush=True)
    else:
        failed.append(label)
        print("FAIL  %s\n%s" % (label, result.stderr[-3000:]), flush=True)

if failed:
    sys.exit("failed: " + ", ".join(failed))
