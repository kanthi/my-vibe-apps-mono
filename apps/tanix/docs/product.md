# Tanix

A custom Linux distro based on NixOS. The whole system is one declarative configuration in a flake, so any build can be reproduced and any change rolled back.

Status: the ISO builds and boots on arm64. One install (Server: Minimal) has been tested end to end.

## The job

Boot a machine, or a VM, into a system set up the way I want it: packages, desktop, defaults and branding already in place, with no manual setup after install.

## What it ships

- A flake that builds one ISO. The ISO boots into a live KDE Plasma session and carries a graphical installer.
- An installer that offers seven systems: KDE Plasma, Hyprland, Server: Minimal, Server: Web (Caddy), Server: Containers (Podman), Server: Database (PostgreSQL) and the Security Edition.
- A Tanix KDE Plasma desktop: macOS-style layout made only from stock Plasma parts.
- Tanix branding on the live session and on installed systems: name in `/etc/os-release`, logo, boot menu, boot splash, installer, wallpapers.
- A Tanix base on every system (tools, zsh and starship, flakes, clean-up, zram, optional nightly upgrades) and a finished setup per choice. See [features.md](features.md).

## Out of scope for now

- A package repository or binary cache of its own. Installs download from the NixOS cache.
- A custom installer program. Tanix adapts the Calamares installer that NixOS ships.
- Offline installs. The chosen desktop is downloaded during installation.

## Decided

- **Build host:** a NixOS machine in OrbStack on these Apple Silicon Macs (2026-11-15).
- **CPU architecture:** aarch64 first. An x86_64 ISO is defined in the flake but has never been built; it belongs on CI.
- **Channel:** NixOS stable, currently 26.05. Chosen as the safer default; not explicitly requested.
- **Desktops:** KDE Plasma and Hyprland only. The live session runs KDE Plasma. The ten other desktops the NixOS installer offers were removed (2026-11-16).
- **KDE look:** macOS-style layout using Plasma defaults only, no external themes (2026-11-16).
- **Servers:** a minimal server (SSH, no desktop) and a web server (the same plus Caddy) (2026-11-16).
- **Security Edition:** KDE Plasma cut down to a minimal base, hardened, red look. Tools are added later per category, grouped like Kali's menu; the categories exist and are empty (2026-11-17).
- **Feature round 2026-11-17:** all proposed features added except the firewall, which is decided later.
- **Servers:** SSH keys only and fail2ban on every server; nightly upgrades on servers, off on desktops; container (Podman) and database (PostgreSQL) servers added (2026-11-17).
- **HyOma** (an Omarchy-style Hyprland edition): built and install-tested on 2026-11-17, then removed from the goals for now at the user's request (2026-11-17). Not in the installer; files removed.
- **Hyprland login:** stays SDDM with its stock theme (my choice between SDDM and greetd; 2026-11-17).
- **Boot menu:** classic white on black (2026-11-16).
- **Testing:** in VMware Fusion, not emulation (2026-11-16).
- **Distribution, for now:** only the latest build, as a single ISO, until the feature set settles. Planned route: GitHub Actions builds, Cloudflare R2 hosts. Prepared but not switched on; see [building.md](building.md) (2026-11-16).
- **Installed systems are plain NixOS configurations** in `/etc/nixos`, with the Tanix module copied next to them. They do not use a flake.

## Open questions

- **Security Edition tools:** which tools in each category?
- **Firewall:** firewalld or a stricter nftables setup.
- **A Tanix web address** for the installer's links (now nixos.org).
- Target hardware: VMs only, or a specific machine too?
- Should installed systems follow a Tanix flake for updates instead of the plain NixOS channel? That would let Tanix ship changes to installed machines.
- A plain VM image (qcow2) was in the original scope. Still wanted, now that the ISO installs into a VM?

## To-do

Parked on request ("nothing yet, add all this to to-do", 2026-11-18). Nothing here is started.

**Tanix Server ISO** (see [server-iso.md](server-iso.md))
- **Offline install.** Today the ISO (528 MB) downloads the software during install and needs internet. Make it install from the ISO: carry the software for all four roles (one shared set, ~2.3 GB, roughly 700-900 MB compressed), use the network only if present. Estimated ISO size 1.2-1.4 GB. Choose one:
  1. make this ISO offline-capable (one ISO, ~1.3 GB);
  2. two variants: `tanix-server-netinstall` (528 MB) and `tanix-server` (offline, ~1.3 GB);
  3. slim the server base first (below), then add offline, aiming near 1 GB.
  Installed servers still need internet for nightly updates.
- **Slim the server base.** An installed Minimal server is ~2.0 GB of software (all roles ~2.3 GB; ~2.3-2.6 GB used on disk incl. the NixOS package source and the boot partition). Most is the shared Tanix base: kernel + modules ~310 MB, vim and helix, git, Python, Perl, Nix. Roles add only ~300 MB together. Consider a lighter base for servers (e.g. no helix).
- **Domain question in the installer** when Web is chosen, so Caddy sets up HTTPS during install. Today: plain HTTP on port 80; run `sudo tanix-web-setup` afterwards.
- **Trim the ISO further:** Python (132 MB) and Perl (57 MB) in the installer image.
- **Boot menu labels:** maybe drop the version number in front of each entry.
- **Database (PostgreSQL)** as a boot menu entry / checklist item (the module exists).
- **Cosmetic:** dialog box widths (welcome text wraps badly on some lines).
- **Not yet tested:** FTP, Containers and Custom entries end to end; SSH key import (`github:NAME`); `tanix-web-setup` on a Server-ISO install; x86_64 / BIOS; real hardware.

**Desktop ISO**
- Rebuild it to pick up the server.nix and default.nix changes (it evaluates; not rebuilt).
- Tanix KDE screenshot for the installer's System page (still the upstream picture).
- Optional: Tanix light/dark global themes, so switching to Breeze Dark keeps the Tanix icons.
- Optional: an installer page to pick the icon theme.
