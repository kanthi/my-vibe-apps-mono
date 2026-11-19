# Details agent worklog

## 2026-11-01: product plan created

**Request:** create a new app called "Details" with all features and UI/UX from vitalsmac.com, write a product development plan, then choose the stack (GPUI or other) once the feature-level plan exists.

**Done**
- Created `apps/details/`: README, `.gitignore`, `.env.example`, this worklog. No lockfile, since there is no code or stack yet.
- `docs/features.md`: catalog with IDs, milestones and risk, plus the site's roadmap and exclusions.
- `docs/ui-ux.md`: surfaces, layouts, interaction rules, visual language.
- `docs/product-plan.md`: rules, budgets, milestones M0 to M6, stack requirements, spikes, risks, testing, decisions D1 to D8.
- Added a row to the root README catalog.

**Sources:** `llms-full.txt`, the feature pages and the roadmap from vitalsmac.com, read 2026-11-01 (Vitals 1.9.2), and screenshots of the site's demos. Firecrawl was unavailable, so pages were fetched with curl and headless Chrome. The site's claims and simulated demos were not checked against the real app. Risk ratings and spike ideas are my judgement and are unverified.

**Reference images:** `docs/references/` holds 8 new captures (`current/`) and 16 copied from `apps/pramana/docs/references` (`earlier/`). Git-ignored because the repo is public. `apps/pramana`'s 16 images are tracked, and in the local `origin/main` ref, so they appear to be public already; not changed.

**Status:** nothing built, no tests, no artifacts, nothing committed. `apps/pramana` and `apps/meter` not touched.

**Next steps**
1. User answers D1 to D8 in `docs/product-plan.md`. The stack decision is what the user wants next.
2. Run spike S1 to measure the stack candidates, then feasibility spikes S2 to S7.
3. Do not assume Meter's "do not affect the host" rule applies or doesn't: D6 is open, so ask before running anything that affects the host.

## 2026-11-01: mockups

**Request:** create mockups for the plan.

**Done**
- `docs/mockups/build-mockups.mjs` generates `docs/mockups/index.html`: 12 static screens. Overview (dark and light), CPU with an app unfolded, App card and process inspector, menu bar and dropdown (normal and strain), Projects with the Stop All confirmation, Agents and plans, Over Time and Peaks, Sound, assistant approval, Settings Insights, first run and honest gaps.
- Same screens published to a private Claude design canvas, "Details mockups". Run `node build-mockups.mjs <dir>` to regenerate its files.
- Linked from `docs/ui-ux.md`.

**Design choices**
- Own look: system font, hairline cards, placeholder letter marks. No Vitals or vendor artwork.
- Overview rings became segmented bars, which departs from what `ui-ux.md` described from the reference. Plan rings stay on Agents.
- Multi-series colours passed the dataviz palette validator in dark and light, including colour-blind separation. The single-subsystem colours are identity colours, always paired with an icon and label, and failed that validator as one set (blue and purple are close), so don't put them together in one chart.

**Status:** local `index.html` was rendered in headless Chrome and checked by eye; wrapping and label fixes applied. The canvas itself was not opened. All numbers are sample data. Nothing committed.

**Not mocked:** Memory, Disk, Network, GPU, Neural, Battery, Bluetooth and Containers tabs (they reuse the CPU layout), Worth a Look, other Settings pages, narrow window widths, VoiceOver.

**Next:** user reviews the mockups, then the D1 to D8 decisions above still stand.
