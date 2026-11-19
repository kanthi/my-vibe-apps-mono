# Incubator conventions

This repository is a folder of independent apps, not a coupled monorepo.

## Where to work

- Put every new app in `apps/<kebab-case-slug>/`.
- Treat that folder as a standalone repository: its own README, `.gitignore`, lockfile, env example, and scripts.
- Open the app folder as the project root when coding it, so context stays inside the app.

## Do not

- Add a root `package.json`, pnpm/npm/yarn workspace, Turborepo, or Nx.
- Add shared runtime libraries under `packages/` (or similar) that apps import.
- Add a root `tsconfig` that apps extend, a root `.env`, or CI that an app needs in order to run.
- Nest an app inside another app, or group apps by stack (`web/`, `mobile/`) or status (`wip/`, `live/`).

Status belongs in the app README and the root catalog table.

## Templates vs shared code

- `templates/` is for copy-paste starters. Copy into `apps/<slug>/`. Never import a template as a dependency.
- If two apps need the same helper, copy it for now. A real shared package is a later decision, and it blocks clean extraction.

## Lifecycle

- Abandoned: `git mv apps/<slug> archive/<slug>`.
- Graduated: extract with `git filter-repo --subdirectory-filter apps/<slug>` on a fresh clone, then leave a pointer README or move the folder to `archive/`.

## Root files

Root `.gitignore` is only OS, editor, and secrets. Stack-specific ignores (`node_modules`, `.next`, `dist`, `.venv`, Xcode userdata, …) live in the app.

## Agent Context & Work Logging Protocol

Every time an AI agent performs work on an app in this repository:
1. **Always read the app's `AGENT_WORKLOG.md` (and `AGENTS.md` / `README.md`) first** to understand the app's current state, recent changes, architecture choices, and ongoing tasks before doing any work.
2. **Every time something is done, record that information in `apps/<slug>/AGENT_WORKLOG.md`** before ending your turn. Document:
   - What was done (changes made, files touched, rationale).
   - Current build, test, and artifact status.
   - Ongoing work / next steps / unresolved questions.
3. This ensures any future agent immediately understands the ongoing context and progress without the user needing to re-explain every time.

