# Tanix

A custom Linux distro built on NixOS. One flake builds one ISO that boots into a live desktop and can install Tanix from there.

Status: `wip`. The ISO builds and boots (arm64), and a server install was tested end to end in VMware Fusion. The other installer choices are not yet install-tested; see [AGENT_WORKLOG.md](AGENT_WORKLOG.md).

Sibling project: [Tanux](../tanux/), the Fedora-based distro. The two are independent and share no code.

## What you get

- **Live session and installer in one ISO.** It boots into KDE Plasma without touching the disk; the graphical installer (Calamares) opens on top.
- **A choice of system in the installer.** KDE Plasma, Hyprland, four servers (minimal, web with Caddy, containers with Podman, database with PostgreSQL), or the Security Edition (a minimal, hardened KDE with tool categories laid out like Kali's; the tools are still to be chosen).
- **Tanix Server ISO** (528 MB, text only). Its boot menu offers the server type (Minimal, Web, FTP, Containers, or Custom with a checklist); a short text installer asks for the disk, host name and user, then installs. Software is downloaded during the install. See [docs/server-iso.md](docs/server-iso.md).
- **A shared base on every system.** Everyday command-line tools, zsh with starship, flakes on, weekly clean-up, zram, optional nightly upgrades (on for servers). See [docs/features.md](docs/features.md).
- **A Tanix desktop.** KDE Plasma arranged with a menu bar on top and a dock at the bottom, built only from stock Plasma parts.
- **Tanix branding.** Name, logo, boot menu, boot splash, installer and wallpapers, on the live session and on installed systems.

## Build it (macOS, Apple Silicon)

Needs [OrbStack](https://orbstack.dev). Nix is not needed on the Mac.

```sh
./scripts/setup-builder.sh     # once: creates the NixOS build machine in OrbStack
./scripts/build-iso.sh         # writes ~/Workspace/FusionVMs/tanix/tanix-<version>-aarch64-linux.iso
./scripts/build-iso.sh server  # the Tanix Server ISO: tanix-server-<version>-aarch64-linux.iso
./scripts/check-desktops.sh    # checks every installer choice still evaluates
./scripts/fusion-vm.sh         # boots the ISO in a VMware Fusion test VM
```

Details: [docs/building.md](docs/building.md).

## Layout

| Path | What it is |
| --- | --- |
| `flake.nix`, `flake.lock` | Entry point and the pinned NixOS revision (NixOS 26.05) |
| `modules/iso.nix` | The desktop ISO: live session, file name, boot menu |
| `modules/iso-server.nix` | The Tanix Server ISO: size cuts, boot menu per server type |
| `modules/iso-boot-menu.nix` | Boot menu look shared by both ISOs |
| `modules/tanix/` | The Tanix module: base, branding, KDE, Hyprland, servers, Security Edition, with artwork and default configs; also copied onto installed systems |
| `installer/` | Changes to the Calamares installer: choices, branding, patch; `installer/server/` is the Server ISO's text installer |
| `scripts/` | Build, check, builder setup and test VM scripts |
| `.github/workflows/` | Automatic ISO build, prepared but not active (see docs/building.md) |
| `icon/` | The Tanix app icon (source of the logo) |
| `docs/` | Documentation, listed below |
| `build/` | Scratch output; ignored by git. ISOs and test VMs go to `~/Workspace/FusionVMs/tanix` instead (see docs/building.md). |

## Documentation

- [docs/product.md](docs/product.md): goals, scope, decisions, open questions.
- [docs/building.md](docs/building.md): building and testing on macOS.
- [docs/features.md](docs/features.md): what each installer choice includes, and the options to change it.
- [docs/installer.md](docs/installer.md): the ISO, the installer's choices, how to add one.
- [docs/server-iso.md](docs/server-iso.md): the Tanix Server ISO and its text installer.
- [docs/branding.md](docs/branding.md): what is branded, where, and how to change it.
- [AGENT_WORKLOG.md](AGENT_WORKLOG.md): current state and history for whoever works on this next.
