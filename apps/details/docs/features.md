# Details: feature catalog

Source: vitalsmac.com, read 1 November 2026 (Vitals 1.9.2), including its feature pages, changelog, FAQ and roadmap. Behaviour and numbers below are what that site states. They have not been checked against the real app, and nobody on this project has run it.

- **M** is the milestone in [product-plan.md](product-plan.md).
- **Risk** is my judgement of how hard the feature is to build on macOS. `H` means it depends on undocumented or restricted system access and needs a spike. These ratings are unverified.

## 1. Apps, not processes

| ID | Feature | Behaviour | M | Risk |
|---|---|---|---|---|
| APP-1 | Group by app | Each helper counts under the app that started it. About 60 rows instead of about 1,000. One figure each for memory, CPU, energy, disk and network | 1 | M |
| APP-2 | Attribution rules | Attribute by who started the process. A language server counts under VS Code. A terminal command counts under the terminal, or under its project on the Projects tab | 1 | M |
| APP-3 | Unfold an app | Clicking a row shows its processes with their own numbers. Charts move aside and are not covered | 1 | L |
| APP-4 | Sort and pin | The busiest app is first on every tab. Pin watched apps to the top | 1 | L |
| APP-5 | Find | ⌘F by name, port number, or `/regex/` | 1 | L |
| APP-6 | Columns | A Columns menu per tab: PID, user, path, version, start time, idle wake-ups, readings. Each tab keeps its own choice | 3 | L |
| APP-7 | Split inside browsers | Chrome, Safari, Firefox, Edge, Brave, Arc and Dia split into tabs, extensions, graphics and the browser. Slack and VS Code split the same way. Not memory per individual tab | 3 | M |
| APP-8 | Quit | Quit an app or one process from any list. Says how many processes will close and waits for a yes. Force Quit is one more click. Can quit a single tab's process | 1 | M |
| APP-9 | Row actions | Right-click: copy name or path, turn App Nap off, Alert When…, Don't Alert About | 2 | M |
| APP-10 | System processes | Listed under macOS, or hidden with one switch. Their numbers need the helper (SYS-1) | 1 | L |

## 2. Subsystem tabs

| ID | Tab | Contents | M | Risk |
|---|---|---|---|---|
| TAB-1 | Overview | A card per subsystem with a live figure, supporting numbers and a history strip. Breakdowns: memory by type, memory by app, power by app. Agents card. Over Time, Peaks and Worth a Look are reached from here | 1 | L |
| TAB-2 | CPU | Now, average today, load, chart, user and system, P and E core counts, top app, app list. 100% can mean one core or the whole Mac | 1 | L |
| TAB-3 | Memory | In use of total, pressure label, free, swap, app, wired, compressed, top app, list. Uses process footprint. Pressure chart under the memory chart, swap beside it once used | 1 | M |
| TAB-4 | Disk | Free of total, rate chart, used, written today, read and write now, volumes, top app by writes, list | 1 | M |
| TAB-5 | Network | Download chart, upload, today, 7 and 30 days, active interface, top app, list. Totals must not jump after sleep. Bytes or bits | 1 | M |
| TAB-6 | GPU | Name, use, chart, average and peak, GPU memory, apps heaviest first. Says so when unavailable and never shows a false zero | 4 | H |
| TAB-7 | Neural Engine | Apple silicon only. Activity estimated from memory traffic against this Mac's observed peak, chart, apps holding it open. Notice after 20 busy minutes. The site's changelog shows repeated fixes for M5 and M6 | 4 | H |
| TAB-8 | Battery | Charge, state, time left, cycles, maximum capacity, temperature, power per app. Hidden with no battery. Desktop Macs still show power | 4 | M |
| TAB-9 | Sound | See SND features | 4 | H |
| TAB-10 | Bluetooth | Connected devices with battery, lowest first. AirPods left, right and case | 4 | M |
| TAB-11 | Projects | See DEV features | 3 | M |
| TAB-12 | Agents | See AGT and LIM features | 3 | M |
| TAB-13 | Containers | Appears only while an engine runs. See DOC features | 3 | M |

