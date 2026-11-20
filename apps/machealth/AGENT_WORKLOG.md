# MacHealth — Agent Worklog

## 2026-11-20 — Layout Fix (Taffy/GPUI `min_w_0` Text Offset) & macOS HIG UI/UX Polish

### What Was Done
- **Root Cause & Fix for Overlapping Text in `Home & Clean Up` (and Table Rows)**:
  - Diagnosed why `Safe Clean Up` candidate names/paths (`mise`, `master-ev.mp4`, `com.google.antigravity`, `com.spotify.client`, etc.) were displaced hundreds of pixels upward over the `Home — Files by Kind` card:
    - In GPUI 0.2.2 (`taffy 0.9`), placing `.min_w_0()` on `card()` and on inner `div().flex().flex_col().min_w_0()` text columns inside horizontal `.items_center()` rows caused Taffy's intrinsic `min_content` pass to measure long file paths at `width = 0` (1 character per line = ~500–880px tall), and `.items_center()` vertically centered that tall bounding box at a large negative Y offset (`(56 - 800) / 2 = -372px`).
  - Removed `.min_w_0()` from `card()` and all inner text columns across `cleanup.rs`, `activity.rs`, `health.rs`, `agents.rs`, `security.rs`, and `menubar.rs`, replacing raw full paths with single-line tilde-shortened paths (`shorten_home_path`, e.g., `~/Library/Caches/...`) and `.overflow_hidden()` + fixed-width right-aligned metric columns.
- **macOS Human Interface Guidelines (HIG) UI/UX Improvements**:
  - **macOS System Settings → Storage Multi-Segment Bar** (`views/cleanup.rs`): Replaced the 8 stacked blue bars in *Home — Files by Kind* with a unified multi-color proportional storage bar + a 2-column category legend grid with color dots (`Videos`, `Images`, `Archives`, `Documents`, `Apps & Installers`, `Developer & Code`, `Audio`, `Other`).
  - **Activity Monitor Column Header & Tabular Alignment** (`views/activity.rs`): Added column headers (`Application`, `Impact`, active tab metric, `Memory`, `Actions`) with right-aligned fixed-width numeric columns and inset helper drawer cards.
  - **Refined macOS Sequoia Palette & Sidebar/Toolbar Hierarchy** (`theme.rs`, `components.rs`, `views/mod.rs`): Crisp accent-filled active sidebar item (`#0A84FF` / `#007AFF` with white text), unified 52px toolbar, subtle row hover highlight, and display-proportional initial window sizing (`main.rs`).

### Current Build / Test Status
- `cargo test --workspace` (in `apps/machealth/`): **52 passed, 0 failed, 0 warnings**.
- `cargo run --bin machealth -- --smoke-test`: **Passed (exit code 0)** with zero host writes/registrations.

---

## 2026-11-20 — Phases 4 & 5: GPUI Desktop UI, Menu Bar Panel, Global Hotkey & Host-Isolated Verification

### What Was Done
- **Architecture Refinement (Pure Rust GPUI 0.2.2 vs NSH JS Engine)**:
  - Inspected `nsh-core` / `nsh-platform-gpui` (0.6.1) on crates.io and found that NSH is a JS/QuickJS shadow-tree bridge rather than a pure-Rust view DSL.
  - Built `machealth-ui` and `machealth-app` directly on pure-Rust **`gpui` 0.2.2** (`features = ["font-kit", "runtime_shaders"]`, matching `apps/meter-gpui`) + `objc2-app-kit` 0.3 + Carbon `RegisterEventHotKey`, keeping zero JS runtime overhead and compiling without requiring a separate Metal SDK toolchain.
