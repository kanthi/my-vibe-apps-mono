use crate::state::{AppState, Metric, Page};
use crate::theme::Theme;
use crate::widgets::*;
use chrono::{Duration, Local, NaiveDate};
use gpui::{
    div, prelude::*, px, Context, Entity, IntoElement, PathPromptOptions, Render, SharedString, Subscription, Window,
};
use meter_core::format;
use meter_core::insights::{self, PaceVerdict};
use meter_core::{AccountStatus, AlertKind, Quota, ToolId};
use std::collections::BTreeMap;

pub struct RootView {
    state: Entity<AppState>,
    _subs: Vec<Subscription>,
}

impl RootView {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let subs = vec![
            cx.observe(&state, |_, _, cx| cx.notify()),
            cx.observe_window_appearance(window, |_, _, cx| cx.notify()),
        ];
        RootView { state, _subs: subs }
    }

    fn nav(&self, t: &Theme, page: Page, label: &'static str, glyph: &'static str, cx: &mut Context<Self>) -> gpui::AnyElement {
        let current = self.state.read(cx).page;
        let selected = current == page;
        let hover = t.line;
        div()
            .id(SharedString::from(format!("nav-{label}")))
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .when(selected, |d| d.bg(t.selected).font_weight(gpui::FontWeight::SEMIBOLD))
            .when(!selected, move |d| d.hover(move |s| s.bg(hover)))
            .on_click(cx.listener(move |this, _, _, cx| this.state.update(cx, |s, cx| {
                s.page = page;
                cx.notify();
            })))
            .child(div().w(px(18.)).text_color(t.secondary).child(glyph))
            .child(label)
            .into_any_element()
    }
}

impl Render for RootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = Theme::for_appearance(window.appearance());
        let page = self.state.read(cx).page;
        let body = match page {
            Page::Overview => self.overview(&t, cx).into_any_element(),
            Page::Usage => self.usage(&t, cx).into_any_element(),
            Page::Accounts => self.accounts(&t, cx).into_any_element(),
            Page::Settings => self.settings(&t, cx).into_any_element(),
        };
        div()
            .size_full()
            .flex()
            .bg(t.window)
            .text_color(t.text)
            .text_sm()
            .child(
                div()
                    .w(px(190.))
                    .h_full()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_3()
                    .bg(t.sidebar)
                    .border_r_1()
                    .border_color(t.line)
                    .child(self.nav(&t, Page::Overview, "Overview", "◎", cx))
                    .child(self.nav(&t, Page::Usage, "Usage", "▮", cx))
                    .child(self.nav(&t, Page::Accounts, "Accounts", "☺", cx))
                    .child(self.nav(&t, Page::Settings, "Settings", "⚙", cx)),
            )
            .child(div().id("page").flex_1().min_w_0().h_full().overflow_y_scroll().p_6().child(body))
    }
}

// ---------------------------------------------------------------- overview

fn pace_view(t: &Theme, q: &Quota, now: chrono::DateTime<chrono::Utc>) -> Option<(String, gpui::Rgba)> {
    match insights::pace(q) {
        PaceVerdict::NotEnoughData => None,
        PaceVerdict::OnCourse { projected_percent } => Some((format!("✓ On course, about {:.0}% by reset", projected_percent), t.green)),
        PaceVerdict::Tight { projected_percent } => Some((format!("⚠︎ Tight, about {:.0}% by reset", projected_percent), t.orange)),
        PaceVerdict::WillRunOut { at } => Some((format!("✕ Runs out {}", format::until(at, now)), t.red)),
    }
}

fn quota_row(t: &Theme, q: &Quota, now: chrono::DateTime<chrono::Utc>) -> impl IntoElement {
    let pace = pace_view(t, q, now);
    let color = pace.as_ref().map(|p| p.1).unwrap_or(t.accent);
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .flex()
                .justify_between()
                .child(div().font_weight(gpui::FontWeight::MEDIUM).child(q.label.clone()))
                .child(div().child(format!("{:.0}% used", q.used_percent))),
        )
        .child(usage_bar(t, q.used_percent, color))
        .child(
            div()
                .flex()
                .justify_between()
                .text_xs()
                .child(div().text_color(t.secondary).child(match q.resets_at {
                    Some(r) => format!("Resets {}", format::until(r, now)),
                    None => String::new(),
                }))
                .children(pace.map(|(text, c)| div().text_color(c).child(text))),
        )
}

