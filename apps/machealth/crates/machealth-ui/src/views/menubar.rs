use crate::components::*;
use crate::format::{format_bytes_si, format_memory_gib};
use crate::state::AppState;
use crate::theme::Theme;
use gpui::{
    Context, Entity, FocusHandle, FontWeight, IntoElement, KeyDownEvent, Render, SharedString,
    Subscription, Window, div, prelude::*,
};
use machealth_core::ActivityTab;
use std::rc::Rc;

/// Compact Menu Bar Panel & Quick Search view opened by NSStatusItem click or Ctrl+Opt+Space.
pub struct MenuBarPopoverView {
    state: Entity<AppState>,
    focus_handle: FocusHandle,
    on_open_main: Rc<dyn Fn(&mut Window, &mut gpui::App)>,
    _subs: Vec<Subscription>,
}

impl MenuBarPopoverView {
    pub fn new(
        state: Entity<AppState>,
        on_open_main: impl Fn(&mut Window, &mut gpui::App) + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle);
        let subs = vec![
            cx.observe(&state, |_, _, cx| cx.notify()),
            cx.observe_window_appearance(window, |_, _, cx| cx.notify()),
        ];
        Self {
            state,
            focus_handle,
            on_open_main: Rc::new(on_open_main),
            _subs: subs,
        }
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let has_cmd = event.keystroke.modifiers.platform || event.keystroke.modifiers.control;
        if has_cmd {
            return;
        }
        match key {
            "backspace" => {
                self.state.update(cx, |s, cx| {
                    s.search_query.pop();
                    s.run_quick_search(cx);
                });
            }
            "enter" => {
                self.state.update(cx, |s, cx| {
                    if let Some(first) = s.search_results.first().cloned() {
                        s.open_search_result(&first.path, false, cx);
                    } else {
                        s.run_quick_search(cx);
                    }
                });
            }
            "escape" => {
                self.state.update(cx, |s, cx| {
                    s.search_query.clear();
                    s.search_results.clear();
                    cx.notify();
                });
            }
            _ => {
                if let Some(ref ch) = event.keystroke.key_char {
                    if !ch.chars().any(|c| c.is_control()) {
                        self.state.update(cx, |s, cx| {
                            s.search_query.push_str(ch);
                            s.run_quick_search(cx);
                        });
                    }
                }
            }
        }
    }
}

impl Render for MenuBarPopoverView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = Theme::for_appearance(window.appearance());
        let state = self.state.read(cx);

        let (score_text, verdict_text, score_fg, score_bg) = match &state.report {
            Some(r) => {
                let (fg, bg) = t.grade_colors(r.grade);
                (format!("{} / 100", r.overall_score), r.verdict.clone(), fg, bg)
            }
            None => ("…".into(), "Checking Mac health…".into(), t.secondary, t.track),
        };

        let vitals_strip = match &state.snapshot {
            Some(snap) => div()
                .w_full()
                .flex()
                .gap_2()
                .child(
                    card(&t)
                        .flex_1()
                        .p_2()
                        .gap_1()
                        .child(secondary(&t, "CPU"))
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .child(format!("{:.0}%", snap.cpu.total_percent)),
                        ),
                )
                .child(
                    card(&t)
                        .flex_1()
                        .p_2()
                        .gap_1()
                        .child(secondary(&t, "Memory"))
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .child(format_memory_gib(snap.memory.used_bytes)),
                        ),
                )
                .child(
                    card(&t)
                        .flex_1()
                        .p_2()
                        .gap_1()
                        .child(secondary(&t, "Heat"))
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .child(snap.thermal.label.clone()),
                        ),
                ),
            None => div(),
        };

        // Top 3 busy apps
        let mut top3 = card(&t).p_3().gap_1().child(secondary(&t, "Busiest Right Now"));
        if let Some(ref procs) = state.processes {
            for g in procs.top_groups(ActivityTab::Cpu, 3) {
                top3 = top3.child(
                    div()
                        .w_full()
                        .flex()
                        .justify_between()
                        .text_xs()
                        .child(div().font_weight(FontWeight::MEDIUM).child(g.display_name))
                        .child(secondary(
                            &t,
                            format!("{:.1}% CPU · {}", g.cpu_percent, format_memory_gib(g.memory_bytes)),
                        )),
                );
            }
        }

        // Quick Search (Spotlight mdfind)
        let query_display = if state.search_query.is_empty() {
            "Type to Quick Search files (Spotlight)…".to_string()
        } else {
            format!("{}|", state.search_query)
        };
        let query_is_empty = state.search_query.is_empty();
        let mut search_box = card(&t)
            .p_3()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(secondary(&t, "Quick Search"))
                    .child(secondary(&t, "Ctrl+Opt+Space")),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(t.card_elevated)
                    .border_1()
                    .border_color(t.accent)
                    .text_xs()
                    .text_color(if query_is_empty { t.secondary } else { t.text })
                    .child(query_display),
            );

        if let Some(ref err) = state.search_error {
            search_box = search_box.child(div().text_xs().text_color(t.red).child(err.clone()));
        } else {
            for (idx, res) in state.search_results.iter().take(6).enumerate() {
                let p_open = res.path.clone();
                let p_reveal = res.path.clone();
                let st_open = self.state.clone();
                let st_reveal = self.state.clone();
                search_box = search_box.child(
                    div()
                        .w_full()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .py_1()
                        .border_t_1()
                        .border_color(t.line)
                        .child(
                            div()
                                .id(SharedString::from(format!("qs-open-{idx}")))
                                .flex()
                                .flex_col()
                                .overflow_hidden()
                                .cursor_pointer()
                                .on_click(move |_, _, cx| {
                                    let p = p_open.clone();
                                    st_open.update(cx, |s, cx| s.open_search_result(&p, false, cx));
                                })
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(res.name.clone()),
                                )
                                .child(secondary(
                                    &t,
                                    format!("{} · {}", res.parent_display, format_bytes_si(res.size_bytes)),
                                )),
                        )
                        .child(ghost_button(
                            &t,
                            SharedString::from(format!("qs-reveal-{idx}")),
                            "Finder",
                            move |_, _, cx| {
                                let p = p_reveal.clone();
                                st_reveal.update(cx, |s, cx| s.open_search_result(&p, true, cx));
                            },
                        )),
                );
            }
        }

        let st_bal = self.state.clone();
        let st_undo = self.state.clone();
        let has_undo = state.undo_ledger.has_pending_undo();
        let open_main_cb = self.on_open_main.clone();
        let open_main_for_bal = self.on_open_main.clone();

        div()
            .id("menubar-popover-root")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .bg(t.window)
            .text_color(t.text)
            .text_sm()
            .overflow_y_scroll()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(badge(score_text, score_fg, score_bg))
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("MacHealth"),
                            ),
                    )
                    .child(ghost_button(
                        &t,
                        "pop-open-main",
                        "Open Window",
                        move |_, window, cx| (open_main_cb)(window, cx),
                    )),
            )
            .child(secondary(&t, verdict_text))
            .child(vitals_strip)
            .child(top3)
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(primary_button(
                        &t,
                        "pop-balance",
                        "Balance the Load",
                        move |_, window, cx| {
                            st_bal.update(cx, |s, cx| s.open_balance_preview(cx));
                            (open_main_for_bal)(window, cx);
                        },
                    ))
                    .when(has_undo, |d| {
                        d.child(ghost_button(&t, "pop-undo", "Undo Balance", move |_, _, cx| {
                            st_undo.update(cx, |s, cx| s.perform_undo_balance(cx));
                        }))
                    }),
            )
            .child(search_box)
    }
}
