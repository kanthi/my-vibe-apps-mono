# Meter agent worklog

## 2026-10-24 — plan, scaffold, reader spike

**Done**
- Planned the app (see Plan below). Scope now includes Codex and Claude Code alongside agy and Grok.
- Scaffolded a SwiftPM app (no XcodeGen installed; avoids hand-edited pbxproj): `Package.swift`, `Sources/Meter/` (`MenuBarExtra` window-style + "Meter" window), `scripts/bundle.sh` → `build/Meter.app` (LSUIElement, ad-hoc signed).
- Added `.gitignore`, empty `.env.example`.

**Status**: `swift build` + `scripts/bundle.sh` succeed. The menu-bar item is a placeholder. Not launched or visually verified. No tests yet. No readers yet.

**Reader spike findings (structure only, no prompt text read)**
- **Codex**: `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`. `event_msg`/`token_count` carries `info.total_token_usage` (input, cached_input, output, reasoning_output, total) and `rate_limits.primary` = `{used_percent, window_minutes, resets_at (epoch)}`. Best source: real quota percent + reset.
- **Claude Code**: `~/.claude/projects/<proj>/*.jsonl`. Assistant lines have `message.model`, `message.usage` (input, cache_creation, cache_read, output tokens), `cwd`, `sessionId`, `timestamp`, `isSidechain` (subagents → count under parent). No quota on disk.
- **Grok**: `~/.grok/sessions/<url-encoded cwd>/<session-id>/{events.jsonl,chat_history.jsonl}`. `events.jsonl` has `turn_started{model_id,session_relationship}`; no token/cost fields in the sampled session. TODO: check `chat_history.jsonl`, `~/.grok/dashboard`.
- **agy**: `~/.gemini/antigravity-cli/conversations/<id>.db` (SQLite: `steps`, `gen_metadata`, `trajectory_metadata_blob`) and `conversation_summaries.db` (`parent_conversation_id`, `workspace_uris`, `last_modified_time`). TODO: inspect `gen_metadata` for tokens; `cli.log` has `quota_manager.go` lines but showed "not logged in".

**Plan / next steps**
1. `UsageReader` protocol + `Session`/`Quota` model + SQLite cache in `~/Library/Application Support/Meter/`.
2. Readers: Codex (easiest, has quota), Claude Code, Grok, agy. Fixture-based tests with scrubbed samples.
3. Menu-bar label = tightest limit %, else tokens today (decision: fallback when no quota on disk). Per-tool rows, model mix, busiest project.
4. Insights (closest to cap, pace, out of pattern, idle quota) as tested pure functions.
5. Settings: launch at login (SMAppService), refresh interval, tool toggles. FSEvents + 60s timer refresh.

**Open questions**: whether Grok/agy store tokens and quota on disk; no-quota fallback chosen as "tokens today" (user did not pick explicitly).

## Standing rule (from the user, 2026-10-24)
Do not let app testing affect the host macOS until the user says the app has everything they want. Until then:
- Do not launch `Meter.app` or `open` it; do not copy it to `/Applications` or `~/Applications`.
- Do not register launch-at-login (`SMAppService`) or write to the host's login items, defaults, LaunchAgents, or Keychain in any test.
- Tests run only against scrubbed fixtures inside the repo, never against the live `~/.grok`, `~/.gemini`, `~/.codex`, `~/.claude` folders. Build with `swift build` / `swift test` only; the readers write their cache to a path injected by the test, not `~/Library/Application Support/Meter/`.
- Any live-data check must be explicit, read-only, and approved by the user first.
- Guard the launch-at-login code behind a disabled-by-default setting until the user finalizes.

## Planned tools, not yet readable (user request, 2026-10-24)
- **Grok Bot** and **Claude Duet**: the user wants both supported; neither is installed yet, and I have no data on where they keep logs, tokens, or quota. Do not guess formats. Each becomes its own reader once installed and inspected (read-only, with user approval per the standing rule).
- "etc.": the reader list is open-ended. Design: a `UsageReader` registry where each tool declares `id`, display name, `isInstalled`, and `sessions(since:)`. A tool that is not installed shows as "Not installed" (or is hidden by a setting), never as zero usage.
- Open: exact product names / install source for Grok Bot and Claude Duet.

