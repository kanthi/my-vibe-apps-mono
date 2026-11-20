# MacHealth

Local-first macOS health monitor, plain-English task manager, AI agent monitor, load balancer, and disk cleaner built with **Rust**, **GPUI**, and **AppKit / Carbon**.

- **Status:** `wip` (Phases 1–5 complete)
- **Platform:** macOS 14+ (Apple Silicon & Intel)
- **Stack:** Rust (2024) · GPUI (`gpui` 0.2.2 with `runtime_shaders`) · AppKit (`objc2-app-kit` `NSStatusItem`) · Carbon (`RegisterEventHotKey`)

## Documentation

- [PRODUCT_DESIGN.md](PRODUCT_DESIGN.md) — Complete features list, product design, architecture, and phased implementation plan.
- [AGENT_WORKLOG.md](AGENT_WORKLOG.md) — Ongoing engineering log, build/test status, and next steps.

## Build, Test & Run (Zero Host Registration Mode)

Run commands from inside `apps/machealth/`:

```bash
# Run all 52 unit & live macOS integration tests across the workspace
cargo test --workspace

# Run headless ephemeral smoke test (no windows, no NSStatusItem, no hotkey, zero disk writes)
cargo run --bin machealth -- --smoke-test

# Launch GUI in ephemeral dev mode (no files written to ~/Library/Application Support/MacHealth,
# no menu bar item, and no global hotkey grabbed)
cargo run --bin machealth -- --ephemeral --no-menubar --no-hotkey

# Launch full desktop app + NSStatusItem menu bar badge + Ctrl-Opt-Space Quick Search panel
cargo run --bin machealth
```

### Host Isolation Flags

| Flag / Env Var | Effect |
|---|---|
| `--smoke-test` | Runs full telemetry, process, AI agent, security, and cleanup verification in memory and exits `0` without opening windows or registering menu bar / hotkeys. |
| `--ephemeral` / `MACHEALTH_EPHEMERAL=1` | Keeps history and undo ledger strictly in memory (never creates `~/Library/Application Support/MacHealth/`). |
| `MACHEALTH_DATA_DIR=/tmp/dir` | Redirects persistent JSONL history and undo ledger to a custom sandbox folder. |
| `--no-menubar` | Skips creating the process-scoped `NSStatusItem` menu bar badge. |
| `--no-hotkey` | Skips registering the process-scoped `Control-Option-Space` global hotkey. |
| `--open-popover` | Opens the compact Menu Bar & Quick Search popover immediately on startup. |

## Workspace Layout

```text
apps/machealth/
├── Cargo.toml
├── PRODUCT_DESIGN.md
├── AGENT_WORKLOG.md
└── crates/
    ├── machealth-core/   # macOS telemetry, 8-area scoring, process tree, AI agents, load balancer, security, cleanup, history & Spotlight search
    ├── machealth-ui/     # Reactive GPUI views, theme tokens, state controller, and confirmation modals
    └── machealth-app/    # GPUI macOS host, NSStatusItem menu bar badge, Carbon Ctrl-Opt-Space hotkey & host-isolation CLI
```
