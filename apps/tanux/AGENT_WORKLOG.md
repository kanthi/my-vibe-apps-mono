# Tanux agent worklog

## 2026-11-14: project created

**Request:** "add two new apps called Tanix (Nix Based custom desitro) and Tanux ( Fedora based custom linux distro)". The user also said: do not commit anything; remind them when a commit seems necessary.
**Choice:** route not chosen yet (bootc image vs classic kickstart ISO); see docs/product.md.

**Done**
- Scaffolded `apps/tanux/` per the incubator rules: README, `docs/product.md` (goals, scope, open questions), stack `.gitignore` (built images, build folders, secrets), `.env.example`, this worklog.
- Root README catalog row added.

**Status:** spec only. No build files yet, so no lockfile yet. Nothing committed.

**Next steps**
1. Answer the open questions in `docs/product.md`, mainly desktop, target architecture and the build host (these Macs are Apple Silicon, so builds need a Linux VM or CI).
2. Add the first build definition and its lockfile, then build a VM image.

## 2026-11-15/16: plan written to match Tanix's features

**Request:** (made while working on Tanix) one ISO that live-boots and installs, a Calamares installer offering several desktops (KDE, Hyprland, ...), custom branding with wallpapers, built on macOS with OrbStack; "also document the same in the fedora based tanux too". Still: do not commit.

**Done**
- `docs/plan.md` (new): how Tanux gets the same features. Route, live image, installer module sequence, desktops, branding RPMs, building on macOS, order of work.
- `docs/product.md`: "What it ships" extended; new "Decided" section (classic route, Calamares, KDE live session, OrbStack builder, aarch64 first, Fedora 45); open questions reduced to three.
- `README.md`: stack section now states the chosen route.

**Checked facts (2026-11-15, Fedora bodhi and mdapi rawhide metadata)**
- Current releases: Fedora 43, 44, 45.
- In official repos: `calamares` 3.3.14, `sway` 1.12, `cosmic-session` 1.11.0, `plasma-desktop` 6.8.3, `lorax` 46.2, `livecd-tools`, `generic-release`, `generic-logos`.
- Not in official repos: `hyprland`, `niri`.

**Status:** plan only. No build files, no builder machine, no lockfile. Nothing committed.

**Next steps**
1. `orb create fedora tanux-builder` and test whether `livemedia-creator --no-virt` runs there with the stock Fedora KDE kickstart. This is the main unknown.
2. Then follow "Order of work" in `docs/plan.md`.
3. Root README catalog row updated on 2026-11-16 to name the chosen route.

**Update 2026-11-16:** the user narrowed Tanix to five installer choices (KDE Plasma with a macOS-style layout, Hyprland, Server: Minimal, Server: Web with Caddy, Security Edition with tools still to be chosen), asked for a white-on-black boot menu, and Tanix testing moved to VMware Fusion. `docs/plan.md` and `docs/product.md` here were updated to match. Niri, Sway, COSMIC and the other desktops are no longer in scope. Still plan only; nothing built.

## 2026-11-17: Fedora Everything netinst route chosen, built, and VMware Fusion tested (paused mid-install)

**Requests & Decisions (from user)**
1. Pivoted away from 3 GB Live KDE + Calamares (`docs/plan.md`) to a **Fedora Everything–style netinstall ISO** with Tanix's goals (~1.3 GB, clean per-edition DNF5 installs, fast builds, no live desktop session).
2. **Edition chooser:** Classic white-on-black GRUB boot menu selects among the 5 editions (`Install Tanux: KDE Plasma`, `Install Tanux: Sway`, `Install Tanux: Server Minimal`, `Install Tanux: Server Web (Caddy)`, `Install Tanux: Security Edition`), booting interactive Anaconda Kickstarts (`graphical`) so disk, user, and localization remain interactive in the GUI.
3. **Tiling WM:** Replaced Hyprland with **Sway** (`@sway-desktop-environment`, verified in official Fedora 45 repos; Hyprland is not in official F45 repos and the user chose Sway to avoid third-party COPR repos).
4. **Security Edition:** KDE Plasma + empty `tanux-security` meta-package for now (matching `tanix/modules/tanix/security.nix`; tool list to be decided later).
5. User asked to test in VMware Fusion, then asked to **"pause the testing"** once the `Server: Web (Caddy)` install reached active RPM downloading.