## Alerts feature (user request, 2026-10-24)
- Spec added to `docs/product.md` ("Alerts"): threshold, exhausted/restored, pace, out-of-pattern, reader-trouble; fire-once-per-window dedupe, per-alert toggles, quiet hours, snooze.
- Reference: CodexBar's session-quota notifications (exhausted/restored), reset countdowns, and status badges. Source was a web-search summary, not the official repo (scrape failed); verify details before copying behavior.
- Design: `Notifier` protocol; real impl uses `UNUserNotificationCenter`, tests use a fake. Requesting notification permission touches the host, so it stays disabled until the user finalizes (see standing rule).
- Alert state (what already fired, per tool and window) lives in the Meter cache, injected path in tests.

## Grok Bot = Cursor (user, 2026-10-24)
- Grok Bot comes from Cursor, so it is a Cursor model/feature, not its own tool. Plan a `CursorReader`; Grok Bot usage shows as a model/feature within the Cursor row (or split out if Cursor records it separately).
- Cursor is installed here: `/Applications/Cursor.app`, `~/.cursor`, `~/Library/Application Support/Cursor`. Only existence checked; nothing inside read yet. Inspecting its data (likely SQLite `state.vscdb` and agent transcripts) needs the user's OK first, per the standing rule.
- Claude Duet is still unidentified and not installed.

## Multiple accounts per tool (user, 2026-10-24)
- Spec added to `docs/product.md` ("Accounts"): add (auto-discovered or manual label + data folder), hide/unhide (excluded from menu, percent, insights, alerts; data kept), labels only, no credential reads.
- Model change: `Tool` → many `Account` (`id`, `toolID`, `label`, `dataRoot`, `isHidden`). `Session` and `Quota` carry `accountID`; alert dedupe state is keyed by (account, window). Readers take a `dataRoot` so tests point at fixtures.
- Account list and hidden flags persist in Meter's own settings file (injected path in tests), not host defaults, until the user finalizes.
- Open: how each tool selects an account on disk (e.g. alternate home/config dirs). To check per tool, read-only and with approval.

## Claude Duet identified (user link, 2026-10-24)
Source: claude.com/resources/articles/claude-duet (fetched via WebFetch summary; firecrawl was down).
- Research preview, standalone iOS/Android app plus claude.ai; waitlist, US Pro/Max first. Runs on a cloud VM, checks connected accounts every few hours.
- Included in Pro/Max and shares Claude's usage limits; background checks count toward them.
- Page says nothing about local session logs. Likely no local records → **no local reader possible**.
- Decision: do not build a `ClaudeDuetReader`. Spec notes it counts against the shared Claude plan. Optional later: a manual "shared plan" note on the Claude account row. A quota percent would need the provider's API, which the spec rules out unless the user changes that.

## Cursor spike (read-only, user-approved, 2026-10-24)
Worked on scratchpad copies of the DBs (deleted afterwards); no prompt or reply text read, only table/key names, counts, and model names. Live files untouched.
- **Grok data is in Cursor.** Model `grok-4.6` appears in `~/.cursor/ai-tracking/ai-code-tracking.db` (`ai_code_hashes.model`, 372 rows, source `composer`) and in per-message `modelInfo.modelName` in `~/Library/Application Support/Cursor/User/globalStorage/state.vscdb`. So "Grok Bot" = Cursor composer/agent turns using a Grok model; confirmed the user's suspicion.
- Tables: `ai_code_hashes(model, conversationId, requestId, timestamp, createdAt)`, `scored_commits`, `conversation_summaries`. `state.vscdb`: `ItemTable`, `cursorDiskKV` (`bubbleId:*`, `composerData:*`, `agentKv:*`), `composerHeaders`.
- **Tokens/cost/quota NOT usable locally**: `bubbleId` has `tokenCount {inputTokens:0, outputTokens:0}` and `composerData.usageData` is `{}`. Plan is only `cursorAuth/stripeMembershipType` (not read; account-ish value, skip). No quota or reset time on disk.
- Implication: `CursorReader` reports activity (turns/conversations per model per day, per project via workspace), not tokens or percent. UI shows "turns today" instead of tokens; no percent, no pace alert for Cursor. Spec rule: blank, not invented.
- Not checked: `~/.cursor/projects/*` agent transcripts (may hold more), and how multiple Cursor accounts separate.