## 3. Menu bar and dropdown

| ID | Feature | Behaviour | M | Risk |
|---|---|---|---|---|
| BAR-1 | One item per reading | CPU, memory, GPU, Neural Engine, network, disk, temperature, fan speed, battery, AI plans. Each may be its own item, ⌘-draggable, or share one item | 2 | M |
| BAR-2 | Item style | Value, graph or both. With or without icon and caption. Compact size, framed graphs, a colour per reading. Stable width so the bar does not jump | 2 | L |
| BAR-3 | Item options | Memory: pressure, used or free. Network: upload on or off. Temperature follows CPU, GPU or hottest sensor. Battery: charge or time left, with a bolt while charging | 2 | L |
| BAR-4 | Red under strain | CPU or GPU at 90%, memory under pressure, disk nearly full, Mac hot. Battery red at 20% on battery. Clears by itself | 2 | L |
| BAR-5 | Dropdown | Opens on Overview, as tiles or a one-line list. Icon strip of tabs, uptime, busiest apps, Open, Settings, Quit. Mirrors window totals | 1 | M |
| BAR-6 | Dropdown tab pages | A compact page per tab, including Sound sliders, Projects with Stop, Containers with Stop, and Agents | 2 | M |
| BAR-7 | Strain line | While a reading is red, the top line names the reading, the limit and how long. Clicking opens that tab | 2 | L |
| BAR-8 | Dropdown actions | Hovering an app shows a close button. Clicking an app opens the window on it. Right-clicking the item gives Settings and Quit | 2 | L |
| BAR-9 | Keyboard | Tab between pages, ⌘1 to ⌘0, Esc closes. Global shortcuts for dropdown and window | 2 | L |
| BAR-10 | Dock | Optional. Opened by hand shows the window; at login stays in the menu bar. Closing the window keeps sampling | 1 | L |

## 4. History

| ID | Feature | Behaviour | M | Risk |
|---|---|---|---|---|
| HIS-1 | Local store | One SQLite file. Retain 30 days (default), 90 days, 6 months or 1 year. Documented tables. Delete History… | 2 | M |
| HIS-2 | Over Time | CPU, memory, energy, disk and network for the Mac and each app. Ranges 1 h, 12 h, 24 h, 7 d, 30 d, 90 d, 6 mo, 1 y. Weekly bars past 90 days. Shows when recording began | 2 | M |
| HIS-3 | Hourly ranking | Top apps hour by hour for the last two days; per-app daily rollups further back | 2 | M |
| HIS-4 | Peaks | Highest CPU, memory pressure, temperature and power in the range, and when, with apps beside each. Built from a reading every 5 minutes | 2 | M |
| HIS-5 | At a Limit | Stretches at a limit: CPU or GPU at 90%, memory pressure, set temperature, thermal slowdown | 2 | M |
| HIS-6 | CSV export | Three files: apps by day, Mac by hour, 5-minute readings. From Over Time, toolbar Export, File menu and Settings | 2 | L |

## 5. Alerts and Worth a Look

| ID | Feature | Behaviour | M | Risk |
|---|---|---|---|---|
| ALR-1 | Pattern alerts | CPU busy for 10 minutes (example: 70%), memory growing for an hour, disk or network burst far above usual, Mac hot for minutes. Names the app and what it did | 2 | M |
| ALR-2 | Repeat suppression | An app named once is left alone for an hour. Details never quits an app because of an alert | 2 | L |
| ALR-3 | Worth a Look | Notices beside Overview, by day. Repeats collapse with a count. Each row shows the current state. Filter by type and pin | 2 | M |
| ALR-4 | Thresholds | Settings for how much and how long, and one master switch for notifications | 2 | L |
| ALR-5 | Per-app limit | Tell Me When…: memory or CPU limit for one app for chosen minutes. Right-click starts slightly above current use | 2 | L |
| ALR-6 | Quiet an app | Until tomorrow or always. Listed in Settings with Tell Me Again | 2 | L |
| ALR-7 | Heat view | When hot: apps behind it, highest first, Quit beside each, fan speeds | 4 | M |