**Checked facts (2026-11-17, Fedora 45 repos, `src.fedoraproject.org`, and `Fedora-Everything-netinst-aarch64-45-1.4.iso`)**
- Official ISO verified: `Fedora-Everything-netinst-aarch64-45-1.4.iso` (1,426,759,680 bytes, SHA-256 verified against official `CHECKSUM`), uses classic GTK Anaconda (no `anaconda-webui` inside `install.img`).
- `calamares` 3.3.14-11.fc45 on Fedora defaults to `unpackfs` (`/run/rootfsbase`), confirming Everything/Anaconda is much cleaner for multi-edition netinstalls.
- Fedora 45 `@kde-desktop` uses `plasma-login-manager` (`plasmalogin.service`), NOT `sddm`, and explicitly lists `fedora-release-kde-desktop`; `@swaywm-extended` uses `sddm` + `sddm-wayland-sway` and `/etc/sway/config` includes `/etc/sway/config.d/*.conf`.
- `caddy` 2.11.4-4.fc45 uses `/etc/caddy/Caddyfile` (`http://`) and pulls `system-logos(httpd-logo-ng)`, satisfied cleanly by `tanux-logos-httpd`.
- `plymouth-theme-spinner` relies on `/usr/share/plymouth/themes/spinner/watermark.png` (owned by `fedora-logos` / `tanux-logos`).

**Files created / updated**
- `rpm/*.spec`: 7 spec files producing 14 `noarch` `.fc45` RPMs:
  - `tanux-release.spec` (`common`, default, `kde`, `sway`, `server`, `server-web`, `security`) using Lua `%post -p <lua>` symlink for `/usr/lib/os-release` so empty-chroot installs never fail before `bash`/`coreutils`.
  - `tanux-logos.spec` (`tanux-logos`, `tanux-logos-httpd`)
  - `tanux-backgrounds.spec` (renders 4K `tanux-dark.png` / `tanux-light.png` + KDE & GNOME metadata)
  - `tanux-desktop-kde.spec` (once-per-user Plasma macOS-style top bar + bottom floating dock + left window buttons in `/etc/xdg/kwinrc`)
  - `tanux-desktop-sway.spec` (`/etc/sway/config.d/90-tanux.conf` wallpaper + `kitty`)
  - `tanux-server-web.spec` (`/etc/tanux/Caddyfile`, `caddy.service.d/tanux.conf`, `/var/www` tmpfiles, `caddy.service` preset, offline firewalld HTTP/HTTPS)
  - `tanux-security.spec` (depends on `tanux-desktop-kde`, empty tool list for now)
- `assets/`: `tanux-logo.svg`, `tanux-mark.svg`, `wallpapers/tanux-dark.svg`, `wallpapers/tanux-light.svg` (rendered cleanly via `rsvg-convert`).
- `installer/`: `editions.py` (single source of truth), `build_configs.py` (renders `/ks/*.ks`, `/EFI/BOOT/grub.cfg`, and GRUB theme PNG tiles), `grub/theme.txt`, and `anaconda/` (`fedora.css`, `sidebar-bg.svg`, `sidebar-logo.svg`, `topbar-bg.svg`, `anaconda_header.svg`) packed into `/images/product.img`.
- `scripts/`: `setup-builder.sh`, `build-rpms.sh`, `check_kickstarts.py`, `check-kickstarts.sh`, `build-iso.sh`, `fusion-vm.sh`, `vnc_input.py`.
- `.github/workflows/build-iso.yml` (prepared for post-extraction CI on `ubuntu-24.04-arm` in `quay.io/fedora/fedora:45` -> Cloudflare R2; inactive while in monorepo).
- `docs/`: `product.md`, `building.md`, `installer.md`, `branding.md`, `plan.md` (marked superseded), `screenshots/` (`boot-menu.png`, `installer-welcome.png`, `installer-summary.png`, `installer-progress.png`).
- `README.md` and root `../../README.md` catalog row updated.