## Grok + agy spike (read-only, user-approved, 2026-10-24)
Scratchpad copies only (deleted); structure/keys and a few numbers, no prompt text. Note: agy `cli.log` contains the signed-in account email; the reader must never store or display it (account label is user-chosen).
- **Grok has quota on disk.** `~/.grok/logs/unified.jsonl` lines with `msg: "billing: fetched credits config"` carry `ctx.config.creditUsagePercent` (e.g. 20.0), `currentPeriod {type: USAGE_PERIOD_TYPE_WEEKLY, start, end}`, `onDemandCap/onDemandUsed`, prepaid fields. Weekly window + reset = `end`. 268 such lines; take the latest by `ts`.
- **Grok tokens** appear in the same log (`prompt_tokens`, `completion_tokens`, `reasoning_tokens`, `cached_prompt_tokens`, `tokens_per_sec`; 62 occurrences) — need to confirm the log message that holds them and how to map to session/project. Session `events.jsonl`/`chat_history.jsonl` only have `model_id`.
- `~/.grok/dashboard/workspace.db` and `memtrace/*.jsonl` not inspected (not needed so far).
- **agy**: per-conversation DBs hold `gen_metadata(idx, data blob, size)` and `executor_metadata` — protobuf blobs; `strings` shows only `model_enum`, no readable token fields. Needs a protobuf decode (unknown schema) to extract tokens. `cli.log` quota lines: "skipped (not logged in)" / "starting reload", no percent seen. **agy quota and tokens are unconfirmed.** Fallback: conversations/steps counts per day from `conversation_summaries.db` (`step_count`, `last_modified_time`, `workspace_uris`, `parent_conversation_id`).
- Updated picture: Codex = quota+tokens; Grok = quota+tokens(probable); Claude Code = tokens only; Cursor = turns by model only; agy = activity only (for now).

## Reference study (user request, 2026-10-24)
Cloned CodexBar and OpenUsage (shallow, scratchpad `ref/`, untrusted, never built or run). Write-up: `docs/reference-study.md`.
- Key finding: they get quota by calling provider APIs with the tool's stored login; spend/tokens from local logs. Grok Bot = Cursor `GetSandUsageStatus` (remote), not on disk.
- **Decision pending (user):** keep Meter strictly local (quota only for Codex and Grok; Claude Code/Cursor/agy show activity only), or add an opt-in "live quota" mode that reuses stored logins. Opt-in mode must stay off and untested against the host until the user finalizes (standing rule).
- Scratchpad clones can be deleted any time.

**Decision (user, 2026-10-24): local only for now.** Spec unchanged: no provider calls, no credential reads. Quota percent only where logged locally (Codex, Grok). Claude Code = tokens; Cursor incl. Grok Bot = turns by model; agy = activity. Live-quota mode is deferred; revisit after the app is finalized. Adopted from the study: stale tag + keep last good value, incremental per-file log cache, alert dedupe keyed by (account, window, reset cycle) with baseline-on-launch, pace verdict colors, starred menu-bar metrics, duration-based session/weekly classification.

## Full read-only data inventory (user-approved, 2026-10-24)
Method: names/sizes of folders (auth/token/credential files skipped), then schemas and numeric columns from scratchpad copies (deleted after). No prompt/reply text read, no secrets. Detail below supersedes the earlier per-tool notes where they differ.