fn tool_card(t: &Theme, s: &AccountStatus, now: chrono::DateTime<chrono::Utc>) -> impl IntoElement {
    let title = if s.account.id == "default" {
        s.account.tool.display_name().to_string()
    } else {
        format!("{} · {}", s.account.tool.display_name(), s.account.label)
    };
    // Cards share the row and wrap when the window is narrow; they stop growing at a readable width.
    let mut c = card(t).flex_1().min_w(px(300.)).max_w(px(560.)).child(
        div()
            .flex()
            .justify_between()
            .child(div().text_base().font_weight(gpui::FontWeight::SEMIBOLD).child(title))
            .child(div().text_xs().text_color(t.orange).child(if s.is_stale {
                "Outdated"
            } else if s.error.is_some() {
                "Read problem"
            } else {
                ""
            })),
    );
    if s.quotas.is_empty() {
        let msg = if s.ended_windows.is_empty() {
            format!("No limit is recorded on this Mac for {}.", s.account.tool.display_name())
        } else {
            "The last recorded window has ended; waiting for new data.".to_string()
        };
        c = c.child(secondary(t, msg));
    }
    for q in &s.quotas {
        c = c.child(quota_row(t, q, now));
    }
    let mut stats = div().flex().flex_col().gap_1().pt_2().border_t_1().border_color(t.line).text_xs();
    let line = |k: &str, v: String| div().flex().gap_3().child(div().w(px(110.)).text_color(t.secondary).child(k.to_string())).child(v);
    if s.tokens_today > 0 {
        stats = stats.child(line("Tokens today", format::tokens(s.tokens_today)));
    }
    if s.turns_today > 0 {
        stats = stats.child(line("Turns today", s.turns_today.to_string()));
    }
    if s.tokens_today == 0 && s.turns_today == 0 {
        stats = stats.child(line("Today", "No activity".into()));
    }
    if let Some(m) = &s.top_model {
        stats = stats.child(line("Model", m.clone()));
    }
    if let Some(p) = &s.top_project {
        stats = stats.child(line("Busiest project", format::base_name(p)));
    }
    c.child(stats)
}

impl RootView {
    fn header(&self, t: &Theme, title: &'static str, cx: &mut Context<Self>) -> gpui::AnyElement {
        let (refreshing, last) = {
            let s = self.state.read(cx);
            (s.is_refreshing, s.last_refresh)
        };
        let stamp = last.map(|d| format!("Updated {}", d.with_timezone(&Local).format("%-I:%M %p"))).unwrap_or_else(|| "Not refreshed yet".into());
        div()
            .flex()
            .items_center()
            .justify_between()
            .mb_4()
            .child(heading(title))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(secondary(t, if refreshing { "Refreshing…".to_string() } else { stamp }))
                    .child(button(t, "refresh", "Refresh", cx.listener(|this, _, _, cx| {
                        this.state.update(cx, |s, cx| s.refresh(cx));
                    }))),
            )
            .into_any_element()
    }

    fn overview(&self, t: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let header = self.header(t, "Overview", cx);
        let s = self.state.read(cx);
        let now = s.last_refresh.unwrap_or_else(chrono::Utc::now);
        let shown: Vec<&AccountStatus> = s.dashboard.accounts.iter().filter(|a| !a.account.is_hidden && a.installed).collect();
        let missing: Vec<&str> =
            s.dashboard.accounts.iter().filter(|a| !a.installed && !a.account.is_hidden).map(|a| a.account.tool.display_name()).collect();
        let mut page = div().flex().flex_col().gap_4().child(header);
        if !s.dashboard.notes.is_empty() {
            let mut notes = card(t).gap_1();
            for n in &s.dashboard.notes {
                notes = notes.child(format!("•  {}", n.text));
            }
            page = page.child(notes);
        }
        let mut grid = div().flex().flex_wrap().gap_4();
        for a in &shown {
            grid = grid.child(tool_card(t, a, now));
        }
        page = page.child(grid);
        if shown.is_empty() {
            page = page.child(card(t).child("No tools found").child(secondary(t, "Meter looks for Codex, Grok, Claude Code, Cursor and agy on this Mac.")));
        }
        if !missing.is_empty() {
            page = page.child(secondary(t, format!("Not installed: {}", missing.join(", "))));
        }
        page
    }
}