**Verified (2026-11-17)**
- OrbStack builder `tanux-builder` (`fedora:44`, `aarch64`) created and lean toolchain installed.
- `./scripts/build-rpms.sh`: built all 14 `noarch` `.fc45` RPMs into `build/repo/`.
- `./scripts/check-kickstarts.sh`: all 5 editions pass `ksvalidator -v F45` and `dnf5 --releasever=45` transaction resolution (`kde`: 2205 packages, `sway`: 1456, `server-minimal`: 535, `server-web`: 538, `security`: 2206).
- `./scripts/build-iso.sh`: built `build/tanux-45-aarch64.iso` (1.3 GB) with `implantisomd5`.
- VMware Fusion (`arm64`):
  - White-on-black GRUB menu boots with all 5 Tanux choices (`docs/screenshots/boot-menu.png`).
  - `Install Tanux: Server Web (Caddy)` booted into Tanux-branded Anaconda (`WELCOME TO TANUX 45.`, seamless purple sidebar and top bar, `docs/screenshots/installer-welcome.png`).
  - Installation Summary hub came up with Keyboard, Language Support, Time & Date, Installation Source, Software Selection (`Custom software selected`), and Network (`Connected: enp2s0`, hostname `tanux`) pre-configured cleanly (`docs/screenshots/installer-summary.png`).
  - Selected 30 GB NVMe disk, created admin user `tester`, clicked **Begin Installation**, and confirmed active download (`Downloading 574 RPMs, 73.1 MiB / 656.96 MiB (11%) done.`, `docs/screenshots/installer-progress.png`) before pausing the VM (`vmrun pause`).

**Not verified / paused**
- Note: `scripts/vnc_input.py` was slightly extended (`backspace`, `delete`, `home`, `end` keys added) after `build/tanux-45-aarch64.iso` was built; `build-iso.sh` and `check-kickstarts.sh` do not use `vnc_input.py`.
- The VMware Fusion VM (`build/vm/tanux-test.vmx`) is currently **paused** mid-download (`11%`) per user request (`"pause the testing"`).
- Not yet verified: completion of the `Server: Web (Caddy)` install, booting the installed disk (`./scripts/fusion-vm.sh disk`), verifying `/etc/os-release`, `sshd`, `caddy`, `curl -I localhost`, and full installs of `KDE Plasma` (Plasma layout + wallpapers + `plasmalogin`), `Sway` (`sddm` + wallpaper + `kitty`), `Server: Minimal`, and `Security Edition`.

**Next steps**
1. Resume testing when ready: either unpause the existing VM (`"/Applications/VMware Fusion.app/Contents/Library/vmrun" unpause "$PWD/build/vm/tanux-test.vmx"`) to finish `Server: Web (Caddy)` or run `./scripts/fusion-vm.sh` for a fresh run; after install completes, run `./scripts/fusion-vm.sh disk` and check `os-release`, `sshd`, `caddy`, `curl -I localhost`.
2. Install-test `KDE Plasma` and `Sway` in Fusion to verify `tanux-desktop-kde` (top bar, dock, left window buttons, wallpaper) and `tanux-desktop-sway`.
3. Populate `Requires:` in `rpm/tanux-security.spec` once the Security Edition tool list is chosen.
4. Commit when ready (remind user: everything under `apps/tanux/` and root `README.md` changes are currently uncommitted).
