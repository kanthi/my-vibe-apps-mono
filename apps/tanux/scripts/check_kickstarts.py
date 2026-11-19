#!/usr/bin/env python3
"""Validate every Tanux Kickstart with ksvalidator and dry-run DNF5 resolution.

Mirrors Tanix's check_desktops.py: ensures every edition in installer/editions.py
generates a syntactically valid Kickstart whose package set resolves cleanly
against Fedora 45 + build/repo/.
"""
import os
import subprocess
import sys
import tempfile

APP_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(APP_DIR, "installer"))
from build_configs import render_kickstart
from editions import COMMON_EXCLUDES, EDITIONS, FEDORA_VERSION


def main() -> None:
    only = sys.argv[1:]
    repo_dir = os.path.join(APP_DIR, "build", "repo")
    if not os.path.isdir(os.path.join(repo_dir, "repodata")):
        sys.exit("build/repo/repodata missing; run ./scripts/build-rpms.sh first")

    failed = []
    with tempfile.TemporaryDirectory(prefix="tanux-check-") as tmp:
        repos_dir = os.path.join(tmp, "repos.d")
        cache_dir = os.path.join(tmp, "cache")
        os.makedirs(repos_dir)
        os.makedirs(cache_dir)
        with open(os.path.join(repos_dir, "tanux-check.repo"), "w", encoding="utf-8") as fh:
            fh.write(
                f"[f{FEDORA_VERSION}]\n"
                f"name=Fedora {FEDORA_VERSION}\n"
                f"metalink=https://mirrors.fedoraproject.org/metalink?repo=fedora-{FEDORA_VERSION}&arch=$basearch\n"
                f"enabled=1\ngpgcheck=0\n\n"
                f"[f{FEDORA_VERSION}-updates]\n"
                f"name=Fedora {FEDORA_VERSION} Updates\n"
                f"metalink=https://mirrors.fedoraproject.org/metalink?repo=updates-released-f{FEDORA_VERSION}&arch=$basearch\n"
                f"enabled=1\ngpgcheck=0\n\n"
                f"[tanux]\n"
                f"name=Tanux\n"
                f"baseurl=file://{repo_dir}\n"
                f"enabled=1\ngpgcheck=0\n"
            )

        for ed in EDITIONS:
            eid = ed["id"]
            if only and eid not in only:
                continue

            ks_path = os.path.join(tmp, f"{eid}.ks")
            with open(ks_path, "w", encoding="utf-8") as fh:
                fh.write(render_kickstart(ed))

            ks_res = subprocess.run(
                ["ksvalidator", "-v", f"F{FEDORA_VERSION}", ks_path],
                capture_output=True,
                text=True,
            )
            if ks_res.returncode != 0:
                failed.append(eid)
                print(f"FAIL  {eid} (ksvalidator)\n{ks_res.stderr}{ks_res.stdout}", flush=True)
                continue

            cmd = [
                "dnf5",
                "-q",
                f"--installroot={os.path.join(tmp, 'root-' + eid)}",
                f"--releasever={FEDORA_VERSION}",
                f"--setopt=reposdir={repos_dir}",
                f"--setopt=cachedir={cache_dir}",
                f"--exclude={','.join(COMMON_EXCLUDES)}",
                "install",
                *ed["packages"],
            ]
            dnf_res = subprocess.run(
                cmd,
                input="n\n",
                capture_output=True,
                text=True,
            )
            # `dnf5` exits 1 when the user answers 'n' to the transaction prompt;
            # verify "Transaction Summary:" appeared with zero solver errors.
            combined = dnf_res.stdout + "\n" + dnf_res.stderr
            if "Transaction Summary:" in combined and "Error:" not in combined:
                count_line = [
                    line.strip()
                    for line in combined.splitlines()
                    if "Installing:" in line
                ]
                summary = count_line[-1] if count_line else "ok"
                print(f"ok    {eid:<16} {summary}", flush=True)
            else:
                failed.append(eid)
                print(f"FAIL  {eid} (dnf5)\n{combined[-3000:]}", flush=True)

    if failed:
        sys.exit("failed: " + ", ".join(failed))


if __name__ == "__main__":
    main()