- **Phase 4 — Desktop UI (`crates/machealth-ui/`)**:
  - `theme.rs` — Automatic macOS Light & Dark semantic tokens (`Theme::for_appearance`) and status badge palettes (`Severity`, `HealthGrade`, `ImpactLabel`, `AuditStatus`).
  - `format.rs` — Tabular human formatting (`format_bytes_si`, `format_memory_gib`, `format_rate`, `format_duration_short`).
  - `state.rs` — Reactive `AppState` bridging all `machealth-core` subsystems with background-safe `collect_fast_refresh()` / `collect_slow_refresh()` payloads and `RuntimeConfig` host-isolation controls (`ephemeral`, custom `data_dir`, `allow_external_actions`).
  - `components.rs` — Reusable native cards, badges, progress bars, buttons, segmented controls, and CPU/Memory history bar charts (`history_bars`).
  - `views/` — All 5 sidebar views + modals + menu bar popover:
    - `health.rs`: 8-Area Health Score hero + expandable Findings list with 1-click Fix/Inspect buttons, 8-area score grid, 6 Live Vitals cards (Processor P/E cores, Heat & Battery, Memory breakdown + Swap + Pressure, Storage Finder-accurate + SMART, Network Down/Up, Sleep assertions & timers), Top 5 "Using the Most" switcher, and Hour/Day/Week/Month History charts.
    - `activity.rs`: 5-tab Activity manager (`CPU`, `Memory`, `Energy`, `Disk`, `Network`), expandable parent+helper app rows, Light/Moderate/Heavy badges, plain-English descriptions, and safe **Quit** / **Force** buttons.
    - `agents.rs`: AI Agent Monitor with session/process/CPU/Memory totals, expandable child-process trees, and safe **End Session** action.
    - `security.rs`: 6 Security & Maintenance audit cards with 1-click `x-apple.systempreferences:...` deep links and user `LaunchAgents` list.
    - `cleanup.rs`: Home (Files by Kind) 8-category breakdown bars + Safe Clean Up selectable checklist + Trash summary + **Move Selected to Trash…** flow.
    - `modals.rs`: Mandatory confirmation modals for Quit / Force Quit, End AI Agent Session, Balance the Load preview (with optional idle-app quit checkboxes), Balance Before & After delta summary (with instant Undo), and Move to Trash.
    - `menubar.rs`: Compact `MenuBarPopoverView` with score badge, live CPU/Memory/Heat strip, Top 3 busy apps, Balance/Undo trigger, and live keyboard-driven Spotlight (`mdfind`) Quick Search with click-to-open and Reveal in Finder.
- **Phase 5 — macOS Host, `NSStatusItem`, `Control-Option-Space` Hotkey & Host Isolation (`crates/machealth-app/`)**:
  - `menubar.rs` — Process-scoped AppKit `NSStatusItem` (`StatusBarController`) displaying live SF Symbol + Health Score and toggling the Quick Panel on click; explicitly removes the status item on `Drop`.
  - `hotkey.rs` — Process-scoped Carbon `RegisterEventHotKey` (`GlobalHotKey`) for `Control-Option-Space` (keyCode `49`, `controlKey | optionKey`): requires **zero Accessibility/TCC permissions** and unregisters cleanly in `Drop`.
  - `platform.rs` — Host-isolation CLI & environment flags (`--smoke-test`, `--ephemeral`, `MACHEALTH_EPHEMERAL=1`, `MACHEALTH_DATA_DIR`, `--no-menubar`, `--no-hotkey`, `--open-popover`). Never touches `SMAppService`, `~/Library/LaunchAgents`, `UserDefaults`, Keychain, or LaunchServices.
  - `main.rs` — GPUI application bootstrap with 2s background fast-refresh loop, 5m slow-audit loop, app menus, keybindings (`Cmd+Q`, `Cmd+W`, `Cmd+R`, `Cmd+K`), and headless `--smoke-test` runner.

### Current Build / Test Status
- `cargo test --workspace` (in `apps/machealth/`): **52 passed, 0 failed, 0 warnings** (`48` in `machealth-core`, `2` in `machealth-ui`, `2` in `machealth-app`).
- `cargo run --bin machealth -- --smoke-test`: **Passed (exit code 0)** in ephemeral in-memory mode with zero host writes or registrations.

### Next Steps
- All 5 planned phases are complete and verified on live macOS arm64. Future enhancements can include optional `.app` bundle packaging script (on demand only) or custom history retention settings.

---

## 2026-11-20 — Phase 3: Security Audits, Home/Clean Up, History Store & Quick Search

### What Was Done
- **Security & Maintenance Audits** (`crates/machealth-core/src/security/mod.rs`):
  - Full 6-item unprivileged audit (`run_security_audit`): FileVault, Firewall, Gatekeeper, Time Machine (including `tmutil latestbackup` when configured), macOS Software Updates (instant cached plist read), and Login/Startup Items (`~/Library/LaunchAgents`).
  - Each `SecurityCheckItem` carries `AuditStatus` (`Pass` / `Warn` / `Critical` / `Unknown`), plain-English summary/detail, and exact `x-apple.systempreferences:...` deep link, plus guarded `open_system_settings()`.
