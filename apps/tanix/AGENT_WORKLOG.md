# Tanix agent worklog

## 2026-11-14: project created

**Request:** "add two new apps called Tanix (Nix Based custom desitro) and Tanux ( Fedora based custom linux distro)". The user also said: do not commit anything; remind them when a commit seems necessary.
**Choice:** the build definition is a NixOS flake (planned; nothing written yet).

**Done**
- Scaffolded `apps/tanix/` per the incubator rules: README, `docs/product.md` (goals, scope, open questions), stack `.gitignore` (built images, build folders, secrets), `.env.example`, this worklog.
- Root README catalog row added.

**Status:** spec only. No build files yet, so no lockfile yet. Nothing committed.

**Next steps**
1. Answer the open questions in `docs/product.md`, mainly desktop, target architecture and the build host (these Macs are Apple Silicon, so builds need a Linux VM or CI).
2. Add the first build definition and its lockfile, then build a VM image.

## 2026-11-15 and 2026-11-16: first flake, installer, branding, tested in a VM

**Requests, in order (all from the user; still: do not commit)**
1. Build on macOS with OrbStack; one ISO that both live-boots and installs with a GUI installer like NixOS has; several desktops in the Calamares installer; custom branding and wallpapers; document it all, and document the same for `apps/tanux/`.
2. Add a "Server only (minimal)" choice next to the desktops.
3. "can i see the test vm booting".
4. Boot menu colours: "classic white on black".
5. The MacBook was overheating: find a way with less load.
6. Add "Server (Webserver - Caddy)"; remove all desktops except Hyprland and KDE; KDE gets a macOS-like layout "using defaults only without externals" plus Tanix wallpapers; add a "Security Edition (like kali but minimal - will decide what goes into it)".
7. Asked whether this can move to GitHub with automatic ISO builds, whether free GitHub allows it, and whether Cloudflare's free tier is better; then: "For now only latest build, single iso would be good, untill I get all i want right".

**Decisions**
- Builder: OrbStack machine `tanix-builder` (NixOS, arm64). Flakes are off by default there, so scripts pass `--extra-experimental-features`. `apps/tanix` is untracked, so flake refs use `path:.`.
- Channel `nixos-26.05`, pinned in `flake.lock` (my choice of stable; the user did not pick). aarch64 only so far; `iso-x86_64` is defined and never built.
- The ISO is the stock NixOS "graphical Calamares Plasma 6" image plus the Tanix module and an overlay on the installer.
- Installed systems are plain (non-flake) NixOS configs in `/etc/nixos`, importing a copy of `modules/tanix` at `/etc/nixos/tanix`.
- Security Edition = the KDE choice + `tanix.security.enable`. The tool list in `modules/tanix/security.nix` is intentionally empty; the user will decide. Basing it on KDE was my assumption.

**Files**
- `flake.nix`, `flake.lock`: outputs `packages.<system>.iso`, `installer-extensions`, `nixosModules.tanix`, `overlays.installer`.
- `modules/iso.nix`: ISO name, volume label, white-on-black GRUB theme (`bootMenuTheme`), `/etc/tanix`, no screen locking in the live session, and `nixpkgs.flake.setNixPath = false` (see "Bugs found").
- `modules/tanix/default.nix` + `assets/`: os-release, logo icon, Plymouth logo, wallpapers, the KDE layout script, window buttons left, Hyprland wallpaper service. `modules/tanix/security.nix`: Security Edition options.
- `installer/`: overlay (`default.nix`), `patches/tanix-installer.patch`, `config/packagechooser.conf` (5 choices), `branding/tanix/`.
- `scripts/`: `setup-builder.sh`, `build-iso.sh`, `check-desktops.sh` + `check_desktops.py`, `fusion-vm.sh`, `vnc_input.py`.
- `docs/`: `building.md`, `installer.md`, `branding.md`, `product.md`, `screenshots/`.

**Bugs found by testing and fixed**
- Enlightenment did not evaluate in the pinned nixpkgs (moot now: removed with the other desktops).
- Install failed with "experimental Nix feature 'flakes' is disabled": a flake-built system sets `NIX_PATH` to `flake:nixpkgs`, which the installer's plain `nixos-install` cannot use. Fixed in `modules/iso.nix`.
- With no GRUB theme, the ISO draws unselected menu entries in black, unreadable on a dark background. Replaced by a proper theme.
- A GRUB `label` with id `__timeout__` showed its placeholder text literally; a `progress_bar` works.
- The Plasma lock screen appeared during a long install; locking is now off in the live session.

