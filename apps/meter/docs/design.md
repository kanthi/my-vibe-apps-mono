# Interface design

Source: Apple's Human Interface Guidelines pages fetched on 2026-10-24 from the JSON behind developer.apple.com (the menu bar, materials, windows, color, typography, accessibility, motion, layout). Apple's own page change logs show no macOS 27 rewrite of the menu bar extras section; the macOS 27 change reported in press (MacRumors, Daring Fireball) is that Apple revised the menu guidance to use menu item icons "sparingly and with purpose", and macOS 27.0 shows a reduced set of menu item images, with `NSMenuItem.preferredImageVisibility` to control it. That second part comes from a forum roundup, not Apple's HIG, so verify it in the SDK before relying on it.

## Rules that shape Meter

**Menu bar item**
- Show a **menu, not a popover**, unless the content is too complex for a menu. Meter's first version is a native menu (`MenuBarExtra` with `.menu` style): a header line, one line per tool (name, percent or tokens today, reset), the notes as disabled lines, then `Open Meter…`, `Settings…` (⌘,), `Quit`. The scaffold currently uses `.window`; switch it. A richer panel can come later only if a menu cannot carry the content.
- Use a **symbol** for the icon, template-colored, readable in light, dark, and when selected. The menu bar is 24 pt high. Keep any text beside the icon short (a percent and a tool abbreviation).
- **The user decides whether the item appears.** A setting "Show in menu bar", offered during first run, not forced. Do not depend on being visible: the system hides items when space is tight.
- Provide a **Dock menu** with the same status and Open/Settings, so the app is reachable when the item is hidden.
- Menu item icons: few, only for familiar actions. No decorative icon on every row (macOS 27 direction).
- Menu titles: short; title-style capitalization if more than one word. Always show the same items; disable instead of hiding.

**Window and Settings**
- A normal window for detail (per-tool cards, charts, accounts) that can be resized and moved; standard menus (App, Edit, View, Window, Help) with standard shortcuts. Settings opens with ⌘, and holds app-level settings only: show in menu bar, launch at login (off by default), refresh, tools and accounts (add, hide), alerts, quiet hours.
- Every command available from the menu bar. Support keyboard-only use.

**Material and color**
- Standard system materials and controls; they pick up the macOS 27 look automatically. Liquid Glass only on the controls/navigation layer, never on content (cards, charts, rows). No custom glass on rows.
- System colors and the user's accent color (do not hard-code blue). Pace verdict colors must come from system colors and **never carry meaning alone**: pair with a word or symbol ("Limit in 3h", "tight"). Meet contrast in light, dark, and Increase Contrast.

**Type and layout**
- SF Pro via system text styles (macOS has no Dynamic Type; use the built-in macOS styles). Numbers use monospaced digits so percents do not jitter.
- Respect standard margins and alignment; prefer less nesting on large displays but keep density comfortable.

**Motion and accessibility**
- Motion optional: no animation as the only signal; honor Reduce Motion (no bar-fill sweeps, no pulsing). Brief feedback only.
- VoiceOver labels on every gauge ("Codex, 20 percent used, resets Tuesday"), the status item included. Notifications carry plain-language text.

## Decided (user, 2026-10-24)
- Two surfaces. The **app** is a regular macOS app with a Dock icon and a full window: every tool, account, chart, history, alert, and setting. The **menu bar item** is minimal: only useful, needed information. Closing the window does not quit; Meter keeps running in the menu bar until you choose Quit.
- Menu bar content, in order: the tightest limit across visible accounts (tool, percent, reset), a warning line only when an alert condition is true, `Open Meter…`, `Quit`. Nothing else: no per-tool tables, charts, or settings in the menu.
- Menu bar label: icon plus the tightest percent (a setting can reduce it to icon only). The menu bar draws template images, so a filling limit changes the symbol (an exclamation gauge from 90%) instead of its color, and the percent is always text.
- Menu lines: at most three, the tightest first. Chosen by the author as the default.
- Built: Overview, Usage and Accounts in the window; General and Alerts in Settings (⌘,); a Dock menu; pace colors always paired with words and a symbol; progress bars drawn as plain capsules so color does not depend on window focus.
