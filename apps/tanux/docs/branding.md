# Tanux branding

## Where the branding shows up

| Place | What | Defined in |
| --- | --- | --- |
| `/etc/os-release` | `NAME=Tanux`, `ID=tanux`, `ID_LIKE=fedora`, `LOGO=tanux`, plus per-edition `VARIANT` | `rpm/tanux-release.spec` |
| Hostname | Defaults to `tanux` | `installer/build_configs.py` (`network --hostname=tanux`) |
| ISO file and volume | `tanux-45-aarch64.iso`, volume label `TANUX_45` | `scripts/build-iso.sh`, `installer/editions.py` |
| ISO boot menu | Classic white text on black, title `"Tanux"`, five edition entries | `installer/grub/theme.txt`, `installer/build_configs.py` |
| Anaconda installer | "TANUX 45 INSTALLATION", purple sidebar & top bar, stacked-disks logo | `installer/anaconda/` -> `/images/product.img` |
| Boot splash (Plymouth) | Tanux stacked-disks mark (`system-logo-white.png`, `watermark.png`) | `rpm/tanux-logos.spec`, `assets/tanux-mark.svg` |
| Installed boot menu | Entries named `Tanux` | Follows `NAME=Tanux` in `/etc/os-release` |
| System icon | Icon named `tanux` (plus `fedora-logo-icon` / `start-here` / `kmenu` compatibility links) | `rpm/tanux-logos.spec`, `assets/tanux-logo.svg` |
| Wallpapers | 4K dark and light purple wallpapers (`tanux-dark.png`, `tanux-light.png`) | `rpm/tanux-backgrounds.spec`, `assets/wallpapers/` |
| KDE Plasma layout | Top bar (28 px) with global menu, floating bottom dock (56 px), left window buttons | `rpm/tanux-desktop-kde.spec` |
| Sway defaults | Tanux dark wallpaper + `kitty` default terminal | `rpm/tanux-desktop-sway.spec` |

## Branding RPMs

Fedora trademark rules require remixes to swap out `fedora-release` and `fedora-logos`. All Tanux branding and defaults ship as `noarch` RPMs built by `./scripts/build-rpms.sh` into `build/repo/` (embedded on the ISO at `/tanux-repo`):

- **`tanux-release`** (`common`, default, `kde`, `sway`, `server`, `server-web`, `security`): provides `system-release`, `system-release(45)`, `system-release-product`; conflicts `fedora-release`, `fedora-release-variant`, `generic-release`; sets `/usr/lib/os-release` via Lua `%post` scriptlet so empty-chroot installs never fail on missing `/bin/sh`.
- **`tanux-logos`** (`tanux-logos`, `tanux-logos-httpd`): provides `system-logos`, `system-logos-icewm`, `system-logos(httpd-logo-ng)`; conflicts `fedora-logos`, `generic-logos`; ships `tanux` icons, menu icon symlinks, and Plymouth `spinner` watermark.
- **`tanux-backgrounds`**: renders `tanux-dark.svg` and `tanux-light.svg` at 3840x2160 into `/usr/share/backgrounds/tanux/`, `/usr/share/wallpapers/Tanux-{dark,light}/` (KDE), and `/usr/share/gnome-background-properties/tanux.xml`.
- **`tanux-desktop-kde`**: installs `/usr/share/tanux/plasma-layout.js`, `/usr/libexec/tanux-plasma-setup` (runs once per user via `/etc/xdg/autostart/tanux-plasma-setup.desktop`), and `/etc/xdg/kwinrc` (`ButtonsOnLeft=XIA`, `ButtonsOnRight=`).
- **`tanux-desktop-sway`**: installs `/etc/sway/config.d/90-tanux.conf`, which `sway-config-fedora` includes automatically.
- **`tanux-server-web`**: installs `/etc/tanux/Caddyfile`, a `caddy.service.d/tanux.conf` drop-in, `tmpfiles.d` for `/var/www`, and a systemd preset enabling `caddy.service`.
- **`tanux-security`**: meta-package requiring `tanux-desktop-kde` with an empty `Requires:` block for future security tools.

## Palette and artwork

- Tanux purple palette (from `icon/AppIcon.svg`): `#120826` (deepest), `#371A6B`, `#5B35A4`, `#6C42BD`, `#8E6AD8`, `#9D6BEA`, `#CDB7F6`, `#F1E9FF`.
- `assets/tanux-logo.svg` matches `icon/AppIcon.svg`.
- `assets/tanux-mark.svg` extracts the three stacked purple-white disks on a transparent 512x512 canvas.

## Not branded yet

- Login and lock screens (`plasmalogin`, `sddm`, Plasma lock screen) use their stock Fedora look.
- Installer URLs in `tanux-release.spec` (`HOME_URL`, `SUPPORT_URL`, `BUG_REPORT_URL`) point at `fedoraproject.org` / `bugzilla.redhat.com` since Tanux has no site of its own.
- Breeze colors and icon theme are stock Plasma Breeze.