- **Home (Files by Kind) & Safe Clean Up** (`crates/machealth-core/src/cleanup/mod.rs`):
  - `FileKind` classifier across all 8 kinds (`Documents`, `Images`, `Videos`, `Audio`, `Archives`, `Apps & Installers`, `Developer & Code`, `Other`) with bounded traversal (`scan_home_files_default` / `scan_files_by_kind`), `.app` bundle folding, file counts, total bytes, and top-5 largest files per bucket.
  - Safe Clean Up scanner (`scan_cleanup_default` / `scan_cleanup_roots`) detecting old `.dmg`/`.pkg` installers in `~/Downloads`, forgotten large downloads (>30 days, >=25 MB), rebuildable third-party `~/Library/Caches` (strictly skipping `com.apple.*`, `CloudKit`, `GeoServices`, `PassKit`, etc.), Xcode `DerivedData`, and informational `~/.Trash` size summary.
  - Mandatory `build_cleanup_confirmation()` payload, path guardrails (`validate_trash_target`), and recoverable `move_items_to_trash()` using `Foundation` `NSFileManager.trashItemAtURL:resultingItemURL:error:` (with collision-safe `~/.Trash` fallback — never permanent deletion).
- **Local Persistent History Store** (`crates/machealth-core/src/history/mod.rs`):
  - Append-only local JSONL store (`~/Library/Application Support/MacHealth/history.jsonl`) with 30-day automatic compaction (`prune_older_than_retention`).
  - Queries for `Hour`, `Day`, `Week`, and `Month` (`HistoryRange`) with ~60-point chart downsampling and plain-English summary generation (*"Processor X% on average, briefly up to Y%. Memory averaged Z%. Busiest: ..."*).
- **Spotlight Quick Search** (`crates/machealth-core/src/search/mod.rs`):
  - Native `/usr/bin/mdfind` search (`quick_search`) with query literal sanitization, filename/content modes, optional directory/kind filters, relevance ranking, and `/usr/bin/open` (`open_file`) + `/usr/bin/open -R` (`reveal_in_finder`).
- Re-exported all Phase 3 APIs from `crates/machealth-core/src/lib.rs` with end-to-end live verification tests.

### Current Build / Test Status
- `cargo test -p machealth-core -- --nocapture` (run in `apps/machealth/`): **48 passed, 0 failed, 0 compiler warnings**.
- Verified live on macOS arm64:
  - Security audit: 5 passed, 1 attention (Time Machine not configured), all 6 deep links verified.
  - Clean Up scan: found 17 safe-to-review items (`7.28 GB`) while skipping protected Apple caches.
  - Spotlight Quick Search: live `mdfind` returned `PRODUCT_DESIGN.md` cleanly.
  - History JSONL append, query, downsampling, and 30-day pruning verified in isolated temp store.

### Next Steps
1. **Phase 4**: Build `crates/machealth-ui` (NSH `nsh-core`/`nsh-base` or GPUI-native views matching `PRODUCT_DESIGN.md`) and `crates/machealth-app` (GPUI desktop host) with all 5 sidebar views (Health, Activity, Agents, Security, Home & Clean Up), live polling, and confirmation modals.
2. **Phase 5**: `NSStatusItem` menu bar panel, `Control-Option-Space` global hotkey, System Settings deep links, and live macOS verification.

---

## 2026-11-20 — Phase 2: Process Tree, AI Agents Monitor & Load Balancer

### What Was Done
- **Process Tree & 5-Tab Activity Engine** (`crates/machealth-core/src/processes/`):
  - `collector.rs` — Hybrid native collector combining `ps` (`pid`, `ppid`, `uid`, `%cpu`, `rss`, `etime`, `args`), macOS `libproc` (`proc_pidpath`, `proc_pid_rusage(RUSAGE_INFO_V4)` for Activity Monitor-accurate `ri_phys_footprint`, disk I/O read/write bytes, CPU time ns via `mach_timebase_info`, and idle/interrupt wakeups), optional unprivileged `nettop -P -L 1 -J bytes_in,bytes_out` per-process network bytes, and frontmost app detection via `lsappinfo`.
  - `grouping.rs` — Outermost `.app` bundle + ancestor-chain roll-up (`group_processes`) collapsing Chrome/Brave/Electron/Safari WebKit helpers and spawned child helpers under their parent application, plus 5-tab sorting (`ActivityTab::Cpu | Memory | Energy | Disk | Network`) and `top_groups()` for Health Dashboard "Using the Most".
  - `descriptions.rs` — Plain-English `ImpactLabel` (`Light` / `Moderate` / `Heavy`) and human-readable descriptions for common macOS services, browsers, IDEs, and AI tools.
  - `actions.rs` — Confirmation payload builder (`build_quit_confirmation`), strict `check_process_safety` guardrails (refuses PID 0/1, MacHealth itself, root/other-user PIDs, protected macOS system/UI services like `WindowServer`, `Dock`, `Finder`, `launchd`, `/System/Library/CoreServices/*`, etc.), and safe `quit_process` / `quit_app_group` (`SIGTERM` / `SIGKILL`).
  - `mod.rs` — Stateful `ProcessSampler` tracking per-process CPU%, disk I/O rates, network throughput rates, and composite Energy Impact deltas.