## 6. Process Inspector

| ID | Feature | Behaviour | M | Risk |
|---|---|---|---|---|
| INS-1 | Drawer | Clicking a process opens a side drawer. An ⓘ appears on hover in an app's process list. Parent, launcher and children are links | 3 | L |
| INS-2 | Identity | PID, owning app, process kind (tab, extension, GPU, dev server), parent, launched by, running for | 3 | M |
| INS-3 | Security | Signer, Team ID, hardened runtime, sandbox, every entitlement, plain states such as "Not signed" | 3 | M |
| INS-4 | Launch | launchd job behind it, executable path, full arguments, environment folded. Architecture and Rosetta state | 3 | M |
| INS-5 | Lifetime use | CPU time split user and system, memory against lifetime peak, threads, open files, page-ins, bytes read and written, ports, children, live chart while open | 3 | M |
| INS-6 | App card | Version, bundle ID, architecture, signature, protection, what starts it, location, full Info.plist | 3 | L |
| INS-7 | New and ended | New mark on just-started processes. Ended Recently keeps exits for 10 minutes with run time and memory | 3 | M |
| INS-8 | Export | Copy as Text, Save as CSV, including Info.plist, entitlements, environment and processes | 3 | L |
| INS-9 | Honest gaps | Other users' command lines, environments and totals are unavailable; the drawer says so | 3 | L |

## 7. Dev servers and ports

| ID | Feature | Behaviour | M | Risk |
|---|---|---|---|---|
| DEV-1 | Group by project | Everything started from a project folder is one row named after the folder, with total memory and CPU | 3 | M |
| DEV-2 | Ports | Each listening port beside its process and project. Whose it is. Open to the network or this Mac only. Found by ⌘F. macOS-assigned helper ports are not shown as dev servers | 3 | M |
| DEV-3 | Idle detection | Working, or idle for how long. A banner gives the count, the memory and ports freed, and Stop All… | 3 | M |
| DEV-4 | Stop | Stop one server, Force Quit one process, or quit a project, each confirmed with what closes and which ports free | 3 | M |

## 8. Containers

| ID | Feature | Behaviour | M | Risk |
|---|---|---|---|---|
| DOC-1 | Container list | Memory against limit, CPU, network, published ports, image, Compose project and service | 3 | M |
| DOC-2 | Engines | Docker Desktop, OrbStack, Colima, Lighter, Rancher Desktop, Podman, and any registered Docker context. Works without the `docker` command installed | 3 | M |
| DOC-3 | Stop only | Confirmed, 10 seconds then killed. Never starts, restarts or removes | 3 | L |
| DOC-4 | Project footprint | Compose containers under their project, measured inside the VM so not counted twice. Quitting the project stops them | 3 | M |

## 9. Coding agents

| ID | Feature | Behaviour | M | Risk |
|---|---|---|---|---|
| AGT-1 | Session list | Claude Code, Codex, Gemini CLI, Aider, OpenCode, Cursor, Copilot, Amp, Kiro and Goose by name, recognised by command, including inside editors | 3 | M |
| AGT-2 | State | Working, waiting for you or idle, and for how long. Project and run time. How the site infers this is not stated, so it needs a spike | 3 | H |
| AGT-3 | What it started | Dev servers, browsers, tests and tool servers under the session, each with memory and CPU. Finds a closed session's leftovers | 3 | M |
| AGT-4 | Quit Session… | Lists what will close, then quits all of it. Force Quit available | 3 | M |
| AGT-5 | No content access | Reads the process, folder and children, not code or conversations | 3 | L |

## 10. AI plan limits and tokens