**Verified (2026-11-16, VMware Fusion, arm64)**
- ISO boots; boot menu, live KDE session with the Tanix layout and wallpaper, installer branding and the 5-choice System page all seen on screen (see `docs/screenshots/`).
- Full install of **Server: Minimal**: completed, booted from disk, login worked, os-release says Tanix, `/etc/nixos/tanix` imported, sshd active, no display manager.
- Full install of **Server: Web (Caddy)**: completed, caddy and sshd active, `curl -I localhost` answers `Server: Caddy`.
- `./scripts/check-desktops.sh`: all 5 choices evaluate.

**Not verified**
- Installs of KDE Plasma, Hyprland and Security Edition. Unknowns: whether the KDE layout applies on an installed system (seen only in the live session), whether the Hyprland `swaybg` service starts, whether the global menu shows for real apps.
- The x86_64 ISO. Real hardware. Plymouth logo (only a spinner was seen during boot).

**Test setup notes (cost real time; keep)**
- Fusion: attach the ISO as a disk, not a CD (as a CD the live system got CD read errors mid-install and died, leaving a half-configured system with no password). Use bridged networking and the vmxnet3 card. `scripts/fusion-vm.sh` does all three.
- `vmcli ... MKS captureScreenshot` works for screenshots. `vmcli MKS sendKeyEvent/sendKeySequence` was unreliable; `scripts/vnc_input.py` over the VM's VNC port works for mouse and keys.
- Do not test under QEMU emulation in the builder: it pins 8 cores and overheated the Mac. A QEMU process was once left running by a failed `pkill -f` (the pattern matched the calling shell); kill by PID.
- `replaceDependencies` in `installer/default.nix` avoids recompiling Calamares on every installer change.
- Disk: after two days of rebuilds the builder's nix store had grown to 95 GB (every ISO build leaves a 3 GB image behind). `nix-collect-garbage` inside the builder freed 93 GB on 2026-11-16; the next build re-downloads its tools. OrbStack hands the space back to macOS gradually, not at once.
- A Fusion VM "Tanix test" may still exist in `build/vm` (the user was exploring it); `./scripts/fusion-vm.sh stop` powers it off.

