# Reference study: OpenUsage and CodexBar

Both are MIT, Swift, menu-bar apps. Cloned shallow into the session scratchpad (steipete/CodexBar at d34fbb4, robinebers/openusage at 26132ff) and read as docs and source only. Nothing was built or run. We copy ideas, not code.

## How they get data

| | OpenUsage | CodexBar |
|---|---|---|
| Quota % and reset | **Calls each provider's own API** with the login the tool already stored (Codex `chatgpt.com/backend-api/wham/usage`, Claude `api.anthropic.com/api/oauth/usage`, Grok `cli-chat-proxy.grok.com/v1/billing`, Cursor Connect RPC `api2.cursor.sh`, agy local language server then Cloud Code). Reads tokens from `~/.codex/auth.json`, `~/.grok/auth.json`, Cursor's state DB, Keychain; refreshes them. | Same idea, with fallback order: PAT, OAuth API, then the CLI's own RPC (`codex app-server`). Plus optional web-dashboard cookies. |
| Spend and tokens | **Local logs**, parsed by an incremental JSONL scanner cached by path+size+mtime; resumes from the last complete line. Prices applied after parsing from a bundled table. | Local cost scan into SQLite cost stores. |
| Failure | Keep last good values, show a warning triangle, tag "Outdated" after about 10 min, give up on a provider after 120 s. | Per-provider error and credential notifications. |

## How they present it

- Menu bar: user stars up to 2 metrics per provider; text or tiny bars. CodexBar draws a two-bar meter (session on top, weekly below). OpenUsage can hide the strip during screen sharing.
- Popover: one card per provider with progress bars, reset countdowns that tick every 30 s, and a pace-colored fill (blue on course, yellow tight, red with "Limit in 3h 5m").
- Extras: local HTTP API and CLI (OpenUsage), widgets and a CLI (CodexBar), iCloud history sync.
- Multi-account: per-account cards, matched by workspace and email; aliases.

## How they alert

- OpenUsage: three switches, all off by default: **Almost Out** (under 10% left), **Cutting It Close**, **Will Run Out** (projected before reset). Fire on a new crossing, dedupe until the state changes, re-arm after recovery or a new window. A quota already bad at launch sets a baseline with no alert. Permission is requested only when the first switch is turned on, with an in-app prompt to fix it in System Settings.
- CodexBar: exhausted and restored notifications, predictive pace warnings (only if run-out probability is 0.5 or more and an ETA exists), reset-credit expiry, credential trouble. State keyed by provider, account, window, and reset cycle, with a tolerance so a slightly shifted `resetsAt` counts as the same cycle.

## Important finding: Grok Bot

OpenUsage reads **Grok Bot** from Cursor as its own weekly allowance, separate from Cursor's billing meter, through `DashboardService/GetSandUsageStatus` on `api2.cursor.sh` using the Cursor login. That figure is not on disk, which matches our spike (token fields zero, usage empty).

## What this means for Meter

1. **Our "local files only, never call the provider" rule cannot produce quota for Claude Code, Cursor (including Grok Bot), or agy.** Codex and Grok do log a percent locally, so local-only works for those two.
2. The reference apps solve this by reusing the tool's stored login. That means reading credential files or Keychain and making network calls, which is outside today's spec and touches the host. Decision needed from the user; see AGENT_WORKLOG.
3. Adopt regardless: keep last good value with a stale tag, per-file incremental log cache, alert dedupe keyed by (account, window, reset cycle) with baseline-on-launch, pace verdict colors, starring metrics for the menu bar, duration-based session/weekly classification (not primary/secondary slot), preserving `used_percent` verbatim, bundled price table applied after parsing.
