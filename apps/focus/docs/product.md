# Focus

A Pomodoro timer for Mac. It runs one interval at a time and keeps the remaining time in the menu bar.

Status: spec only. Nothing is built yet.

## The job

Start a stretch of work, get told when it ends, take the break, and see how many of those stretches you actually finished today.

## Intervals

Three kinds.

| Interval | Default |
|----------|---------|
| Focus | 25 minutes |
| Short break | 5 minutes |
| Long break | 15 minutes |

A long break arrives after four focus intervals. The counts and the lengths are settings. Changing them applies to the next interval, not the one already running.

## Menu bar

The item shows the time left, as `24:10`, and a small mark for which interval it is. A break looks different from focus, so a glance is enough.

Clicking it opens a small panel:

- Start, pause, and resume.
- Skip to the next interval.
- The name of the current interval, and how many focus intervals are done in this round.
- An optional one-line label for what this stretch is for. The label is for you. It is not a task manager.

When the interval ends, a notification fires and a sound plays. Both can be turned off. The next interval does not start by itself. Focus waits, so a break you are not taking does not eat the clock.

## Today

The panel shows focus intervals finished today, and the minutes those intervals added up to. A skipped interval does not count. A paused interval counts the time that actually ran.

The same numbers are kept for the last seven days, as a simple list of days. That history stays on the Mac.

## What it does not do

- A task list, a calendar, or a project board.
- Blocking other apps, or turning on Do Not Disturb for you.
- An account, a sync, or a streak that scolds you for missing a day.