**Next steps**
1. Install-test KDE Plasma, Hyprland and Security Edition in Fusion (`./scripts/fusion-vm.sh`, then `disk`).
2. Replace the KDE picture on the System page (still upstream's stock Plasma screenshot) with a screenshot of the Tanix layout.
3. User to decide the Security Edition tool list; put it in `modules/tanix/security.nix`.
4. Automatic builds: the user chose "only latest build, single iso" for now (2026-11-16). `.github/workflows/build-iso.yml` is written (GitHub Actions builds on an arm64 runner, uploads one fixed-name ISO to Cloudflare R2) but has never run, and is inert until the app is extracted to its own repo. Remaining setup is the user's: extract the repo, create the R2 bucket and token, add the GitHub secrets. Steps are in `docs/building.md`. Provider limits quoted there are from memory, unverified.
5. Known gaps: login and lock screens unbranded; live user is `nixos`; "NixOS Manual" icon on the live desktop; installer links point at nixos.org; Breeze colours and icons are stock.
6. A commit is due whenever the user wants one: everything under `apps/tanix` and `apps/tanux` is untracked, and the root `README.md` rows changed.

## 2026-11-17: test VMs cleaned up (and the ISO lost with them)

**Request:** "First clean all the vms used for testing so far".

**Done**
- Deleted with `vmrun deleteVM`: `build/vm` ("Tanix test", suspended, 4 GB) and `~/Workspace/FusionVMs/Tanix Test  64-bit Arm.vmwarevm` (made in the Fusion GUI, 14 MB, empty disk, one snapshot).
- `build/vm-b` ("Tanix test b") was already gone from disk; only its Fusion library entry remained.
- Removed all three entries from Fusion's library file (`~/Library/Application Support/VMware Fusion/vmInventory`) while Fusion was closed.
- Left alone: `FusionVMs/Templates/nixos-minimal-golden` (not a Tanix test VM) and the OrbStack machines `tanix-builder`, `tanux-builder`, `clab` (builders, not test VMs).

**Mistake:** `vmrun deleteVM` also deleted `build/tanix-26.05.20261113.f1e5c35-aarch64-linux.iso`, because both VMs had the ISO attached as a disk and deleteVM removes every attached disk file. There is no ISO now. The builder's store was garbage-collected on 2026-11-16, so there is no copy there either; `./scripts/build-iso.sh` rebuilds it (re-downloads its tools first).

**Rule:** never use `vmrun deleteVM` on these VMs. Power off, then `rm -rf` the VM folder (as `scripts/fusion-vm.sh` already does).

**Status:** no test VMs, no ISO. Source files untouched. Nothing committed.

**Next steps:** rebuild the ISO before any further VM testing; then the 2026-11-16 list still stands.

## 2026-11-17: feature round (all proposed features except the firewall)

**Requests:** "with out testing anything yet, lets discuss adding features to existing ones"; I proposed a list; then "except firewall add propesed features, we shall decide about firewalld later on"; then "For security KDE, lets make KDE as minimal as possible for the base and then later we shall add tools just like in Kali Linux for each catergory".

**Done** (full description in `docs/features.md`)
- New modules in `modules/tanix/`: `base.nix` (tools, flakes, weekly gc + optimise, zram, zsh + starship + fastfetch, `tanix.autoUpgrade`), `plasma.nix` (SDDM + lock screen wallpaper, accent colour, KCalc/Filelight/Partition Manager, Flatpak + Flathub), `hyprland.nix` + `assets/hyprland/*` (config, Waybar, hyprlock, hypridle, mako, cliphist, grim/slurp; copied into ~/.config by a user service), `server.nix` (SSH keys only, fail2ban, `tanix-add-ssh-key`, Tailscale toggle, web/containers/database roles, `tanix-web-setup`, welcome page).
- `security.nix` rewritten: minimal Plasma (excludePackages, no orca/kde-pim/baloo/printing/flatpak/extras), AppArmor, random MAC, no sshd, red wallpaper + accent, 13 empty Kali-style categories.
- `default.nix`: imports the new modules; `security` wallpaper variant (sed recolour of the dark SVG at build time); options `tanix.branding.accentColor` and internal `wallpaperPath`.
- `iso.nix`: live user `tanix` (stock `nixos` user disabled, autologin/getty/trusted-users/helpLine forced), desktop icons without the NixOS manual, Flatpak off in the live session.
- Installer: two new choices `server-containers`, `server-database` (packagechooser.conf, tiles, `TANIX_SERVER_IDS`); server snippets now just set `tanix.server.*`; web server config moved from the patch into `server.nix`; new `cfgtanixupdates` line (`true` for servers, `false` for desktops); Security Edition gets no printing (`tanix_misc()`). Patch regenerated against the pinned upstream `main.py` and checked to re-apply cleanly.
- `scripts/check_desktops.py` updated to match. Docs: new `docs/features.md`; README, installer.md, branding.md, product.md updated.

**My decisions (not asked):** zsh as the default shell (over fish); vim + helix as editors; autoUpgrade written into configuration.nix instead of an installer toggle (Calamares has no simple toggle page); domain/e-mail via `tanix-web-setup` after install (Calamares System page has no text fields); Hyprland keeps SDDM stock theme (greetd not done); tor/proxychains and the wireshark group left for the Security tool decision; web server keeps opening 80/443 (as tested) until the firewall decision.

**Verified:** evaluation only. `./scripts/check-desktops.sh`: all 7 choices ok. The ISO configuration evaluates (`isoImage.drvPath`). The wallpaper derivation builds, and the red security wallpaper looks right. One bug fixed: `services.orca.enable` conflicted with Plasma's mkDefault; now `mkOverride 900`.

**Not verified (nothing installed or booted):** everything at runtime. Riskiest: the live-user rename (activation script, Calamares running as `tanix`), the Hyprland defaults copy racing Hyprland's own placeholder config, SDDM picking up `theme.conf.user`, the lock screen and accent settings via /etc/xdg, Flathub service, keys-only SSH on a fresh server (log in on the console first).

**Status:** no ISO exists (lost on 2026-11-17, see above). Builder stopped again. Nothing committed.

**Next steps**
1. Rebuild the ISO and install-test each choice, starting with the live session (user rename) and KDE.
2. Take the Tanix KDE screenshot for the installer's System page during that test.
3. User: tools per Security category; firewall choice; a Tanix web address for the installer links.
4. A commit is due whenever the user wants one.

## 2026-11-17: "Hyprland Omarchy" edition, documented only

**Request:** "add another edition called "Hyperland Omarch" replicating omarchy hyperland features, start with documenting about it and discuss before adding packages etc".

**Done:** research, then `docs/hyoma.md` (first written as `docs/omarchy-edition.md`) (what Omarchy 3 and 4 contain, a feature-by-feature map to NixOS, three build approaches, seven open decisions). No code, packages or installer changes.

**Key findings:** current Omarchy is 4.x "Quattro" (Quickshell shell, Lua Hyprland config for Hyprland 0.56). The pinned NixOS 26.05 has Hyprland 0.55.4, so Omarchy 4 does not fit stable; every Omarchy 3 component (Walker 2.16.2 + Elephant, SwayOSD, Waybar, mako, hyprlock, hypridle, Foot, lazygit, mise) is in nixpkgs. Several Omarchy default apps are unfree and x86_64-only. Community NixOS ports exist (codingismy11to7/omarchy, henrysipp/omarchy-nix, omnixy).

**My recommendation (awaiting user):** approach A, Omarchy 3 feature set on stable, with Omarchy's MIT dotfiles/themes vendored with attribution.

**Renamed:** the user named the edition **HyOma** ("rename it as HyOma"). Doc moved to `docs/hyoma.md`; planned installer id `hyoma`, label "Desktop: HyOma".

**Next:** user answers the remaining decisions at the end of `docs/hyoma.md`; then module, installer choice, tile, check script, docs.

## 2026-11-17: HyOma built (minimal), ports compared, build output moved

**Requests:** "yes collect existing ports compare it with official omarchy, and then create our own version of it"; then, rescoping: "we are not intending to get app to app similarity from omarchy, i would like to get similar setup, like ai agent tools install options, layout and keybindings to start off. We start with Tanix theme (dark/light) only for now. exclude x64 apps for now. make it minimal usable for first time. we can take it from there. To test the iso use vmware fusion".

**Done**
- Cloned (read-only, scratchpad) official Omarchy v3.8.4 and the ports codingismy11to7/omarchy, henrysipp/omarchy-nix, omnixy; comparison table in `docs/hyoma.md`.
- `modules/tanix/hyoma.nix`: fetches Omarchy v3.8.4 (pinned hash), installs it at `/run/current-system/sw/share/omarchy` with patches: Arch-only commands -> `assets/hyoma/bin` (`hyoma-not-on-nixos`, NixOS `omarchy-update`, `omarchy-pkg-present/missing`, `omarchy-first-run`, `omarchy-launch-wifi` via nmtui); AI submenu replaced (`assets/hyoma/install-ai-menu.sh`: Claude Code, Codex, Gemini CLI, Copilot CLI, OpenCode, Crush, Pi via Omarchy's `omarchy-npx-install`); themes replaced by Tanix + Tanix Light (`assets/hyoma/themes/`); Arch paths fixed; fcitx5 dropped; `cp --no-preserve=mode` for copies out of the store. User service `hyoma-setup` links `~/.local/share/omarchy` and seeds `~/.config` once. Minimal package list, no unfree/x86-only. nix-ld on (for mise's Node.js). SDDM with Omarchy's theme, Flatpak, Elephant and SwayOSD user services.
- Installer: choice `hyoma` ("Desktop: HyOma"), snippet `tanix.hyoma.enable = true; tanix.hyoma.theme = "Tanix";`, tile. Patch regenerated and re-applies.
- `base.nix`: default shell zsh now at priority 900 (HyOma sets bash); Flathub service moved here from plasma.nix (any system with Flatpak on). `hyprland.nix` and the swaybg service skip themselves when HyOma is on.
- **Build output moved out of the app folder** (`scripts/build-iso.sh`, `scripts/fusion-vm.sh`, docs): ISO and test VM now in `~/Workspace/FusionVMs/tanix` (`TANIX_OUT`). Reason: `path:.` copies the whole app folder, `build/` included, into the store on every build; a 3 GB ISO there was re-copied every time (likely the 95 GB of 2026-11-16), and a growing log in `build/` failed one build ("unexpected end-of-file").
- `scripts/vnc_input.py`: `combo` command for key chords (super/ctrl/alt/shift).

**Verified (builder, no VM):** all 8 choices evaluate. The HyOma Omarchy package builds; themes = tanix, tanix-light; AI menu replaced and `bash -n` clean; no `#!/bin/bash` left. `hyoma-setup` run against a throwaway home: links, config copy, bashrc, Walker/Elephant set-up, Tanix theme applied with background; second run keeps user edits; switching to Tanix Light works (first attempt failed: read-only copied theme could not be replaced; fixed with `--no-preserve=mode`).

## 2026-11-17: HyOma first VM test (Fusion), then cleanup

**Install test:** ISO built and booted in Fusion; live user `tanix` and desktop icons (GParted, Install System) correct. Installer System page shows "Desktop: HyOma" with its tile. Install of HyOma (erase disk, no unfree) completed in ~8 min. Installed system boots to SDDM with "Hyprland (uwsm-managed)" preselected, and login works.

**Faults found in the installed HyOma (not yet fixed):**
1. **SDDM shows the stock theme, not Omarchy's.** Log: theme "requires missing .../sddm-wrapped/bin/sddm-greeter". Omarchy's `metadata.desktop` has no `QtVersion=6`, so SDDM looks for the Qt5 greeter. Fix: add `QtVersion=6` to the theme's metadata in the build.
2. **Config error at line 23 of hyprland.conf** (`source = ~/.local/state/omarchy/toggles/hypr/*.conf`, no match). Omarchy's installer creates that folder and copies `default/hypr/toggles/flags.conf` (`install/config/omarchy-toggles.sh`). Fix: do the same in `hyoma-setup`.
3. **First login only: "omarchy-hyprland-monitor-watch: command not found".** uwsm read `~/.config/uwsm/env` (which puts Omarchy's bin on PATH) before `hyoma-setup` had copied it. On the second login it was gone. Fix idea: a system-wide uwsm env (`/etc/xdg/uwsm/env`) or ensure setup runs before uwsm starts.
4. **`xkbcli` missing** (used by `omarchy-menu-keybindings`, i.e. Super+K). Add the package that provides it.
5. **"Learn Keybindings" notice shows on every login**: first-run marker probably not cleared; check `omarchy-first-run`.
6. **No Waybar, and no key binding worked** (Super+Return, Super+K, even Ctrl+Alt+F2 inside Hyprland); mouse worked (notifications dismissable). The same VNC chords work at SDDM and on the console, so input reaches the VM. Cause not found yet. A diagnostics run (`hyprctl` from an autostart script) wrote empty files because the hard reset lost unsynced writes; rerun with `sync` was stopped by the user.
- Harmless noise: Elephant's archlinuxpkgs provider (no pacman), elephant runner `sbin` stats.

**Cleanup (user request "delete all test vms on fusion, also clean all that can be in build server too"):** test VM powered off and its folder `~/Workspace/FusionVMs/tanix/vm` deleted (10 GB; not `vmrun deleteVM`); its entry removed from Fusion's `vmInventory` (backup in the session scratchpad). `~/Workspace/FusionVMs/Templates` untouched. Builder: `nix-collect-garbage -d` (root and user) freed 12.2 GB, store disk 24 GB -> 13 GB used; builder stopped. The ISO `~/Workspace/FusionVMs/tanix/tanix-26.05.20261113.f1e5c35-aarch64-linux.iso` (3.1 GB) was kept.

`scripts/vnc_input.py`: added F1-F12 and `superr`/`meta` modifiers.

**Next:** fix 1-5, find cause of 6 (likely Hyprland not processing binds or crashing parts of the session; get `hyprctl binds` and hyprland.log with `sync`), rebuild, retest. A commit is due whenever the user wants one.

## 2026-11-17: HyOma removed from the goals

**Request:** "remove HyOma for now from the list of out goals and its files".

**Removed:** `modules/tanix/hyoma.nix`, `modules/tanix/assets/hyoma/`, `docs/hyoma.md`, `build/hyoma-eval.nix`, `build/hyoma-setup-test.sh`; the `hyoma` choice in `installer/config/packagechooser.conf`, its tile in `installer/default.nix`, the `cfghyoma` snippet and branch in `installer/patches/tanix-installer.patch` (regenerated from the saved upstream `main.py`, dry-run applies); HyOma conditions in `hyprland.nix` and `default.nix` (swaybg), comments in `base.nix`; mentions in README, `docs/installer.md`, `docs/features.md`. `docs/product.md` "Decided" keeps one line recording that HyOma was built, tested and removed for now.
- Kept: base's zsh default stays at priority 900 (still correct without HyOma); `scripts/vnc_input.py` additions.
- Backup of the removed files (and the docs/modules/installer/scripts as they were): session scratchpad `hyoma-removed/hyoma-files.tgz` (temporary; may be cleared). The fault list from the HyOma VM test above remains in this log if HyOma comes back.

**Verified:** `./scripts/check-desktops.sh`: all 7 choices ok. ISO evaluates. Not rebuilt; the ISO in `~/Workspace/FusionVMs/tanix` still contains HyOma. Builder stopped.

**Next:** rebuild the ISO when next testing. A commit is due whenever the user wants one.

## 2026-11-18: Tanix icon themes, 3D as default

**Request:** "i have a couple of custom icon themes in Downloads called tanix* make them default icons for my tanix, make 3d default with both available in the installer, do not build iso yet".

**Done**
- Sources: `~/Downloads/tanix-minimal.tar.gz` and `tanix-minimal-3d.tar.gz` (inspected: only `index.theme` + SVGs + relative symlinks, no scripts or external links). Extracted to `modules/tanix/assets/icons/tanix-minimal/` and `.../tanix-minimal-3d/`, symlinks kept. Each covers 23 app icon names (Dolphin, Konsole, Kate, Firefox, Discover, System Settings, Gwenview, Okular, Spectacle, KCalc and generic names) and `Inherits=breeze,hicolor`.
- `modules/tanix/plasma.nix`: package `tanix-icon-themes` (both themes in `share/icons`), installed on every Plasma system; new option `tanix.desktop.iconTheme` (enum `tanix-minimal-3d` default, `tanix-minimal`, `breeze`) written as `[Icons] Theme=` in `/etc/xdg/kdeglobals`. Users can switch in System Settings > Icons.
- Docs: `docs/branding.md` (table row, "Changing things"), `docs/features.md` (KDE Plasma section).

**Interpretation (my decision):** "both available in the installer" taken as: both themes ship on the ISO (live session, where the installer runs) and on every installed Plasma system (KDE, Security Edition), 3D active by default, flat selectable in System Settings. No extra Calamares page for picking the icon theme (packagechooser is one choice per page; a second page needs a patch change). Not applied to Hyprland (it has no icon theme set and no Breeze for fallback).

**Verified (builder, evaluation only, no ISO built):** `./scripts/check-desktops.sh`: all 7 choices ok. ISO evaluates; `/etc/xdg/kdeglobals` contains `Theme=tanix-minimal-3d`. The `tanix-icon-themes` package builds, both theme folders present, symlinks intact. `assets/icons` reaches installed systems through the existing `/etc/tanix` copy of `modules/tanix`. Builder stopped.

**Not verified:** how the icons look in Plasma (dock, launcher, Dolphin); whether the Security Edition should keep the blue/purple icons next to its red look.

**Next:** rebuild the ISO when the user asks, check the icons in the live session; optionally an installer page to choose the icon theme. Icons for Hyprland: dropped, user said "forget about hyperland for now" (2026-11-18). A commit is due whenever the user wants one.

## 2026-11-18: ISO rebuilt (icon themes, no HyOma)

**Request:** "build the iso now". `./scripts/build-iso.sh` succeeded; `~/Workspace/FusionVMs/tanix/tanix-26.05.20261113.f1e5c35-aarch64-linux.iso` (3.0 GB) replaced the previous ISO (which still had HyOma). Contains the Tanix icon themes (3D default). Builder stopped. Not booted or tested yet. Hyprland stays in the installer (user: "do not remove it"); only Hyprland icons are dropped.

**Next:** boot in Fusion (`./scripts/fusion-vm.sh`), check icons in the live session and the install choices.

## 2026-11-18: icon themes tested in Fusion, two fixes, ISO rebuilt

**Request:** "once built, test it in vmware fusion".

**First boot (ISO from 10:15):** dock showed stock Breeze icons. Cause: at login Plasma writes the Breeze look-and-feel defaults (`[Icons] Theme=breeze`) to `~/.config/kdedefaults/kdeglobals`, which beats `/etc/xdg/kdeglobals`. Also: every new Konsole showed zsh's "new user" wizard (no `~/.zshrc`).

**Fixes**
- `modules/tanix/default.nix`: the once-per-user Plasma setup script (wallpaper + layout) now also runs `plasma-workspace/libexec/plasma-changeicons <tanix.desktop.iconTheme>`, which writes the user's `~/.config/kdeglobals` and tells running apps. The `/etc/xdg/kdeglobals` default stays as a fallback.
- `modules/tanix/base.nix`: `programs.zsh.shellInit` defines `zsh-newuser-install() { :; }` (in /etc/zshenv) so the wizard never starts.
- `docs/branding.md` notes the per-user apply.

**Second boot (rebuilt ISO, 10:35):** first look showed Breeze Dark icons; the journal showed the user had opened System Settings and picked the Breeze Dark global theme (confirmed by the user), and KDE resets icons when a global theme is applied. After a clean reboot with no input: dock and launcher show Tanix Minimal 3D icons, `~/.config/kdeglobals` has `Theme=tanix-minimal-3d`, Konsole opens without the zsh wizard. All 7 choices evaluate.

**Known behaviour:** choosing a global theme in System Settings (e.g. Breeze Dark) replaces the icons with that theme's (breeze-dark). A Tanix global theme (light/dark, defaulting to the Tanix icons) would fix that; not done.

**Status:** ISO `~/Workspace/FusionVMs/tanix/tanix-26.05.20261113.f1e5c35-aarch64-linux.iso` (10:35 build) has both fixes. Test VM left running (live session; nothing installed). Builder stopped. Not tested: an installed system (first-login apply there is the same script, untested), the flat theme option.

**Next:** install-test KDE from this ISO; optional Tanix global theme; commit when the user wants.

## 2026-11-18: KDE install test from the 10:35 ISO: passed

Installed "Desktop: KDE Plasma" in the Fusion test VM (erase disk, no unfree, test user `tanixt` / `tanixtest`), ~8 min. Installed system boots to SDDM (form appears after a key press/mouse move), login works. First login: Tanix wallpaper, menu bar + dock, **Tanix Minimal 3D icons** in the dock, `~/.config/kdeglobals` `Theme=tanix-minimal-3d`; Konsole opens without the zsh wizard. Observed: the KDE tile on the installer's System page is still the upstream screenshot (known, see earlier next steps). Note for VNC automation: typing into the desktop while the installer has focus changes its language (typed "exit" chose Esperanto).

User asked to stop after this test. Test VM left running on the installed system. Nothing committed.

**Cleanup (user: "stop the vm and delete it too"):** test VM stopped, folder `~/Workspace/FusionVMs/tanix/vm` deleted, its entry removed from Fusion's `vmInventory` (backup in the session scratchpad). ISO (10:35 build) kept. Templates untouched.

## 2026-11-18: Tanix Server ISO (text installer, boot menu per server type)

**Requests:** "lets build a smallest server iso with ssh,web,ftp,containers for now"; then "option to select server types at live iso boot menu, like Server minimal, Server Web, Server Containers, etc"; "in depth is not required at the moment".

**Done** (details in `docs/server-iso.md`)
- `modules/iso-server.nix` (new): minimal NixOS installer image + Tanix module; size cuts (no firmware, ZFS, docs, NetworkManager, polkit, git, tanix.base; xz); live user `tanix`; installer autostarts on tty1; boot menu entries via specialisations (`tanix.role=web|ftp|containers|custom`, default Minimal).
- `installer/server/tanix-install.sh` (new): dialog-based installer (writeShellApplication, shellcheck at build).
- `modules/iso-boot-menu.nix` (new): GRUB theme moved out of `iso.nix`, shared.
- `server.nix`: new `tanix.server.ssh.enable` (default true; SSH/fail2ban now conditional) and `tanix.server.ftp.enable` (vsftpd, local users chrooted, passive 51000-51099, firewall 21; `security.pam.services.vsftpd = { }` because NixOS only defines it for virtual users; without it every login failed).
- `default.nix`: wallpapers installed only with Plasma or Hyprland.
- `flake.nix`: `server-iso-{aarch64,x86_64}`, package `iso-server`. `build-iso.sh [desktop|server]`; `fusion-vm.sh` honours `TANIX_ISO`.
- Docs: README, new `docs/server-iso.md`. `build/server-eval.nix` is a throwaway eval check (ignored by git).

**Verified:** all 7 desktop-installer choices still evaluate; desktop ISO evaluates. Server ISO built: 528 MB. Fusion: full install (all roles) OK, services active; FTP fixed and verified on the test VM by hand; rebuilt ISO shows the 5 boot entries, Web entry preselects Web. Not tested: other entries end to end, SSH key import, x86/BIOS, real hardware. The "Done" dialog -> reboot happened without my input once (likely the user).

**Status:** ISOs in `~/Workspace/FusionVMs/tanix`: desktop (10:35) and server (12:50). Test VM running the server ISO installer (welcome screen, Web). Builder stopped. Nothing committed.

**Next / ideas:** trim Python/Perl from the server ISO; database role in the boot menu; cosmetic dialog widths; desktop ISO not rebuilt since the server.nix/default.nix changes (evaluates fine).

## 2026-11-18: logo concepts (striped T, T-visor helm)

**Request:** a new Tanix logo with a few variations, using two references the user shared: a white mark of converging stripes on an orange-to-purple blurred gradient, and the T-shaped visor of a well-known film helmet. Both are inspiration only: the marks are original shapes, not copies of either.

**Done**
- `icon/concepts/` (new): 10 icon concepts as 1024px SVG + PNG, and 3 bare white marks (SVG, transparent, no tile) for boot splash, docs and wallpapers.
  - Striped T (chevron wings over a tapering stem, cut into stripes): `01` on an ember gradient, `02` on a Tanix-blue gradient, `03` ember-filled on black, `10` Tanix blue on light.
  - T-visor helm (generic rounded helm, T visor with a glass sheen): `04` silver against an orange sun, `05` gunmetal on ember, `06` striped white helm on ember, `09` white helm on Tanix blue.
  - `07` badge with a T-visor cut on Tanix blue; `08` the glass T visor alone on ember.
  - Marks: `mark-stripe-t.svg`, `mark-helm-striped.svg`, `mark-badge.svg`.
- Nothing wired in: `modules/tanix/assets/tanix-logo.svg` (the blue cube), the Plymouth logo and the installer branding are unchanged.

**Status:** checked as rendered contact sheets only, not at 16-32 px. Not committed.

**Next:** the user picks a direction; then replace `tanix-logo.svg`, the Plymouth logo and `installer/branding/tanix/` images, and update `docs/branding.md`.

## 2026-11-18: sizes measured, to-do list added

Measured on the builder: Server ISO 528 MB; installed server software Minimal ~2.0 GiB, all roles ~2.3 GiB (closure sizes; plus ~200 MB NixOS channel and the 512 MB ESP on disk). The test VM's installed disk was wiped when the VM was recreated for the boot-menu test, so not measured on disk. `build/server-eval.nix` now uses `path:` (git mode fails on untracked files).

User asked whether the ISO can install offline: not yet (netinstall). Options and all other open items parked in the new **To-do** section of `docs/product.md` on request ("nothing yet, add all this to to-do"). Nothing started.

**Cleanup (user: "stop build server and vms and delete test-vms"):** builder stopped; test VM stopped, `~/Workspace/FusionVMs/tanix/vm` deleted, its Fusion `vmInventory` entry removed (backup in the session scratchpad). Both ISOs kept. No VMs running.

## 2026-11-18: brand concepts (logo marks, wordmark, lockups, banners)

**Request:** the user clarified Tanix is a distro, so they want a logo and branding, not app-icon tiles.

**Done** (`branding/concepts/`, all generated SVG; banners also as 1920x1080 PNG)
- `marks/`: four one-colour marks, each in ember gradient, blue gradient, white and ink: `ridge` (striped chevron T), `visor` (striped helm with a T visor cut), `hex` (T visor in a hexagon, a nod to Nix), `signal` (the T visor alone, cut by stripes).
- `lockups/`: TANIX wordmark (geometric caps drawn as strokes, so no font dependency) in solid and cut versions, ink and white; five horizontal and three stacked mark + wordmark lockups.
- `banners/`: four full-bleed heroes (ridge on ember, ridge + wordmark on blue, visor against a sun, signal on black).
- Proposed palette: existing Tanix blues (#0A1030, #1A379E, #6A9BFF, #C9D8FF) plus ember #FF7A1A, flame #E0341A, violet #9A2BD8, ink #0B0D12, paper #F5F3EE.
- The earlier app-icon-tile concepts in `icon/concepts/` are kept for now; the brand set supersedes them.

**Status:** checked as rendered sheets only, not at small sizes. Nothing wired in (`modules/tanix/assets/tanix-logo.svg`, Plymouth, installer branding unchanged). Not committed.

**Next:** user picks a mark direction; then apply it to the logo, Plymouth, GRUB theme, installer branding, wallpapers and `docs/branding.md`.

## 2026-11-18: brand concepts, round 2 (trident, bastion, seal)

**Request:** "delete last one that you designed and recreate a complete new ones" from two new references: a red segmented trident breaking into particles over murky teal water, and a gold business card with a blocky shield crest (battlements, a T inside) and a dial.

**Done**
- Deleted the round-1 brand set (ridge/visor/hex/signal marks, its lockups and banners) from `branding/concepts/`.
- New set in `branding/concepts/`, generated SVG (banners also 1920x1080 PNG). All marks are original, built from blocks with clean cuts; no reference logo copied.
  - `marks/`: `trident` (three blades, split twin shaft; centre blade + crossbar read as a T), `bastion` (battlemented shield with a T outline and chevron cuts), `seal` (trident inside a split ring). Each in crimson, gold, ink, white.
  - `lockups/`: TANIX wordmark (stroke-drawn caps, one horizontal cut) in ink, white, gold; four horizontal and three stacked lockups.
  - `banners/`: `01` faceted trident glowing in teal water and breaking into fragments; `02` gold and black business cards; `03` gold bastion on ink; `04` seal on crimson.
- Palette: crimson #E3243B, garnet #A3122A, deep teal #0E2730, teal #2A5A66, gold #D8AE52, pale gold #F4D98A, ink #0E0F12, bone #EFE9DC.
- Design canvas board "Tanix brand concepts" replaced with the new sheet.
- `icon/concepts/` (first-round app-tile concepts) is still there, superseded.

**Status:** checked as rendered sheets at full size only, not at 16-32 px. Nothing wired in (`modules/tanix/assets/tanix-logo.svg`, Plymouth, installer branding unchanged). Not committed.

**Next:** user picks a mark; then test it small, and apply to logo, Plymouth, GRUB theme, installer branding, wallpapers and `docs/branding.md`. Consider deleting `icon/concepts/`.