**Codex** (`~/.codex`) — best structured source:
- `state_5.sqlite` → `threads(id, rollout_path, created_at, updated_at, source, cwd, title*, model, model_provider, reasoning_effort, tokens_used, archived, git_branch, cli_version, agent_nickname, agent_role, ...)`. `tokens_used` per thread = ready-made total; `source` JSON flags subagents (`{"subagent":...}`); `thread_spawn_edges(parent_thread_id, child_thread_id)`; `projects`/`project_roots`. (*`title`/`first_user_message` are prompt text: never read.)
- `thread_history_1.sqlite` → `thread_turns(thread_id, status, started_at, completed_at, duration_ms)` (turn counts, durations, interrupted/completed) and `thread_items(item_type...)` (counts of commandExecution, mcpToolCall, fileChange, webSearch; do not read `item_json`).
- `goals_1.sqlite` → `thread_goals(token_budget, tokens_used, time_used_seconds)` (empty here).
- `logs_2.sqlite` → `logs(ts, level, target, thread_id, ...)`; 267 rows mention rate limits; bodies are free text, avoid.
- `sessions/YYYY/MM/DD/rollout-*.jsonl` → `token_count` events with `total_token_usage`, `last_token_usage`, `model_context_window`, and `rate_limits.primary{used_percent, window_minutes, resets_at}`, plan type, credits. Source for quota.
- Models seen: gpt-6.1-sol, gpt-6-luna, codex-auto-review (guardian subagent).

**Claude Code** (`~/.claude/projects/<proj>/<session>.jsonl`, plus `subagents/` subfolders):
- Assistant lines: `message.model`, `message.usage` {input, output, cache_creation, cache_read, thinking in `output_tokens_details`}, `timestamp`, `cwd`, `sessionId`, `isSidechain`, `effort`, `entrypoint`, `gitBranch`, `requestId`, `thinkingDurationMs`. Only today's session survives on this Mac (retention cleanup), so history is short; need our own daily rollup cache to keep history. Dedupe by `requestId`/message id (streamed duplicates possible, unverified).
- Also `~/.claude/sessions/*.json`, `history.jsonl` (prompts, skip), `file-history`, `settings.json` (skip). No quota anywhere.

**Grok** (`~/.grok`):
- `logs/unified.jsonl` (each line has `ts`, `sid` = session id, `ver`, `msg`, `ctx`): `shell.turn.inference_done` → `prompt_tokens`, `cached_prompt_tokens`, `completion_tokens`, `reasoning_tokens`, `ttft_ms`, `tokens_per_sec`, `model_elapsed_ms` per inference (62 here). Joins to session by `sid`. `billing: fetched credits config` → `creditUsagePercent`, weekly `currentPeriod`, on-demand cap/used (276 snapshots = quota history).
- `sessions/<enc cwd>/<id>/events.jsonl` → `turn_started{model_id}`, `turn_ended{outcome}`, `first_token`; project from folder name. `memtrace/*.jsonl` = process memory samples (not usage). `worktrees.db(meta, worktrees)`. `models_cache.json` has context windows.

**agy / Antigravity** (`~/.gemini/antigravity-cli`):
- `conversations/<id>.db`: `gen_metadata(idx, data protobuf, size)` one row per model call (276 in the largest); `steps` (554), `trajectory_meta`, `executor_metadata`. Decoded wire format by hand (no schema). **Likely, not yet verified against ground truth:** field `1.4.2` = input tokens, `1.4.3` = output tokens, `1.4.5` = cached/read tokens, `1.9.10.1` = context tokens used with `1.9.10.4` = context window (256000), `1.11.2`/`1.4.9-10` = durations/latency. Treat as experimental; verify by comparing to agy's own displayed usage before shipping, and degrade to step counts if the layout changes.
- `conversation_summaries.db`: `step_count`, `last_modified_time`, `workspace_uris`, `parent_conversation_id`, `nesting_depth`, `agent_name`, `killed`. Quota: none on disk (`cli.log` has only refresh attempts and the signed-in email, never read into Meter).
- `~/.gemini/antigravity/` mirrors some of this (`conversations`, `brain`, `cli.log`).

