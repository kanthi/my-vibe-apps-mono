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
| [meter](apps/meter/) | Menu-bar monitor of local AI usage, starting with agy and Grok | macOS (not started) | wip |
| [freedium-browser-ext](apps/freedium-browser-ext/) | Opens the current page on freedium-mirror.cfd | Browser extension (Chrome, Brave, Firefox, Safari) | usable |
| [timed-macos-scr](apps/timed-macos-scr/) | 3D analog clock macOS screen saver with live seconds and top-right digital HUD | macOS (Swift / SwiftUI / ScreenSaver) | usable |

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