Off until turned on.

| ID | Feature | Behaviour | M | Risk |
|---|---|---|---|---|
| LIM-1 | Plan rings | Claude, Codex, Copilot, Cursor, Antigravity, Grok, Ollama, OpenCode. A ring per plan, in a colour per provider, week first | 5 | H |
| LIM-2 | Limit windows | 5-hour, week, month, per-model, each with reset to the minute | 5 | H |
| LIM-3 | Pace | "On pace, 10% to spare" or "runs out in 4 days at this rate", red if it runs out first | 5 | L |
| LIM-4 | Accounts | Every signed-in account and Claude Code profile. A tool with no sign-in is left out | 5 | H |
| LIM-5 | Tokens | Claude Code and Codex today and this week, from local logs: sent, received, cached, total, by model, weekly chart | 5 | L |
| LIM-6 | Menu bar plans | A number after each provider's mark, optional 5-hour value, red when nearly used | 5 | L |

The site says it reuses each tool's stored sign-in to ask the provider. That handles credentials and may be restricted by provider terms. See decision D4.

## 11. Assistants (MCP)

Off until turned on.

| ID | Feature | Behaviour | M | Risk |
|---|---|---|---|---|
| MCP-1 | Read tools | System, apps, a number's owner, ports, projects, containers, agents, limits, history, `process_details`. Every answer is timestamped | 5 | M |
| MCP-2 | One-click add | Add buttons for Claude, Claude Code, Codex and Cursor, plus a command for anything else. Must preserve the assistant's existing tools | 5 | M |
| MCP-3 | Visibility switches | System readings, apps, projects, history. Project folders by name only. Environment is never sent | 5 | L |
| MCP-4 | Quit approval | A separate window: requester, target, memory and ports freed, what is connected, safety verdict. No answer in a minute means no. A no is remembered | 5 | M |
| MCP-5 | Self-refusal | Always refuses macOS parts, anything that would respawn at once, and itself | 5 | M |
| MCP-6 | Trusted folders | "Safe" requests may proceed in chosen folders. Safe means nothing of yours is lost. Force Quit always asks | 5 | M |
| MCP-7 | Audit log | Requester, target, outcome (quit, declined, unanswered, refused). Show All for the month; Clear | 5 | L |
| MCP-8 | Remote use | Runs over SSH via the command's `mcp` subcommand | 5 | L |

## 12. Command line

| ID | Feature | Behaviour | M | Risk |
|---|---|---|---|---|
| CLI-1 | Read commands | `status`, `top`, `app`, `find`, `process`, `ports`, `projects`, `containers`, `agents`, `limits`, `awake`, `battery`, `history`, `whats-wrong` | 3 | M |
| CLI-2 | Output | Table, `--json`, `--csv`, `--watch` every 2 seconds. Nothing clipped in a pipe. Exit 1 when the app does not answer. JSON fields stay stable | 3 | L |
| CLI-3 | Time travel | `top cpu day`, `top --at 14:02`, `top --from … --to …` | 3 | L |
| CLI-4 | Stopping | `can-quit` gives safe, ask first or leave alone, with the reason. `quit` and `stop` (`--port`, `--idle`) need a yes. Needs-a-person actions are refused without a terminal, even with `--yes` | 3 | M |
| CLI-5 | Install | Linked into a Homebrew or local bin directory, man page, zsh, bash and fish completion. Answers from the running app and measures nothing itself | 3 | L |
| CLI-6 | URL scheme | Open a tab or toggle the dropdown from scripts | 6 | L |

## 13. Sensors, fans and power