**Cursor** (`~/.cursor`, `~/Library/Application Support/Cursor`): `ai-tracking/ai-code-tracking.db` (`ai_code_hashes.model/conversationId/timestamp`, `scored_commits` with AI-vs-human line counts, `conversation_summaries`); `state.vscdb` `cursorDiskKV` `bubbleId:*` (`modelInfo.modelName`, token fields all 0) and `composerData:*` (`createdAt`, `lastUpdatedAt`, `status`, empty `usageData`); `~/.cursor/projects/*` holds canvases/mcps/terminals only. No tokens/quota. New extra signal: AI-attributed lines of code per model and commit scoring.

**Cross-tool summary table** (what Meter can show locally): Codex = tokens, turns, durations, tool-call counts, quota %, reset, plan; Grok = tokens per inference, latency, quota % weekly, on-demand cap; Claude Code = tokens by type, model, project, thinking time; agy = tokens (experimental), context fill, steps, subagent tree; Cursor = turns by model, AI-code lines.
**Privacy rules for readers**: never select `title`, `first_user_message`, `item_json`, `preview`, `raw_summary`, message content, or log bodies; never open `auth*`, `*oauth*`, token or credential files; email in `cli.log` must not be read.

## Claude quota: where installed tools get it (2026-10-24)
User noted existing tools show Claude quota. Both CodexBar.app and OpenUsage.app are installed here. Checked (names/sizes only; did not open credential or cached-response files):
- They fetch Claude quota live from Anthropic (OAuth usage API; CodexBar falls back to the `claude` CLI via PTY, then web cookies). Not stored as a readable file: OpenUsage's `~/Library/Application Support/OpenUsage` holds only pricing, log-scan cache, Antigravity/Cursor items; CodexBar's has account snapshots for Codex, a Cursor session file (credential, avoid) and a Claude probe sandbox. Their `Library/Caches/<id>/Cache.db` are raw HTTP response caches (could contain account data), so Meter will not read them.
- Local-only options for Claude 5h/weekly quota, none enabled yet: (a) Claude Code's statusLine hook receives usage JSON on stdin; a Meter statusline script could append it to a Meter-owned file. Needs a change to `~/.claude/settings.json`, which already has an Orca statusLine, so it would have to chain with it. Host change: deferred until the user finalizes. (b) Run `claude` `/usage` in a PTY like CodexBar: spawns a process, rejected. (c) Live OAuth call: deferred opt-in mode.
- Not verified: the exact statusLine JSON field names. Check Claude Code docs before building (a).

## Design guidelines (user request, 2026-10-24)
Fetched Apple HIG JSON (menu bar, materials, windows, color, typography, accessibility, motion, layout) from developer.apple.com into the scratchpad; summarized in `docs/design.md` (scratch copies deleted). Honest limits: Apple's page change logs show no dedicated "macOS 27" rewrite of menu bar extras; the macOS 27 point found is "use menu item icons sparingly" (press) and reduced menu icons by default (forum roundup, unverified in SDK).
- Key conflict with the scaffold: HIG says show a menu, not a popover, for a menu bar extra. `MenuContent` still uses `.menuBarExtraStyle(.window)`; change to `.menu` when UI work starts.
- Also adopt: user-controlled visibility, Dock menu, Settings… ⌘,, system colors plus words (not color alone), VoiceOver labels, Reduce Motion, Liquid Glass only on controls.

## Two-surface decision (user, 2026-10-24)
Full-featured app window plus a minimal menu bar item that keeps running after the window closes. Details in `docs/design.md` ("Decided").
- Scaffold changed: `MenuBarExtra` now `.menu` style; `AppDelegate` returns false from `applicationShouldTerminateAfterLastWindowClosed`; removed `LSUIElement` from `scripts/bundle.sh` so the app has a Dock icon. `swift build` result recorded below. Not launched.