- **AI Agent Monitor** (`crates/machealth-core/src/agents/mod.rs`):
  - Signature catalog covering Claude / Claude Code, ChatGPT, Cursor, Codex, Ollama, LM Studio, Antigravity, Windsurf/Aider/Goose/OpenCode/Cline/Copilot, MCP servers, and local Python/Node AI workers.
  - Full recursive BFS child-process tree attribution (`detect_ai_agents`) avoiding double-counting when an agent spawns MCP servers or sub-agents, rolling up combined CPU, Memory, Energy, Disk, and Network totals.
  - Safe `build_end_session_confirmation` and `end_agent_session` (terminates descendants before root under guardrails).
- **Balance the Load + Before/After + Undo** (`crates/machealth-core/src/balancer/mod.rs`):
  - `scan_balance_candidates` detects heavy background CPU hogs (`taskpolicy -b`) and long-idle memory-heavy GUI apps (optional quit), automatically excluding the frontmost app, protected system processes, MacHealth itself, and already-throttled PIDs.
  - `apply_balance_plan` executes `/usr/sbin/taskpolicy -b -p <pid>` via `TaskPolicyExecutor` (with mockable trait for unit tests), captures before/after `MetricsReading` deltas, and atomically persists an `UndoLedger` (`~/Library/Application Support/MacHealth/balance_undo.json`).
  - `undo_balance` restores throttled processes back to normal scheduling via `/usr/sbin/taskpolicy -B -p <pid>` (gracefully handling already-exited PIDs) and updates the persisted ledger.

### Current Build / Test Status
- `cargo test -p machealth-core -- --nocapture` (run in `apps/machealth/`): **38 passed, 0 failed, 0 compiler warnings**.
- Verified live on macOS arm64:
  - 837 processes rolled into 591 app groups (e.g., Safari 28 procs, T3 Code 31 procs, Brave Browser 14 procs).
  - Live AI Agent detection found 5 active sessions (Claude, Antigravity, Codex) with full child process tree attribution (30 total processes).
  - Live `/usr/sbin/taskpolicy -b` and `-B` round-trip verified on a spawned child process without admin privileges, plus live `SIGTERM` graceful quit test.

### Next Steps
1. **Phase 3**: Security & maintenance auditors (with full System Settings deep links), Home (files by kind) & safe Clean Up scanner (Trash-based), local History store (Hour/Day/Week/Month), and Spotlight Quick Search (`mdfind`).
2. **Phase 4**: NSH (`machealth-ui`) + GPUI (`machealth-app`) desktop views.
3. **Phase 5**: `NSStatusItem` menu bar panel, `Control-Option-Space` hotkey, and live macOS verification.

---

## 2026-11-20 — Phase 1: Foundation, Telemetry Core & 8-Area Health Score Engine

### What Was Done
- Initialized standalone Cargo workspace inside `apps/machealth/` (`Cargo.toml`, `Cargo.lock`, edition 2024) with `crates/machealth-core`.
- Implemented unprivileged native macOS telemetry collectors in `crates/machealth-core/src/telemetry/`:
  - `cpu.rs` — Mach `host_processor_info` (`PROCESSOR_CPU_LOAD_INFO`) per-core tick delta sampler (`CpuSampler`) + `sysctlbyname` Apple Silicon P/E core topology (`hw.nperflevels`, `hw.perflevel*.name/logicalcpu`), plain-English load levels (`Light` / `Moderate` / `Heavy`).
  - `memory.rs` — Mach `host_statistics64` (`HOST_VM_INFO64` rev1 ABI) Activity Monitor memory breakdown (App, Wired, Compressed, Cached, Free), `vm.swapusage` (`xsw_usage`), and `kern.memorystatus_vm_pressure_level` (`Normal` / `Tight` / `Critical`).
  - `thermal.rs` — `Foundation` `NSProcessInfo.processInfo.thermalState` via Objective-C FFI + `pmset -g therm` throttling parser.
  - `storage.rs` — Finder-accurate available capacity via `NSURL` (`NSURLVolumeAvailableCapacityForImportantUsageKey` / `NSURLVolumeTotalCapacityKey`), `libc::statfs("/")` raw free & purgeable delta, and `diskutil info /` APFS/SMART parser.
  - `network.rs` — BSD `libc::getifaddrs` (`AF_LINK` `if_data`) delta throughput sampler (`NetworkSampler`) for active non-loopback interfaces.
  - `battery.rs` — `ioreg -r -c AppleSmartBattery` parser supporting both Apple Silicon (`NominalChargeCapacity / DesignCapacity` health %) and desktop Macs (wall-power state).
  - `sleep.rs` — `pmset -g assertions` active sleep-blocker parser (with `HH:MM:SS` duration, reason, `Created for PID`) and `pmset -g custom` Battery/AC timer parser.
  - `health_inputs.rs` — Non-privileged collectors for Stability (`~/Library/Logs/DiagnosticReports` 7-day crash scan), Startup Items (`~/Library/LaunchAgents` plist scan), and Security/Updates/Backups summary (`fdesetup`, `socketfilterfw`, `spctl`, `tmutil`, cached `com.apple.SoftwareUpdate` preferences).
  - `mod.rs` — `SystemSnapshot` and stateful `TelemetryCollector`.
