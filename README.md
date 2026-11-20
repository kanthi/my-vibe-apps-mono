# my-vibe-apps-mono

Incubator for apps I vibe-code. Each app under `apps/` is a standalone project. When one is good enough to live on its own, it gets extracted into a new repo with its git history.

This is **not** a coupled monorepo. There is no root workspace, no shared packages, and no shared lockfile. Apps do not import each other.

## Layout

```text
apps/         active apps (one folder per app, kebab-case slug)
archive/      abandoned or graduated apps (git mv, do not delete)
templates/    copy-paste starters — copy into apps/, never import
```

## Apps

| App | What it is | Stack | Status |
|-----|------------|-------|--------|
| [pramana](apps/pramana/) | Menu-bar Activity Monitor that lists apps, not processes | macOS (not started) | wip |
| [focus](apps/focus/) | Menu-bar Pomodoro timer | macOS (not started) | wip |
| [meter](apps/meter/) | Mac app and menu-bar monitor of local AI usage: Codex, Grok, Claude Code, Cursor, agy | macOS (Swift / SwiftUI, SwiftPM) | wip |
| [meter-gpui](apps/meter-gpui/) | Rust + GPUI rewrite of Meter, with an AppKit menu bar item | macOS (Rust / GPUI / objc2) | wip |
| [machealth](apps/machealth/) | Local-first Mac health monitor, plain-English task manager, AI agent tracker, load balancer, and cleaner | macOS (Rust / GPUI / NSH / objc2) | wip |
| [details](apps/details/) | App-first Mac activity monitor with history, alerts, and dev server, container and agent awareness | macOS (stack not chosen) | wip |
| [freedium-browser-ext](apps/freedium-browser-ext/) | Opens the current page on freedium-mirror.cfd | Browser extension (Chrome, Brave, Firefox, Safari) | usable |
| [timed-macos-scr](apps/timed-macos-scr/) | 3D analog clock macOS screen saver with live seconds and top-right digital HUD | macOS (Swift / SwiftUI / ScreenSaver) | usable |
| [khoop](apps/khoop/) | Offline-first WHOOP companion & calorie deficit tracker | Multidevice: Swift / SwiftUI (iOS, macOS, watchOS) & Kotlin / Compose (Android) | usable |
| [tanix](apps/tanix/) | Custom Linux distro based on NixOS: one flake builds a live + installer ISO offering KDE, Hyprland, two server setups and a Security Edition | Linux distro (NixOS / Nix flakes, Calamares) | wip |
| [tanux](apps/tanux/) | Custom Linux distro based on Fedora 45: Everything-style netinstall ISO offering KDE, Sway, two server setups and a Security Edition | Linux distro (Fedora 45; Everything netinst + Anaconda + RPMs) | wip |
| [guarto](apps/guarto/) | Renamed Octavo and moved to [kanthi/octavo](https://github.com/kanthi/octavo): Markdown books to a website, EPUB and PDF | CLI (Go) | graduated |

Status is one of: `wip`, `usable`, `shipped`, `archived`, `graduated`.

## New app

1. Create `apps/<slug>/` as if it were already its own repository.
2. Give it a `README.md`, a stack-specific `.gitignore`, `.env.example`, and its own lockfile.
3. Add a row to the table above.

Do not add a root `package.json`, workspaces, or shared libraries that apps import. That makes extraction a rewrite.

## Done with an app

```bash
git mv apps/<slug> archive/<slug>
```

Then update this table.

## Extract to its own repo

Run on a **fresh clone**, not your working copy:

```bash
git clone https://github.com/kanthi/my-vibe-apps-mono.git <new-app>
cd <new-app>
git filter-repo --subdirectory-filter apps/<slug>
gh repo create kanthi/<slug> --private --source . --push
```

After that, replace `apps/<slug>/` here with a short README that points at the new repo (or `git mv` it to `archive/`).
