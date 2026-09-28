# Meter

A menu-bar monitor for how the AI tools on this Mac are being used. The first two are Antigravity, the `agy` command, and Grok. Others can be added later as their own readers.

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

## Adding a tool

A new tool is a new reader inside Meter: where that tool keeps its logs, which fields are tokens, cost, model, and project, and how its quota window works. Claude Code, Codex, and others wait until their records are understood the same way. Readers are not shared with any other app in this repo.

## What it does not do

- A bill from the provider, or a guess at a price the tool did not record.
- A transcript browser.
- An account, or a copy of usage sent off the Mac.
- Starting, stopping, or steering an agent.