## Reader layer: Codex + Grok (2026-10-24)
Recommendations accepted by the user ("go ahead with your recommendations"): menu-only status item, icon + tightest percent (setting to hide the percent), up to three lines in the menu.
**Done**
- Package split: `MeterCore` (library), `Meter` (app, imports MeterCore), `MeterCoreTests`.
- `Sources/MeterCore/Model.swift`: `ToolID`, `Session`, `Quota`, `ReaderOutput`, `UsageReader` protocol (`isInstalled`, `read(since:)`), date/JSONL helpers (ISO8601 via `Date.ISO8601FormatStyle`, Sendable-safe).
- `CodexReader`: `state_5.sqlite` `threads` read-only, selecting only id, times, source, cwd, model, tokens_used (never `title`); subagent flag from `source`; quota from newest `rollout-*.jsonl` `token_count.rate_limits` (primary/secondary labeled Session/Weekly/Monthly by window length).
- `GrokReader`: `logs/unified.jsonl` only. Latest `billing: fetched credits config` → weekly quota; `shell.turn.inference_done` summed per `sid` (output = completion + reasoning, assumption); project from the `sessions/<encoded cwd>/<sid>` folder; model from `turn_started.model_id`.
- Tests (Swift Testing, 6 passing) use generated temp-dir fixtures only, including a decoy `title` column. They never touch real home folders. Ran `swift test` and `swift build`; no app launch.
**Not done / next**
- Reader robustness: Codex `tokens_used` is a single total (no input/output split); Codex DB path is `state_5.sqlite` (version number may change, glob it); reading a DB while Codex has it open in WAL mode untested against live data; large Grok log re-read each time (needs the incremental cache).
- Next readers: Claude Code (JSONL, dedupe by requestId), Cursor (models/turns), agy (activity first, experimental tokens).
- Then: daily rollup store, account model (add/hide), insights and alert logic (pure functions), menu UI wired to real data.
- Grok output-token semantics (is reasoning already inside completion?) unverified.

## Readers: Claude Code, Cursor, agy (2026-10-24)
**Done** (all with generated-fixture tests; `swift test`: 15 tests in 5 suites pass; `swift build` ok; app not launched; real home folders untouched by tests)
- `Session.turns` added (model calls/turns for tools without token counts).
- `ClaudeCodeReader`: `~/.claude/projects/**/*.jsonl` (incl. `subagents/`). Assistant lines only; dedupes by `requestId` keeping the largest `output_tokens`; input = input + cache_creation, cached = cache_read; one session per `sessionId` (subagent work folded in); no quota (none on disk).
- `CursorReader`: `state.vscdb` `cursorDiskKV` `composerData:*`; selects only `createdAt`, `lastUpdatedAt`, `modelConfig.modelName`, `json_array_length(fullConversationHeadersOnly)` via `json_extract`. Empty drafts (0 headers) skipped. Grok Bot appears as model `grok-4.6`. Tokens/quota nil. Project unknown (nil).
- `AgyReader`: `conversation_summaries.db`; steps as `turns`, workspace → project, child conversations (`parent_conversation_id`) folded into the parent. Tokens (protobuf) deferred and unverified.
**Known gaps / next**
- No live-data check yet for any reader (needs user approval). Cursor `state.vscdb` and Codex DB are WAL files opened by running apps: verify read-only open works live.
- agy start time falls back to last-modified when `last_user_input_time` is the zero date.
- Remaining: `ReaderRegistry` (installed tools), daily rollup store (history survives log cleanup), account model (add/hide), insights + alerts (pure functions, injected notifier), UI wired to data, incremental log cache.

