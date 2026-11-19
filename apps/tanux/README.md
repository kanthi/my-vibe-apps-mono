# Tanux

A custom Linux distro built on Fedora 45. One build pipeline produces a Fedora Everything–style netinstall ISO that boots straight into a Tanux-branded installer offering five editions.

Status: `wip`. The RPMs, Kickstarts, and arm64 ISO build cleanly; all five editions pass Kickstart validation and DNF5 dependency resolution; the white-on-black GRUB menu, branded Anaconda installer, and `Server: Web (Caddy)` installation download start (`73.1 MiB / 656.96 MiB`) were verified in VMware Fusion before pausing testing. End-to-end completion and booted post-install checks are not yet verified; see [AGENT_WORKLOG.md](AGENT_WORKLOG.md).

Sibling project: [Tanix](../tanix/), the NixOS-based distro. The two are independent and share no code.

## What you get

- **Everything-style netinstall ISO (~1.3 GB).** Boots into a classic white-on-black GRUB menu where you pick the edition, then runs an interactive Anaconda install pulling only that edition's packages from Fedora 45 mirrors plus the embedded `/tanux-repo` RPM repository.
- **Five editions.** KDE Plasma, Sway, Server: Minimal, Server: Web (Caddy), and Security Edition (KDE plus an empty meta-package ready for a curated security tool set).
- **A Tanux KDE desktop.** KDE Plasma arranged with a menu bar on top, a floating dock at the bottom, and window buttons on the left, built only from stock Plasma parts.
- **Tanux branding.** Name in `/etc/os-release` (`NAME=Tanux`, `ID=tanux`, `LOGO=tanux`), system icon, boot menu, Plymouth boot splash, Anaconda branding (`product.img`), and 4K dark/light wallpapers shipped as `noarch` RPMs.

## Build it (macOS, Apple Silicon)

Needs [OrbStack](https://orbstack.dev). RPM/ISO tools run inside a Fedora OrbStack machine.

```sh
./scripts/setup-builder.sh     # once: creates tanux-builder (Fedora 44 arm64) and installs build tools
./scripts/check-kickstarts.sh  # validates every Kickstart and dry-runs DNF5 package resolution
./scripts/build-iso.sh         # builds RPMs and writes build/tanux-45-aarch64.iso
./scripts/fusion-vm.sh         # boots the ISO in a VMware Fusion test VM
```

Details: [docs/building.md](docs/building.md).

## Layout

| Path | What it is |
| --- | --- |
| `rpm/` | `noarch` RPM specs (`tanux-release`, `tanux-logos`, `tanux-backgrounds`, `tanux-desktop-kde`, `tanux-desktop-sway`, `tanux-server-web`, `tanux-security`) |
| `assets/` | Tanux SVG logo, transparent stacked-disk mark, and 4K dark/light SVG wallpapers |
| `installer/` | Single edition table (`editions.py`), Kickstart/GRUB generator (`build_configs.py`), white-on-black GRUB theme, and Anaconda branding assets (`product.img`) |
| `scripts/` | Builder setup, RPM build, Kickstart/DNF5 checks, ISO build, Fusion test VM, and VNC automation |
| `.github/workflows/` | Automatic ISO build, prepared but not active (see [docs/building.md](docs/building.md)) |
| `icon/` | The Tanux app icon (source of the logo) |
| `docs/` | Documentation, listed below |
| `build/` | Upstream ISO cache, local RPM repo, built ISO, and Fusion VM; ignored by git |

## Documentation

- [docs/product.md](docs/product.md): goals, scope, decisions, open questions.
- [docs/building.md](docs/building.md): building and testing on macOS.
- [docs/installer.md](docs/installer.md): the ISO, the five editions, how to add or change one.
- [docs/branding.md](docs/branding.md): what is branded, RPMs, and how to change artwork or layouts.
- [AGENT_WORKLOG.md](AGENT_WORKLOG.md): current state and history for whoever works on this next.
