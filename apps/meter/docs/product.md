# Meter

A menu-bar monitor for how the AI tools on this Mac are being used. The first four are Antigravity (the `agy` command), Grok, Codex, and Claude Code. Others can be added later as their own readers.

Status: spec only. Nothing is built yet.

## The job

Each tool has its own quota, its own reset, and its own session log. Meter puts those figures in one place and says which one matters right now: which tool is closest to its cap, which project spent the most today, and whether today's pace runs out before the window resets.

## What it reads

Meter reads records the tool already wrote on this Mac: session logs, local billing snapshots, and whatever usage file that tool keeps. It does not send prompts anywhere, and it does not call the provider.

It does not store the text of prompts or replies. A session contributes its tool, model, project, token counts, recorded cost, start, and end. If a tool did not record a cost, Meter leaves the cost blank. It does not invent a price.

Work a tool folded into a parent session, such as a subagent, is counted once, under that session.

## Per tool

One row per tool.

- How much of the current quota is used, and when it resets.
- Tokens today, and recorded cost today, when the tool wrote one down.
- The model mix for today.
- The busiest project today.

The menu bar shows the tightest limit, as a percent. It shifts toward a warning as that limit fills, and it names the tool.

## The intelligent part

Four notes, written in plain language, and only when they are true.

- **Closest to the cap.** The tool with the least room left in its current window.
- **Pace.** If today's rate continues, whether that tool runs out before the reset, and about when.
- **Out of pattern.** A project or session whose tokens are several times that project's recent days.
- **Idle quota.** A tool that has barely been used while another is near its cap.

Meter does not move work between tools, and it does not stop a session. It reports.

## Accounts

A tool can have more than one account, such as personal and work. Each account is its own row under its tool, with its own quota, window, and usage.

- **Add.** Meter lists the accounts it can find on its own. You can also add one by hand: a label and the folder that account's records live in (for example a second config or home directory for that tool).
- **Hide.** A hidden account leaves the menu, the menu-bar percent, the notes, and the alerts. Its records stay on disk and are not deleted. Unhide brings it back with history.
- **Label.** You name each account. Meter never reads or shows credentials, tokens, or keys, and tells accounts apart by folder and label only.
- **Rollup.** The menu-bar percent uses visible accounts only. A tool with several visible accounts shows the tightest one.
- **No account found.** A tool with records but no identifiable account shows as one default account.

## Alerts

Notes in the menu say what is true now. Alerts tell you when something changes while you are not looking. They are modeled on what CodexBar offers (exhausted and restored session notifications, reset countdowns, status badges), plus Meter's own insights.

- **Threshold.** A tool crosses a chosen percent of its window (defaults 80 and 95).
- **Exhausted and restored.** A tool's window hits 100%, and again when it resets and room returns.
- **Pace warning.** Today's rate will use up the window before the reset. Sent once per window.
- **Out of pattern.** A project or session far above its recent days.
- **Reader trouble.** A tool's logs stopped updating or became unreadable. Silence is not shown as zero.

Rules: each alert fires once per window per tool, with no repeats on every refresh. Per-tool, per-account, and per-alert switches, quiet hours, and a snooze. Alerts name the tool and the number, never prompt text. The menu-bar icon also tints at the thresholds. Delivery uses macOS notifications behind a small notifier interface, so tests use a fake and never reach the host.

## Adding a tool

A new tool is a new reader inside Meter: where that tool keeps its logs, which fields are tokens, cost, model, and project, and how its quota window works. Grok Bot comes from Cursor, so it is read through a Cursor reader, not a separate tool. Cursor records which model each turn used (Grok shows up there) but not tokens or quota, so its row shows turns and conversations per model, with no percent. Claude Duet runs in the cloud and in phone and web apps, not on this Mac, and shares its usage limits with the rest of Claude, so it leaves no local records for Meter to read. Meter can only say that its use counts against the same Claude plan. Other tools wait until they are installed and their records are understood the same way. A tool that is not installed shows as not installed, never as zero usage. Readers are not shared with any other app in this repo.

## What it does not do

- A bill from the provider, or a guess at a price the tool did not record.
- A transcript browser.
- An account, or a copy of usage sent off the Mac.
- Starting, stopping, or steering an agent.