// ---------------------------------------------------------------- usage

impl RootView {
    fn usage(&self, t: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let header = self.header(t, "Usage", cx);
        let (metric, range, days, tz) = {
            let s = self.state.read(cx);
            (s.usage_metric, s.usage_range_days, s.dashboard.days.clone(), s.tz)
        };
        let today = chrono::Utc::now().with_timezone(&tz).date_naive();
        let dates: Vec<NaiveDate> = (0..range).rev().map(|i| today - Duration::days(i)).collect();
        // date -> tool -> value
        let mut by_day: BTreeMap<String, BTreeMap<ToolId, i64>> = BTreeMap::new();
        for d in &days {
            let v = if metric == Metric::Tokens { d.tokens() } else { d.turns };
            if v > 0 {
                *by_day.entry(d.day.clone()).or_default().entry(d.tool).or_default() += v;
            }
        }
        let key = |d: &NaiveDate| d.format("%Y-%m-%d").to_string();
        let max = dates.iter().map(|d| by_day.get(&key(d)).map(|m| m.values().sum::<i64>()).unwrap_or(0)).max().unwrap_or(0);
        let tools_used: Vec<ToolId> = ToolId::ALL.iter().copied().filter(|tool| by_day.values().any(|m| m.contains_key(tool))).collect();
        let fmt = |n: i64| if metric == Metric::Tokens { format::tokens(n) } else { n.to_string() };

        let controls = div()
            .flex()
            .gap_4()
            .mb_2()
            .child(
                div()
                    .flex()
                    .gap_1()
                    .child(segment(t, "m-tokens", "Tokens", metric == Metric::Tokens, cx.listener(|this, _, _, cx| {
                        this.state.update(cx, |s, cx| { s.usage_metric = Metric::Tokens; cx.notify(); });
                    })))
                    .child(segment(t, "m-turns", "Turns", metric == Metric::Turns, cx.listener(|this, _, _, cx| {
                        this.state.update(cx, |s, cx| { s.usage_metric = Metric::Turns; cx.notify(); });
                    }))),
            )
            .child(div().flex().gap_1().children([7i64, 14, 30].map(|n| {
                segment(t, SharedString::from(format!("r-{n}")), format!("{n} days"), range == n, cx.listener(move |this, _, _, cx| {
                    this.state.update(cx, |s, cx| { s.usage_range_days = n; cx.notify(); });
                }))
            })));

        let page = div().flex().flex_col().gap_3().child(header).child(controls);
        if max == 0 {
            return page.child(card(t).child("No usage yet").child(secondary(
                t,
                "Usage appears here as Meter reads your tools. History is kept after a tool cleans up its logs.",
            )));
        }
        let chart_h = 220.0f32;
        let mut cols = div().flex().items_end().gap_1().h(px(chart_h)).w_full();
        for (i, d) in dates.iter().enumerate() {
            let m = by_day.get(&key(d));
            let total: i64 = m.map(|m| m.values().sum()).unwrap_or(0);
            let mut col = div().flex().flex_col().justify_end().flex_1().h_full().items_center();
            col = col.child(div().text_xs().text_color(t.secondary).child(if total > 0 && range <= 14 { fmt(total) } else { String::new() }));
            let mut stack = div().flex().flex_col().w_full().rounded_sm().overflow_hidden();
            if let Some(m) = m {
                for tool in ToolId::ALL.iter().rev() {
                    if let Some(v) = m.get(tool) {
                        let h = (*v as f32 / max as f32) * (chart_h - 20.0);
                        stack = stack.child(div().w_full().h(px(h.max(1.0))).bg(t.tool_color(*tool)));
                    }
                }
            }
            col = col.child(stack);
            let _ = i;
            cols = cols.child(col);
        }
        let step = if range > 14 { 5 } else { 2 };
        let mut labels = div().flex().gap_1().w_full();
        for (i, d) in dates.iter().enumerate() {
            labels = labels.child(div().flex_1().flex().justify_center().text_xs().text_color(t.secondary).child(
                if (dates.len() - 1 - i) % step == 0 { d.format("%b %-d").to_string() } else { String::new() },
            ));
        }
        let mut legend = div().flex().gap_4().pt_2();
        for tool in tools_used {
            legend = legend.child(
                div().flex().items_center().gap_1().child(div().size(px(10.)).rounded_full().bg(t.tool_color(tool))).child(tool.display_name()),
            );
        }
        page.child(card(t).child(secondary(t, format!("Peak day: {}", fmt(max)))).child(cols).child(labels).child(legend))
    }
}

