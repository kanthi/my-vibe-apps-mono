# Tanix Server ISO

A small, text-only installer ISO for servers, next to the desktop ISO.

```sh
./scripts/build-iso.sh server   # ~/Workspace/FusionVMs/tanix/tanix-server-<version>-aarch64-linux.iso
```

## Boot menu

| Entry | Installs |
| --- | --- |
| Install Server: Minimal (SSH only) | default entry |
| Install Server: Web (Caddy) | SSH + Caddy serving /var/www |
| Install Server: FTP (vsftpd) | SSH + FTP for the system's users |
| Install Server: Containers (Podman) | SSH + Podman with a `docker` command |
| Install Server: Custom (choose the software) | shows a checklist: SSH, web, FTP, containers |

Each entry is a NixOS specialisation that only adds `tanix.role=<type>` to the kernel command line (a few KB each). The installer reads it; a type skips the checklist, Custom shows it. List in `extraRoles`, `modules/iso-server.nix`.

## Installer (`installer/server/tanix-install.sh`)

Starts by itself on the first console (quit it for a shell; `sudo tanix-install` restarts it). Steps: welcome, internet check, disk (the ISO's own disk is hidden), host name, time zone, user and password, optional SSH key (`github:NAME`, URL or pasted), software (Custom only), summary with "Quit" as default, then: erase, GPT + EFI (or BIOS) partitions, ext4, `nixos-generate-config`, copy the Tanix module to `/etc/nixos/tanix`, write `configuration.nix` with the `tanix.server.*` lines, `nixos-install`, set the password. Log: `/tmp/tanix-install.log`.

The installed server is the same Tanix server as from the desktop ISO (base, nightly upgrades), with roles as `true/false` lines in `/etc/nixos/configuration.nix`.

## Size (528 MB; desktop ISO 3.0 GB)

Taken out of the installer image: hardware firmware, ZFS, manuals, NetworkManager (wired DHCP only), polkit, git, the Tanix base tools, desktop wallpapers; xz compression. Largest remaining: NixOS package source 197 MB (needed to install), kernel + modules 314 MB, Python 132 MB (could probably go), Perl 57 MB.

Consequences: needs a wired network with internet during install; no Wi-Fi; physical machines whose network card needs firmware are not supported by the installer (installed systems are not affected).

## Status (2026-11-18)

Tested in VMware Fusion (arm64): install with all four roles; services active, web page served, `docker` works, password login and sudo work; FTP login and upload work after the PAM fix. Boot menu entries appear; the Web entry preselects Web. Not tested: the other entries end to end, SSH key import, BIOS/x86, physical hardware.