| ID | Feature | Behaviour | M | Risk |
|---|---|---|---|---|
| SYS-1 | Optional helper | Reads other users' and system processes. Answers one question, takes no commands, quits with the app. The app works without it | 4 | H |
| SEN-1 | Temperatures | CPU, GPU and other sensors, Celsius or Fahrenheit, kept in history. The site's changelog shows per-chip tuning | 4 | H |
| SEN-2 | Fan speeds | Every fan's rpm | 4 | H |
| SEN-3 | Fan control | Auto or a set speed within the Mac's limits. Survives restarts and sleep. Returns to auto when the app quits. Needs the helper | 4 | H |
| PWR-1 | Power per app | Watts per app with helpers counted, estimated on Intel. Overview breakdown | 4 | H |
| PWR-2 | Keeping Mac awake | Each app holding sleep off, its reason, how long, whether the screen stays on, Quit | 4 | M |
| PWR-3 | Idle wake-ups | Sortable column; App Nap off per app | 4 | M |

## 14. Sound

| ID | Feature | Behaviour | M | Risk |
|---|---|---|---|---|
| SND-1 | Per-app volume | A slider for each app with sound open, playing first, mute, applied while dragging, remembered. No driver, nothing recorded | 4 | H |
| SND-2 | Output strip | Device picker and level | 4 | L |
| SND-3 | Microphone | Device, level, and the apps using it now | 4 | M |

## 15. Personalisation

| ID | Feature | Behaviour | M | Risk |
|---|---|---|---|---|
| CUS-1 | Palette | Default, monochrome, colour-blind set, per-reading colours | 6 | L |
| CUS-2 | Number type | Rounded, San Francisco, SF Mono, New York or any installed font, light to bold | 6 | L |
| CUS-3 | Density and range | Comfortable or compact, charts of 5, 15 or 30 minutes, light or dark. Live preview in Settings | 6 | L |
| CUS-4 | Arrange window | Choose and order tabs. Show, hide or reorder sections, cards and lists. Reset | 6 | M |
| CUS-5 | Arrange dropdown | Its own tiles, and tiles or list layout | 6 | M |
| CUS-6 | Units | Celsius or Fahrenheit, bytes or bits, per core or per Mac, with a live example | 1 | L |

## 16. Export, automation and platform

| ID | Feature | Behaviour | M | Risk |
|---|---|---|---|---|
| EXP-1 | Image export | Copy or save a 1200 × 630 card or a dashboard image. Background: window, light, dark or transparent. Name on or off | 6 | L |
| AUT-1 | Shortcuts actions | Get a reading or the busiest app | 6 | M |
| PLT-1 | Open at login | Off until asked | 2 | L |
| PLT-2 | Languages | The site lists 13. Plan: English first, built so others can be added | 6 | L |
| PLT-3 | Accessibility | Not described on the site. Added here: VoiceOver labels, keyboard use, contrast, and no meaning carried by colour alone | 1 onward | M |

## 17. After version 1: the site's public roadmap

None of this is in Vitals yet, according to the site. Build only what you still want after version 1.

**Next, in the site's order:** storage map; safe cleanup, one item at a time; drive health; per-drive activity; a one-line Mac Health summary; a watchdog for runaway dev processes.

**Later:** USB devices and link speed; desktop widgets; a "what is holding memory" view; background tasks by app; trends in Worth a Look; battery use since unplugging; fan curves; free-disk limit alert; webhook alerts; internet speed test; IP, Wi-Fi details and connection-drop alert; weekly reports; record a slow moment; Prometheus exporter; Docker disk cleanup; OrbStack machines; a shortcut to quit the heaviest app; notch display.

## 18. Deliberately left out

**Not needed for a personal app, unless D5 changes:** licensing, trial, seats, pricing, cash-back and review programs, the marketing site, and in-app updates.

**The site also rules these out, for reasons that apply here too:**
- Watching other Macs from one window. Use the command or MCP over SSH instead.
- A chat inside the app. The assistant you already use can ask through MCP.
- A battery charge limiter. macOS has its own, and a wrong one strands a laptop.
- Memory per browser tab. Browsers do not expose it reliably.
- A "Free up RAM" button. It empties a cache that soon refills.

**Also out:** sampling and spindumps, Windows, Linux, iPhone and iPad.