// ---------------------------------------------------------------- accounts

impl RootView {
    fn accounts(&self, t: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let header = self.header(t, "Accounts", cx);
        let (accounts, add_tool, statuses) = {
            let s = self.state.read(cx);
            (s.accounts.clone(), s.add_tool, s.dashboard.accounts.clone())
        };
        let mut page = div().flex().flex_col().gap_4().child(header);
        for tool in ToolId::ALL {
            let mut group = card(t).child(div().text_base().font_weight(gpui::FontWeight::SEMIBOLD).child(tool.display_name()));
            for a in accounts.iter().filter(|a| a.tool == tool) {
                let installed = statuses.iter().find(|s| s.account.key() == a.key()).map(|s| s.installed).unwrap_or(true);
                let acct = a.clone();
                let acct2 = a.clone();
                let detail = match (&a.data_root, installed) {
                    (Some(r), true) => Some(r.clone()),
                    (Some(r), false) => Some(format!("{r} (not found)")),
                    (None, false) => Some("Not installed".into()),
                    (None, true) => None,
                };
                let controls = div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(div().text_xs().text_color(t.secondary).child(if a.is_hidden { "Hidden" } else { "Shown" }))
                    .child(toggle(t, SharedString::from(format!("show-{}", a.key())), !a.is_hidden, cx.listener(move |this, _, _, cx| {
                        let hide = !acct.is_hidden;
                        this.state.update(cx, |s, cx| s.set_hidden(hide, &acct, cx));
                    })))
                    .children((a.id != "default").then(|| {
                        button(t, SharedString::from(format!("rm-{}", a.key())), "Remove", cx.listener(move |this, _, _, cx| {
                            this.state.update(cx, |s, cx| s.remove_account(&acct2, cx));
                        }))
                    }));
                group = group.child(row(t, a.label.clone(), detail, controls));
            }
            page = page.child(group);
        }
        let choose = cx.listener(move |this, _, _, cx| {
            let rx = cx.prompt_for_paths(PathPromptOptions { files: false, directories: true, multiple: false, prompt: Some("Choose".into()) });
            let state = this.state.clone();
            cx.spawn(async move |_, cx| {
                if let Ok(Ok(Some(paths))) = rx.await {
                    if let Some(path) = paths.first() {
                        let label = path.file_name().map(|n| n.to_string_lossy().trim_start_matches('.').to_string()).unwrap_or_else(|| "Account".into());
                        let root = path.to_string_lossy().to_string();
                        state.update(cx, |s, cx| s.add_account(add_tool, label, root, cx)).ok();
                    }
                }
            })
            .detach();
        });
        let mut picker = div().flex().gap_1().flex_wrap();
        for tool in ToolId::ALL {
            picker = picker.child(segment(t, SharedString::from(format!("add-{}", tool.raw())), tool.display_name(), add_tool == tool, cx.listener(move |this, _, _, cx| {
                this.state.update(cx, |s, cx| { s.add_tool = tool; cx.notify(); });
            })));
        }
        page.child(
            card(t)
                .child(div().text_base().font_weight(gpui::FontWeight::SEMIBOLD).child("Add an account"))
                .child(secondary(t, "Pick the tool, then the folder that account keeps its records in. The folder name becomes its label."))
                .child(picker)
                .child(div().flex().child(button(t, "choose-folder", "Choose data folder…", choose)))
                .child(secondary(t, "Meter reads only usage numbers from that folder. It never reads credentials or messages. Hidden accounts leave the menu, notes and alerts; their history is kept.")),
        )
    }
}

