# Details: product development plan

Status: draft for review, 1 November 2026. No stack is chosen and no code exists.

Companion documents: [features.md](features.md) lists every feature by ID. [ui-ux.md](ui-ux.md) covers the design.

## 1. What we are building

A native Mac monitor that answers "which app is doing this?" It replaces Activity Monitor for everyday questions, and adds history, alerts, developer awareness and a menu bar presence.

The reference is [Vitals](https://vitalsmac.com) 1.9.2. The target is its full shipped feature set, minus the commercial parts listed in section 18 of `features.md`. Its public roadmap comes after version 1.

**Primary user:** you, a developer running several coding agents, dev servers and containers on one Mac.

**Why build it, when Vitals costs $15 to $39:** that is a question worth answering first. Reasons might be learning, control, or wanting it open or local-only. The answer changes how much of this plan is worth building, so it affects D5.

## 2. Product rules

1. Apps first. Processes are one click away and never hidden.
2. Nothing leaves the Mac by default. No account, no analytics, no telemetry.
3. Nothing closes without a confirmation, and the app never quits anything on its own.
4. Network and credential features are off until turned on.
5. Missing data is labelled. A false zero is a bug.
6. The monitor must not become the problem (see the budgets below).

## 3. Quality budgets

| Measure | Target | Basis |
|---|---|---|
| Background sampling | Every 5 seconds with the window closed | Site's stated behaviour |
| Idle CPU, window closed | About 0.1% of one core | Site's claim, unverified |
| Faster sampling | Only while a window or dropdown is open | Site's stated behaviour |
| Memory, window closed | Set after the stack spike | meter-gpui measured about 60 MB in a VM, doing far less |
| History file | Measure one simulated year in M2 | The site says only "small" |
| CLI and MCP answer | Reads the app's data with no new sampling | Site's stated design |

Treat these as release gates. CPU and memory get measured on every milestone.

## 4. Milestones

Each milestone ends with something you can use daily. Sizes are relative, and I have not estimated dates because the stack is not chosen.

### M0: Decisions and spikes (next)

Settle D1 to D8 below, then run the spikes in section 6.

**Done when:** the stack is chosen from measurements, and each `H`-risk feature is marked feasible, reduced or dropped.

### M1: Live monitor (L)

Sampling engine, app grouping, Overview, CPU, Memory, Disk and Network tabs, a single menu bar item with a dropdown, find, quit with confirmation, units, Dock option.

Features: APP-1 to 5, APP-8, APP-10, TAB-1 to 5, BAR-5, BAR-10, CUS-6, PLT-3.

**Done when:** you can answer "what is using my memory?" faster than in Activity Monitor, and the grouped totals reconcile with `ps` and `top` within an agreed margin.

### M2: Daily driver (L)

Individual menu bar items and styles, red under strain, full dropdown, history, Over Time, Peaks, alerts, Worth a Look, open at login.

Features: BAR-1 to 4, BAR-6 to 9, HIS-1 to 6, ALR-1 to 6, APP-9, PLT-1.

**Done when:** a week of real use produces accurate history and no nuisance alerts, and the app can answer "why was it slow at three?"

### M3: Developer view (L)

Projects, ports, Containers, Agents, Process Inspector, browser split, columns, and the command line.

Features: TAB-11 to 13, DEV-1 to 4, DOC-1 to 4, AGT-1 to 5, INS-1 to 9, APP-6, APP-7, CLI-1 to 5.

**Done when:** a forgotten dev server and an agent's leftovers can be found and stopped from the window, the dropdown and the terminal.

### M4: Hardware (L, highest risk)

Helper, GPU, Neural Engine, temperatures, fans, power per app, sleep holders, wake-ups, battery, Bluetooth, sound mixer, heat view.

Features: SYS-1, TAB-6 to 10, SEN-1 to 3, PWR-1 to 3, SND-1 to 3, ALR-7.

**Done when:** each reading is either checked against a trusted tool or labelled as an estimate. Fans always return to automatic, including after a crash.

Fan control has real hardware consequences, so it ships last in this milestone, with its own failure testing.

### M5: Assistants and plans (M)

MCP server, approval window, audit log, AI plan limits and token counts.

Features: MCP-1 to 8, LIM-1 to 6.

**Done when:** an assistant can answer "what is on port 8080?" and cannot quit anything without your approval, proven by tests that try.

### M6: Personal touches and finish (M)

Palettes, fonts, density, arranging the window and dropdown, image export, Shortcuts, URL scheme, further languages.

Features: CUS-1 to 5, EXP-1, AUT-1, CLI-6, PLT-2.

**Done when:** you would choose Details over Activity Monitor every day. That is version 1.

### After version 1

Pick from the roadmap in section 17 of `features.md`. Storage map and Mac Health look the most useful.

### Order rationale

- The sampling engine and app grouping come first, because every other feature depends on them.
- M3 comes before M4 because it serves your main use and uses mostly documented APIs.
- The riskiest hardware work comes after the app is already useful, so a failed spike costs one tab and not the product.

M3 and M5 could swap if assistant access matters more to you than the inspector.

## 5. Architecture needs, without choosing a stack

These hold whichever stack we choose:

- **One engine, many surfaces.** A background sampler owns the data. The window, dropdown, CLI and MCP all read from it, which keeps their numbers identical.
- **Pure core.** Grouping, alert rules, pace and history logic stay free of UI and system calls, so they can be tested with recorded fixtures.
- **Local socket.** The CLI and MCP reach the app through a local-only channel.
- **Separate helper.** A minimal privileged process that takes no general commands.
- **SQLite history** with rollups, documented for the user.
- **Safety checks in one place.** A single quit-verdict function (safe, ask, refuse) serves the UI, CLI and MCP.

## 6. What the feature list demands of a stack (input for D1)

| Requirement | Used by | Why it matters |
|---|---|---|
| Several status items with custom drawing, ⌘-drag | BAR-1, BAR-2 | Native AppKit API. Other stacks must bridge to it |
| Dense tables: sorting, columns, unfolding, context menus | APP, INS | Needs a mature table or a custom build |
| Text input everywhere: find, thresholds, settings | APP-5, ALR-4 | The meter-gpui worklog notes it had no text input widget |
| Charts refreshing cheaply | All tabs | Drives CPU budget while open |
| Very low idle cost, including UI framework load | Section 3 | Core product promise |
| Direct C and Objective-C access: libproc, IOKit, Core Audio, Security | Most tabs | Needed by any stack, and differs mainly in convenience |
| Privileged helper, login item, notifications | SYS-1, PLT-1, ALR | Needs code signing, whichever stack |
| App Intents for Shortcuts | AUT-1 | I believe this requires Swift. Verify it |
| VoiceOver support | PLT-3 | The meter-gpui worklog says GPUI limits window accessibility |
| Localisation | PLT-2 | Plan the string system early |
| CLI and MCP binaries, sharing core code | CLI, MCP | Easy in Swift or Rust |

Candidates to compare, from what is already in this repo:

- **Swift with AppKit and SwiftUI**, as in `apps/meter`.
- **Rust with GPUI and objc2**, as in `apps/meter-gpui`.
- **A hybrid:** Rust core with a native Swift UI.

**Spike S1 (stack):** build the same small thing in the leading options: a 5-second sampler, three menu bar items, a dropdown, a 100-row live table and a chart. Measure idle CPU, memory, and how the table and text input feel. This turns D1 into a choice based on measurements.

**Feasibility spikes** (my knowledge of macOS, all unverified until run):

| Spike | Question | Starting idea |
|---|---|---|
| S2 Attribution | Can helpers be assigned correctly across real apps? | Process tree plus responsible-process lookup |
| S3 GPU and Neural Engine | Can we read use per chip and per app? | IOReport and IOKit, undocumented and chip-dependent |
| S4 Temperature and fans | Readings and control on your Mac | SMC or HID sensors, helper required for writes |
| S5 Per-app volume | Can it work without a driver? | Core Audio process taps, newer macOS only |
| S6 Agent state | How can "waiting for you" be detected, without reading conversations? | Process and I/O signals, or the tools' own status files |
| S7 Plan limits | What is available locally for each provider? | Reuse Meter's readers first, then see D4 |

## 7. Risks

| Risk | Effect | Response |
|---|---|---|
| Private APIs change with macOS or new chips | GPU, Neural Engine, temperature and power break. The site's changelog shows repeated per-chip fixes, which suggests lasting upkeep | Isolate per-chip code. Label estimates. Fail by saying "unavailable" |
| Fan control fault | Overheating or stuck fans | Stay within reported limits. Always restore automatic on exit and crash. Ship it last, or drop it |
| Quitting the wrong thing | Lost work | One verdict function, always confirm, refuse system processes, test heavily |
| Assistant misuse | Unwanted quits, or data sent to an AI provider | Off by default, approval window, no environment data, visibility switches, audit log |
| Stored sign-ins | Credential handling and provider terms | D4. Prefer local logs |
| Monitor overhead | Defeats its purpose | Budgets as release gates |
| Scope | 13 tabs and about 110 features is a very large solo project | Each milestone is useful alone. Stop whenever it is enough |
| Testing limits | A VM has no real fans, battery or useful sensors | D6 |
| Copying too closely | Legal and reputational exposure if released | Own name, artwork and wording. Get legal advice before a public release |

## 8. Testing

- **Pure logic:** unit tests with recorded process trees and sensor fixtures.
- **Accuracy:** compare with `ps`, `top`, `lsof`, `powermetrics` and Activity Monitor, within stated margins.
- **Long runs:** a multi-day soak for CPU, memory and history growth, and sleep and wake behaviour.
- **Safety:** tests that try to get the CLI and MCP to quit protected things, or to skip approval.
- **Hardware matrix:** only your Mac at first. Other chips are unverified, and the app should say so.
- **UI:** the Tart VM setup from `apps/meter-gpui` works for layout. It cannot prove hardware readings.

## 9. Decisions needed

| # | Decision | My recommendation |
|---|---|---|
| D1 | Stack | Decide after S1. My starting guess is Swift with AppKit and SwiftUI, because this list leans on native menu bar, tables, text input, accessibility and Shortcuts. GPUI needs to beat that on measured idle cost to win |
| D2 | Pramana covers the same reference, as an older spec with no code | Details replaces it. `git mv` Pramana to `archive/` |
| D3 | Meter overlaps LIM features | Keep Meter as it is for now. Copy its readers into Details at M5, as repo rules prefer copying to sharing. Reconsider Meter then |
| D4 | Plan limits: reuse stored sign-ins to ask providers, or stay local? Meter promises that no provider is called | Start local, showing what logs reveal. Treat live calls as a separate opt-in, one provider at a time, after checking each provider's terms |
| D5 | Personal tool or public release? This decides signing, licensing and how closely to follow the reference | Personal first. Developer ID signing may still be needed for the helper |
| D6 | Testing on this Mac: Meter's "do not affect the host" rule doesn't fit here, since real readings need real hardware | Allow read-only runs on the host. Treat quit, fan, volume, helper and login item as separately approved steps |
| D7 | This repo is public, so committing makes this plan public. Screenshots are git-ignored already | Your call. Consider a private repo for this app |
| D8 | Minimum system | Apple silicon and the current macOS first. Drop Intel unless you need it. The reference supports macOS 15 and Intel |