- Implemented the 8-Area Health Score Engine in `crates/machealth-core/src/scoring/mod.rs`:
  - Weighted 0–100 score across all 8 areas (Processor & Heat 15%, Memory & Swap 15%, Storage & Drive Wear 15%, Battery 10%, Sleep & Wake 10%, Stability 10%, Startup Items 10%, Security/Updates/Backups 15%).
  - Plain-English verdict generation (`Great` / `Good` / `Fair` / `Needs Attention`, capping grade when critical findings are present) and prioritized actionable `HealthFinding` items with `FindingAction` targets (System Settings deep links, Balance Load, Clean Up, Activity, Sleep inspection).
- Updated `apps/machealth/README.md` with build/test instructions and workspace layout.

### Current Build / Test Status
- `cargo test -p machealth-core -- --nocapture` (run in `apps/machealth/`): **24 passed, 0 failed, 0 compiler warnings**.
- Verified live macOS arm64 readings end-to-end: CPU P/E breakdown, 16 GiB memory + swap + pressure, 494.4 GB APFS storage (Finder available space + SMART Verified), battery health/cycles, active `pmset` sleep assertions, 7-day crash count, LaunchAgents, and security/Time Machine status.

### Next Steps
1. **Phase 2**: Process tree & parent+helper grouping (`processes/`), 5-tab metrics (CPU/Memory/Energy/Disk/Network), safe Quit/Force Quit guardrails, AI Agent monitor (`agents/`), and Balance the Load (`taskpolicy -b/-B`) with before/after capture and Undo ledger (`balancer/`).
2. **Phase 3**: Expanded security audits, Home/Clean Up scanner (Trash-based), local History store, and Spotlight Quick Search (`mdfind`).
3. **Phase 4**: NSH (`machealth-ui`) + GPUI (`machealth-app`) desktop views.
4. **Phase 5**: `NSStatusItem` menu bar panel, `Control-Option-Space` hotkey, and live macOS verification.

---

## 2026-11-20 — App Folder, Product Design & Implementation Plan

### What Was Done
- Created standalone app folder `apps/machealth/` following the repository's incubator conventions.
- Added:
  - `apps/machealth/README.md` — App overview, stack summary, and workspace layout.
  - `apps/machealth/PRODUCT_DESIGN.md` — Full features list (modeled on Brello for Mac), product design, Rust + GPUI + NSH architecture, and 5-phase implementation plan.
  - `apps/machealth/.gitignore` — Rust/macOS stack-specific ignores.
- Registered `machealth` in the root `README.md` apps catalog table with status `wip`.

### Current Build / Test Status
- Documentation and planning phase only; no Rust crates scaffolded yet.

### Next Steps
1. **Phase 1**: Initialize Cargo workspace (`Cargo.toml`, `crates/machealth-core`) and implement macOS telemetry collectors + 8-area Health Score engine with unit tests.
2. **Phase 2**: Process tree & helper grouping, AI Agent monitor, and Balance the Load (`taskpolicy`) with Undo.
3. **Phase 3**: Security & maintenance audits, Home/Clean Up scanner, local history store, and Spotlight Quick Search.
4. **Phase 4**: NSH (`machealth-ui`) + GPUI (`machealth-app`) desktop views.
5. **Phase 5**: `NSStatusItem` menu bar panel, `Control-Option-Space` hotkey, and live macOS verification.
