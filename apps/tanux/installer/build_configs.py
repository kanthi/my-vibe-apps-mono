#!/usr/bin/env python3
"""Render Kickstart files and GRUB configuration from installer/editions.py."""
import argparse
import os
import shutil
import struct
import sys
import zlib

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from editions import COMMON_EXCLUDES, DISTRO_NAME, EDITIONS, FEDORA_VERSION, ISO_LABEL


def render_kickstart(ed: dict) -> str:
    lines = [
        f"# {DISTRO_NAME} {FEDORA_VERSION} Kickstart — {ed['title']}",
        "# Generated from installer/editions.py; do not edit generated outputs by hand.",
        "",
        "graphical",
        "keyboard --vckeymap=us --xlayouts='us'",
        "timezone UTC --utc",
        "network --bootproto=dhcp --device=link --activate --onboot=yes --hostname=tanux",
        "",
        'url --mirrorlist="https://mirrors.fedoraproject.org/metalink?repo=fedora-$releasever&arch=$basearch"',
        'repo --name="updates" --mirrorlist="https://mirrors.fedoraproject.org/metalink?repo=updates-released-f$releasever&arch=$basearch"',
        'repo --name="tanux" --baseurl="file:///run/install/repo/tanux-repo"',
        "",
        "selinux --enforcing",
    ]

    if ed["firewall_services"]:
        svcs = ",".join(ed["firewall_services"])
        lines.append(f"firewall --enabled --service={svcs}")
    else:
        lines.append("firewall --enabled --service=mdns")

    if ed["services"]:
        lines.append(f"services --enabled={','.join(ed['services'])}")

    lines.extend(["", "%packages"])
    lines.extend(ed["packages"])
    for excl in COMMON_EXCLUDES:
        lines.append(f"-{excl}")
    lines.append("%end")

    if ed["post"]:
        lines.extend(["", "%post --erroronfail"])
        lines.extend(ed["post"])
        lines.append("%end")

    lines.append("")
    return "\n".join(lines)


def render_grub_cfg(label: str) -> str:
    entries = []
    for ed in EDITIONS:
        ks_path = f"/ks/{ed['id']}.ks"
        entries.append(
            f"menuentry '{ed['grub_label']}' --class gnu-linux --class gnu --class os {{\n"
            f"\tlinux /images/pxeboot/vmlinuz inst.stage2=hd:LABEL={label} inst.ks=hd:LABEL={label}:{ks_path} ro rhgb quiet\n"
            f"\tinitrd /images/pxeboot/initrd.img\n"
            f"}}"
        )

    default_ks = f"/ks/{EDITIONS[0]['id']}.ks"
    return f"""set default="0"
set timeout=30

set menu_color_normal=white/black
set menu_color_highlight=black/light-gray

function load_video {{
  if [ x$feature_all_video_module = xy ]; then
    insmod all_video
  else
    insmod efi_gop
    insmod efi_uga
    insmod ieee1275_fb
    insmod vbe
    insmod vga
    insmod video_bochs
    insmod video_cirrus
  fi
}}

load_video
set gfxpayload=keep
insmod gzio
insmod part_gpt
insmod ext2
insmod gfxterm
insmod png

search --no-floppy --set=root -l '{label}'
loadfont /EFI/BOOT/fonts/unicode.pf2
terminal_output gfxterm
set theme=/EFI/BOOT/theme/theme.txt

{chr(10).join(entries)}

submenu 'Troubleshooting -->' {{
\tmenuentry 'Test this media & install {DISTRO_NAME}: KDE Plasma' --class gnu-linux --class gnu --class os {{
\t\tlinux /images/pxeboot/vmlinuz inst.stage2=hd:LABEL={label} inst.ks=hd:LABEL={label}:{default_ks} rd.live.check
\t\tinitrd /images/pxeboot/initrd.img
\t}}
\tmenuentry 'Install {DISTRO_NAME}: KDE Plasma (basic graphics mode)' --class gnu-linux --class gnu --class os {{
\t\tlinux /images/pxeboot/vmlinuz inst.stage2=hd:LABEL={label} inst.ks=hd:LABEL={label}:{default_ks} nomodeset
\t\tinitrd /images/pxeboot/initrd.img
\t}}
\tmenuentry 'Rescue a {DISTRO_NAME} system' --class gnu-linux --class gnu --class os {{
\t\tlinux /images/pxeboot/vmlinuz inst.stage2=hd:LABEL={label} inst.rescue
\t\tinitrd /images/pxeboot/initrd.img
\t}}
}}
"""


def write_png_tile(path: str, r: int, g: int, b: int) -> None:
    raw = b"".join(b"\x00" + bytes([r, g, b, 255]) * 16 for _ in range(16))

    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    ihdr = struct.pack(">IIBBBBB", 16, 16, 8, 6, 0, 0, 0)
    with open(path, "wb") as fh:
        fh.write(
            b"\x89PNG\r\n\x1a\n"
            + chunk(b"IHDR", ihdr)
            + chunk(b"IDAT", zlib.compress(raw))
            + chunk(b"IEND", b"")
        )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ks-dir", required=True)
    parser.add_argument("--grub-cfg", required=True)
    parser.add_argument("--theme-dir")
    parser.add_argument("--label", default=ISO_LABEL)
    args = parser.parse_args()

    os.makedirs(args.ks_dir, exist_ok=True)
    for ed in EDITIONS:
        out_path = os.path.join(args.ks_dir, f"{ed['id']}.ks")
        with open(out_path, "w", encoding="utf-8") as fh:
            fh.write(render_kickstart(ed))

    os.makedirs(os.path.dirname(args.grub_cfg), exist_ok=True)
    with open(args.grub_cfg, "w", encoding="utf-8") as fh:
        fh.write(render_grub_cfg(args.label))

    if args.theme_dir:
        os.makedirs(args.theme_dir, exist_ok=True)
        src_theme = os.path.join(os.path.dirname(os.path.abspath(__file__)), "grub", "theme.txt")
        shutil.copy2(src_theme, os.path.join(args.theme_dir, "theme.txt"))
        write_png_tile(os.path.join(args.theme_dir, "background.png"), 0, 0, 0)
        write_png_tile(os.path.join(args.theme_dir, "select_c.png"), 255, 255, 255)


if __name__ == "__main__":
    main()