// ---------------------------------------------------------------- settings

impl RootView {
    fn setting_toggle(
        &self,
        t: &Theme,
        id: &'static str,
        on: bool,
        f: fn(&mut crate::settings::Settings),
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        toggle(t, id, on, cx.listener(move |this, _, _, cx| {
            this.state.update(cx, |st, cx| st.update_settings(f, cx));
        }))
        .into_any_element()
    }

    fn kind_row(
        &self,
        t: &Theme,
        s: &crate::settings::Settings,
        kind: AlertKind,
        title: &'static str,
        detail: String,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let on = s.alert_policy.enabled_kinds.contains(kind.raw());
        row(t, title, Some(detail), toggle(t, SharedString::from(format!("al-{}", kind.raw())), on, cx.listener(move |this, _, _, cx| {
            this.state.update(cx, |st, cx| st.update_settings(|x| {
                let k = kind.raw().to_string();
                if !x.alert_policy.enabled_kinds.remove(&k) {
                    x.alert_policy.enabled_kinds.insert(k);
                }
            }, cx));
        })))
        .into_any_element()
    }

    fn stepper(
        &self,
        t: &Theme,
        id: &'static str,
        value: String,
        minus: fn(&mut crate::settings::Settings),
        plus: fn(&mut crate::settings::Settings),
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(button(t, SharedString::from(format!("{id}-")), "−", cx.listener(move |this, _, _, cx| {
                this.state.update(cx, |st, cx| st.update_settings(minus, cx));
            })))
            .child(div().w(px(56.)).flex().justify_center().child(value))
            .child(button(t, SharedString::from(format!("{id}+")), "+", cx.listener(move |this, _, _, cx| {
                this.state.update(cx, |st, cx| st.update_settings(plus, cx));
            })))
            .into_any_element()
    }