## Core logic: registry, rollup, accounts, insights, alerts (2026-10-24)
**Done** (`swift test`: 25 tests in 9 suites pass; `swift build` ok; app not launched; temp-dir fixtures only)
- `Registry.swift`: `ToolID.displayName`, `ReaderRegistry.standard()/installed()`.
- `Rollup.swift`: `DayUsage` (day×tool×account×project×model), `Rollup.days(from:calendar:)` (subagents skipped; a session counts on its start day), `RollupStore` JSON with `merge(_:tools:fromDay:)` that replaces the re-read window and keeps older days, so history survives tool log cleanup.
- `Accounts.swift`: `Account` (id, tool, label, optional dataRoot, isHidden), `AccountStore` (add, setHidden, `hiddenKeys()` as "tool/accountID"), `ReaderOutput.excluding(hiddenKeys:)`. Built-in account id is "default" per tool, so keys include the tool.
- `Insights.swift`: `tightest`, `pace` (on course ≤90% projected, tight 90–100, will run out with ETA; needs ≥5% of window elapsed and >0 used; uses `observedAt`), `notes` (closest to cap ≥50%, pace, out of pattern ≥3× of ≥3 prior active days and ≥50k tokens, idle quota ≤10% vs another ≥80%), `MenuSummary.build` (≤3 lines, tightest first, warning only when pace/out-of-pattern note exists).
- `Alerts.swift`: `Alert`, `Notifier` protocol (no real implementation yet), `AlertEngine` (thresholds 80/95, exhausted, restored on new cycle, pace; once per account/window/cycle; first evaluation is baseline including pace; reset-time jitter tolerant).
**Bug fixed during testing**: pace alert fired on second pass for a quota already over pace at launch → now part of the baseline.
**Not done / next**
- Wire to UI: `AppModel` (observable) that runs readers on a timer + file watching, merges rollup, applies hidden accounts, computes `MenuSummary`; menu content and main window views from real data (Accounts add/hide UI, settings).
- Real `Notifier` using `UNUserNotificationCenter` (host-touching, off until user finalizes), quiet hours/snooze/per-alert toggles, stale tag + keep-last-good, incremental log cache.
- Insights quirks to review: note text is English only; `when()` rounds coarsely; "closest to cap" needs ≥50% used (my choice).
- Live read-only run of the readers on real data still awaits user approval.

## Everything built (2026-10-24, user: "continue … do not ask until everything is complete")
**State: first version feature-complete.** `swift test`: 37 tests (4 UI + 33 core) pass; `swift build` has no warnings; `scripts/bundle.sh` builds `build/Meter.app` (Info.plist lints OK; ad-hoc signed; **not launched, not installed**, per the standing rule).
**Live check (user-approved, read-only):** `swift run meter-probe` read real data via temp copies of the databases. All five readers work: Codex (7 sessions, quotas parsed, but its recorded windows were already past → shown as ended, not current), Grok (2 sessions, weekly 20%), Claude Code (1 session, 105 turns), Cursor (1 session, 83 turns, model grok-4.6), agy (17 sessions, 6057 steps).
**Added this round**
- `SQLiteSnapshot`: readers copy db + -wal + -shm to a temp dir and open the copy, so running apps are never touched.
- `Quota.isCurrent`, `ReaderRegistry.make`, `ReaderOutput.tagged`, `AccountStore.effective/setHidden(tool:id:)/remove`, `Dashboard`/`AccountStatus`/`Refresher` (keep last good on failure, stale after 10 min, ended windows separated, hidden accounts excluded from menu/notes/alerts, token/turn fallback lines when no quota), `AlertPolicy` (enabled kinds, quiet hours across midnight, snooze), `Format`, labeled account names in notes/menu.
- `MeterUI` library: `AppModel` (@Observable; timer refresh, rollup merge, alerts through a `Notifier`, accounts, settings persisted as JSON in UserDefaults, launch-at-login via SMAppService only when toggled), `UserNotifier` (UNUserNotificationCenter, no-op without a bundle, permission requested only when the first alert is switched on), views: Overview cards, Usage chart (Swift Charts), Accounts with add sheet (NSOpenPanel only on user click), Settings General + Alerts, `MenuBarLabel`, `MenuContent`.
- `Meter` app: Window + MenuBarExtra(.menu) + Settings scene; `AppDelegate` keeps running after last window closes, Dock menu.
- UI verification: hidden-window PNG renders in tests (`METER_SNAPSHOT_DIR`), inspected Overview, Usage, Accounts, Settings. Fixed: chart date labels, duplicate/ambiguous notes (now include account label), gray progress bars (custom capsule), Settings height.
**Not done / known limits**
- Never launched: menu bar item, Dock menu, notification delivery, launch-at-login and window reopen behaviour are untested at runtime.
- No file watching (timer only); no incremental log cache (full re-scan each refresh; fine at current sizes: Grok log ~2.6 MB).
- Grok output tokens = completion + reasoning (unverified). Codex gives one token total per thread (shown as input). agy tokens unread (protobuf, unverified layout). Claude Code history only goes back as far as the tool keeps logs; rollups start when Meter first runs.
- Added accounts: the account's data folder is passed to the reader; how each tool actually separates accounts on disk is unverified.
- Alert text is English only; no per-tool alert switches (only per kind).
- Live-quota mode for Claude Code/Cursor/agy deferred by user decision.
- Test fixtures use invented data; the one real-data check is the probe above.
**To finalize later (user decision):** launch `build/Meter.app` once, confirm menu bar item and window, then decide on login item, notifications, and whether to add the Claude Code statusLine capture or live-quota mode.

