# Tanux

A custom Linux distro based on Fedora: Fedora 45 underneath, with Tanux branding, desktop defaults, server configurations, and curated editions layered on top via `noarch` RPMs.

Status: `wip`. Built for `aarch64` and tested in VMware Fusion up through the start of package download (`Server: Web (Caddy)`), where testing was paused.

## The job

Install one image and get a Fedora system set up the way I want it: packages, desktop or server profile, defaults and branding already in place, with no manual setup after install.

## What it ships

- One Fedora Everything–style netinstall ISO (`build/tanux-45-aarch64.iso`, ~1.3 GB) built by customizing the official Fedora 45 Everything netinstall ISO.
- A classic white-on-black GRUB boot menu offering five editions: KDE Plasma, Sway, Server: Minimal, Server: Web (Caddy), and Security Edition.
- An interactive Anaconda installer branded with Tanux (`product.img`) that lets the user choose language, disk/partitioning, and user account while applying that edition's Kickstart package set and post-install setup.
- An embedded local DNF repository (`/tanux-repo`) inside the ISO carrying all Tanux `noarch` RPMs (`tanux-release-*`, `tanux-logos`, `tanux-backgrounds`, `tanux-desktop-kde`, `tanux-desktop-sway`, `tanux-server-web`, `tanux-security`).
- A Tanux KDE Plasma desktop with a macOS-style layout made only from stock Plasma parts, plus 4K dark and light wallpapers.

## Out of scope for now

- A hosted package repository (COPR or self-hosted DNF mirror). The embedded `/tanux-repo` inside the ISO installs offline; Fedora 45 base packages come from official Fedora mirrors.
- A live desktop session on the installer ISO (unlike Tanix). The Everything netinstall route prioritizes clean per-edition DNF installs (no KDE files on servers or Sway) and lightweight builds.
- Image-based (`bootc`) delivery.
- Third-party COPR repositories (Hyprland was replaced with Sway so every edition installs purely from official Fedora 45 repositories plus local Tanux RPMs).

## Decided

- **Route (2026-11-17): Fedora Everything–style netinstall ISO.** Starts from the official `Fedora-Everything-netinst-aarch64-45-1.4.iso`, replaces GRUB config/theme, injects Anaconda `product.img` branding, embeds `/ks/*.ks` and `/tanux-repo`, and repacks with `xorriso` + `implantisomd5`.
- **Edition chooser (2026-11-17): white-on-black GRUB boot menu.** Picks the edition Kickstart (`inst.ks=hd:LABEL=TANUX_45:/ks/<id>.ks`) while keeping Anaconda interactive for disk, user, and localization.
- **Desktops (2026-11-17): KDE Plasma and Sway.** Hyprland is not in official Fedora 45 repos; Sway (`@sway-desktop-environment`) is in official repos and replaced Hyprland to avoid third-party COPRs.
- **KDE look:** macOS-style layout using stock Plasma parts only (top bar with global menu, bottom floating dock, left window buttons) + Tanux dark wallpaper.
- **Servers:** Minimal (`openssh-server`, `NetworkManager`, `firewalld`, root SSH login disabled) and Web (`caddy` serving `/var/www` on `:80`, firewall open for SSH/HTTP/HTTPS).
- **Security Edition (2026-11-17):** KDE Plasma + `tanux-release-security` + `tanux-security` meta-package, with its tool list intentionally empty for now (matching Tanix's `security.nix`).
- **Build host:** OrbStack machine `tanux-builder` (`fedora:44` `arm64`, building `noarch` `.fc45` RPMs and repacking the `aarch64` ISO).
- **Testing:** VMware Fusion (`arm64`), attaching the ISO as a SATA disk with bridged `vmxnet3` networking.
- **Fedora release:** Fedora 45 (`45-1.4`).

## Open questions

- **Security Edition contents:** which tools to add to `Requires:` in `rpm/tanux-security.spec`? (Verified in official Fedora 45 repos: `nmap`, `wireshark`, `tcpdump`, `aircrack-ng`, `john`, `hashcat`, `hydra`, `gobuster`; `sqlmap` and `nikto` are not in official F45 repos.)
- **Target hardware:** VMs only, or a specific physical machine too?
- **Post-install updates for Tanux RPMs:** if Tanux updates its own RPMs after install, should those eventually live in a hosted repo/COPR?
- **`x86_64` build:** scripts parameterize `TANUX_ARCH`, but only `aarch64` has been built and booted so far.