    fn settings(&self, t: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let header = self.header(t, "Settings", cx);
        let (s, launch_error) = {
            let st = self.state.read(cx);
            (st.settings.clone(), st.launch_error.clone())
        };
        let general = card(t)
            .child(div().text_base().font_weight(gpui::FontWeight::SEMIBOLD).child("General"))
            .child(row(t, "Show percent in the menu bar", None, self.setting_toggle(t, "set-percent", s.show_percent_in_menu_bar, |x| x.show_percent_in_menu_bar = !x.show_percent_in_menu_bar, cx)))
            .child(row(
                t,
                "Refresh every",
                None,
                div().flex().gap_1().children([(30u64, "30 s"), (60, "1 min"), (300, "5 min")].map(|(n, label)| {
                    segment(t, SharedString::from(format!("rf-{n}")), label, s.refresh_seconds == n, cx.listener(move |this, _, _, cx| {
                        this.state.update(cx, |st, cx| st.update_settings(|x| x.refresh_seconds = n, cx));
                    }))
                })),
            ))
            .child(row(t, "Open Meter at login", launch_error, self.setting_toggle(t, "set-login", s.launch_at_login, |x| x.launch_at_login = !x.launch_at_login, cx)))
            .child(secondary(t, "Meter keeps running in the menu bar when its window is closed. Quit it from the menu bar item or the Dock."));

        let quiet_on = s.alert_policy.quiet_start_minute.is_some();
        let hhmm = |m: Option<u32>| m.map(|m| format!("{:02}:{:02}", m / 60, m % 60)).unwrap_or_default();
        let mut alerts = card(t)
            .child(div().text_base().font_weight(gpui::FontWeight::SEMIBOLD).child("Alerts"))
            .child(secondary(t, "All alerts are off until you turn one on. macOS asks for notification permission at that moment."))
            .child(self.kind_row(t, &s, AlertKind::Threshold, "A limit is filling up", format!("At {:.0}% and {:.0}% of a window.", s.threshold_low, s.threshold_high), cx))
            .child(self.kind_row(t, &s, AlertKind::Exhausted, "A limit is used up", "The window reaches 100%.".into(), cx))
            .child(self.kind_row(t, &s, AlertKind::Restored, "A limit has reset", "Room is available again after being used up.".into(), cx))
            .child(self.kind_row(t, &s, AlertKind::Pace, "I may run out early", "At the current pace a limit runs out before it resets.".into(), cx))
            .child(row(t, "First threshold", None, self.stepper(t, "th1", format!("{:.0}%", s.threshold_low),
                |x| x.threshold_low = (x.threshold_low - 5.0).max(50.0),
                |x| x.threshold_low = (x.threshold_low + 5.0).min(90.0), cx)))
            .child(row(t, "Second threshold", None, self.stepper(t, "th2", format!("{:.0}%", s.threshold_high),
                |x| x.threshold_high = (x.threshold_high - 1.0).max(55.0),
                |x| x.threshold_high = (x.threshold_high + 1.0).min(99.0), cx)))
            .child(row(t, "Hold alerts during quiet hours", None, toggle(t, "quiet", quiet_on, cx.listener(move |this, _, _, cx| {
                this.state.update(cx, |st, cx| st.update_settings(|x| {
                    if x.alert_policy.quiet_start_minute.is_some() {
                        x.alert_policy.quiet_start_minute = None;
                        x.alert_policy.quiet_end_minute = None;
                    } else {
                        x.alert_policy.quiet_start_minute = Some(22 * 60);
                        x.alert_policy.quiet_end_minute = Some(8 * 60);
                    }
                }, cx));
            }))));
        if quiet_on {
            alerts = alerts
                .child(row(t, "From", None, self.stepper(t, "qs", hhmm(s.alert_policy.quiet_start_minute),
                    |x| x.alert_policy.quiet_start_minute = x.alert_policy.quiet_start_minute.map(|m| (m + 24 * 60 - 60) % (24 * 60)),
                    |x| x.alert_policy.quiet_start_minute = x.alert_policy.quiet_start_minute.map(|m| (m + 60) % (24 * 60)), cx)))
                .child(row(t, "Until", None, self.stepper(t, "qe", hhmm(s.alert_policy.quiet_end_minute),
                    |x| x.alert_policy.quiet_end_minute = x.alert_policy.quiet_end_minute.map(|m| (m + 24 * 60 - 60) % (24 * 60)),
                    |x| x.alert_policy.quiet_end_minute = x.alert_policy.quiet_end_minute.map(|m| (m + 60) % (24 * 60)), cx)));
        }
        let snoozed = s.alert_policy.snooze_until.filter(|u| *u > chrono::Utc::now());
        alerts = alerts.child(row(
            t,
            "Snooze",
            snoozed.map(|u| format!("Snoozed until {}", u.with_timezone(&Local).format("%-I:%M %p"))),
            div()
                .flex()
                .gap_1()
                .child(button(t, "sn1", "1 hour", cx.listener(|this, _, _, cx| {
                    this.state.update(cx, |st, cx| st.update_settings(|x| x.alert_policy.snooze_until = Some(chrono::Utc::now() + Duration::hours(1)), cx));
                })))
                .child(button(t, "sn8", "8 hours", cx.listener(|this, _, _, cx| {
                    this.state.update(cx, |st, cx| st.update_settings(|x| x.alert_policy.snooze_until = Some(chrono::Utc::now() + Duration::hours(8)), cx));
                })))
                .child(button(t, "sn0", "Off", cx.listener(|this, _, _, cx| {
                    this.state.update(cx, |st, cx| st.update_settings(|x| x.alert_policy.snooze_until = None, cx));
                }))),
        ));
        div().flex().flex_col().gap_4().child(header).child(general).child(alerts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::RecordingNotifier;
    use chrono::Utc;
    use gpui::{TestAppContext, point, size};
    use meter_core::accounts::Account;
    use std::rc::Rc;

    /// Accounts pointing at an invented home folder, refreshed through the real readers.
    fn sample(home: &std::path::Path) -> (Vec<Account>, std::collections::HashMap<String, meter_core::AccountRun>, Vec<meter_core::DayUsage>, chrono::DateTime<Utc>) {
        meter_core::fakehome::build(home, Utc::now()).unwrap();
        let mut accounts: Vec<Account> = ToolId::ALL
            .iter()
            .map(|t| {
                let root = match t {
                    ToolId::Codex => ".codex",
                    ToolId::Grok => ".grok",
                    ToolId::ClaudeCode => ".claude",
                    ToolId::Cursor => "Library/Application Support/Cursor",
                    ToolId::Agy => ".gemini/antigravity-cli",
                };
                Account { data_root: Some(home.join(root).to_string_lossy().to_string()), ..Account::default_for(*t) }
            })
            .collect();
        accounts.push(Account { id: "work".into(), tool: ToolId::Codex, label: "Work".into(), data_root: Some("/nonexistent".into()), is_hidden: false });
        let now = Utc::now();
        let runs = meter_core::dashboard::refresh(&accounts, &Default::default(), now, now - chrono::Duration::days(30));
        let tz = chrono::FixedOffset::east_opt(0).unwrap();
        let sessions: Vec<_> = runs.values().flat_map(|r| r.output.sessions.clone()).collect();
        let mut days = meter_core::rollup::days_from(&sessions, tz);
        // A couple of older days so the chart has history.
        for i in 1..6 {
            let mut d = meter_core::DayUsage::new(&meter_core::rollup::day_key(now - chrono::Duration::days(i), tz), ToolId::Codex, "default", "/Users/demo/shop", "gpt-demo");
            d.input_tokens = 200_000 + i * 50_000;
            days.push(d);
        }
        (accounts, runs, days, now)
    }

    #[gpui::test]
    fn every_page_lays_out_and_paints_with_data_and_without(cx: &mut TestAppContext) {
        let support = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        let state = cx.new(|_| AppState::new(support.path().to_path_buf(), Some(Rc::new(RecordingNotifier::default()))));
        let (_view, vcx) = cx.add_window_view(|window, cx| RootView::new(state.clone(), window, cx));
        vcx.simulate_resize(size(gpui::px(1000.), gpui::px(680.)));

        let draw_all = |vcx: &mut gpui::VisualTestContext| {
            for page in [Page::Overview, Page::Usage, Page::Accounts, Page::Settings] {
                state.update(vcx, |s, cx| {
                    s.page = page;
                    cx.notify();
                });
                vcx.run_until_parked();
                vcx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
            }
        };
        // Empty: nothing refreshed yet.
        draw_all(vcx);
        // With data from the invented home, a hidden account, an ended window and quiet hours on.
        let (accounts, runs, days, now) = sample(home.path());
        state.update(vcx, |s, _| s.load_for_test(accounts, runs, days, now));
        state.update(vcx, |s, cx| {
            s.update_settings(|x| {
                x.alert_policy.quiet_start_minute = Some(22 * 60);
                x.alert_policy.quiet_end_minute = Some(8 * 60);
                x.alert_policy.enabled_kinds.insert("pace".into());
            }, cx)
        });
        draw_all(vcx);
        state.update(vcx, |s, cx| {
            s.usage_metric = Metric::Turns;
            s.usage_range_days = 30;
            cx.notify();
        });
        vcx.run_until_parked();
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let _ = point(gpui::px(0.), gpui::px(0.));
        assert!(!state.read_with(vcx, |s, _| s.dashboard.accounts.is_empty()));
        assert!(state.read_with(vcx, |s, _| s.dashboard.menu.tightest_percent.is_some()));
    }
}