## Host footprint audit (user question, 2026-10-24)
- Found and fixed a violation of the standing rule: the UI tests created `UserDefaults(suiteName:)` domains, which left 8 `meter-test-*` / `meter-roundtrip-*` plists in `~/Library/Preferences`. Removed those 8 files (created by this session), added `KeyValueStore` (UserDefaults in the app, `MemoryStore` in tests). After a full `swift test`: 0 such files, no `~/Library/Application Support/Meter`. 37 tests pass.
- What running the app writes: see README "What running Meter changes on your Mac".

## Safe ways to run the app (user, 2026-10-24: "I will create a macOS VM and test the UI")
**Chosen by the user: test in a macOS VM** (VMware Fusion is installed on the host; macOS guest support depends on the Fusion version).
- `scripts/make-vm-kit.sh` → `build/vm-kit/` (gitignored): `Meter.app` (ad-hoc signed, arm64), release `meter-probe`, `setup-fake-data.sh`, `README.txt` with a test checklist. `setup-fake-data.sh` refuses unless `sysctl kern.hv_vmm_present` is 1 (verified: it refuses on the host, exit 1, wrote nothing), then runs `meter-probe --fake-home "$HOME"` to create invented Codex, Grok, Claude Code, Cursor and agy data in the VM user's home.
- Also added a no-VM option: `scripts/sandbox.sh` runs the bare binary with `CFFIXED_USER_HOME` + `METER_SANDBOX` pointing at a throwaway folder of fake data (file reads and Meter's files and settings stay there; notifications and login item off; `FileStore` replaces UserDefaults). Verified without opening the GUI: the probe reads the fake home correctly and sees "not installed" for the real tools. NOT fully isolated: UserDefaults writes go through a system daemon that ignores `CFFIXED_USER_HOME` (observed: a test plist landed in the real `~/Library/Preferences`; I deleted it), so AppKit may still write `~/Library/Preferences/Meter.plist` (script deletes it on exit), and the Dock icon and menu bar item appear. Not run by me.
- `meter-probe --fake-home <dir>` generates the fake data (`Sources/meter-probe/FakeHome.swift`). Fake quotas are relative to now, so re-run to refresh.
- 38 tests pass (added a FileStore/sandbox test); no Meter files in `~/Library/Preferences` afterward.
**For the VM test, things never yet run:** menu bar item, Dock menu, window close keeps running, notification permission and delivery, login item, settings persistence across relaunch, add-account open panel, light/dark/contrast/VoiceOver.

## VM test guide written (2026-10-24)
`docs/vm-test-guide.md`: build the kit, copy into the VM, set up fake data, checklist, what to report back, cleanup. The user will test on 2026-10-25. Next agent: start from the user's VM findings.

## 2026-11-14: .gitignore hardened

Added ignore rules so the app stays clean after extraction from the incubator (repo-wide ignore pass). Added `.env` secrets rules, `DerivedData/` and `*.app`. No code changes.
