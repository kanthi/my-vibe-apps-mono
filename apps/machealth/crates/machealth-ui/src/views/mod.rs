pub mod activity;
pub mod agents;
pub mod cleanup;
pub mod health;
pub mod menubar;
pub mod modals;
pub mod security;

use crate::components::*;
use crate::state::{AppState, NavPage};
use crate::theme::Theme;
use gpui::{
    AnyElement, Context, Entity, FontWeight, IntoElement, Render, SharedString, Subscription,
    Window, div, prelude::*, px,
};
use std::rc::Rc;

pub use menubar::MenuBarPopoverView;

pub struct RootView {
    state: Entity<AppState>,
    on_refresh: Rc<dyn Fn(&mut Window, &mut gpui::App)>,
    _subs: Vec<Subscription>,
}

impl RootView {
    pub fn new(
        state: Entity<AppState>,
        on_refresh: impl Fn(&mut Window, &mut gpui::App) + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let subs = vec![
            cx.observe(&state, |_, _, cx| cx.notify()),
            cx.observe_window_appearance(window, |_, _, cx| cx.notify()),
        ];
        Self {
            state,
            on_refresh: Rc::new(on_refresh),
            _subs: subs,
        }
    }

    fn nav_item(
        &self,
        t: &Theme,
        page: NavPage,
        glyph: &'static str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = self.state.read(cx).active_page == page;
        let hover = t.row_hover;
        let icon_color = if selected {
            t.selected_text
        } else {
            t.accent
        };
        let text_color = if selected { t.selected_text } else { t.text };

        div()
            .id(SharedString::from(format!("nav-{}", page.label())))
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .py(px(7.))
            .rounded_lg()
            .text_color(text_color)
            .cursor_pointer()
            .when(selected, |d| {
                d.bg(t.selected).font_weight(FontWeight::SEMIBOLD)
            })
            .when(!selected, move |d| d.hover(move |s| s.bg(hover)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.state.update(cx, |s, cx| {
                    s.active_page = page;
                    cx.notify();
                });
            }))
            .child(div().w(px(18.)).text_color(icon_color).child(glyph))
            .child(page.label())
            .into_any_element()
    }
}

impl Render for RootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = Theme::for_appearance(window.appearance());
        let (page, has_undo, toast, score_pill) = {
            let s = self.state.read(cx);
            let pill = s.report.as_ref().map(|r| {
                let (fg, bg) = t.grade_colors(r.grade);
                (r.overall_score, r.verdict.clone(), fg, bg)
            });
            (
                s.active_page,
                s.undo_ledger.has_pending_undo(),
                s.toast.clone(),
                pill,
            )
        };

        let body: AnyElement = match page {
            NavPage::Health => health::render_health_view(&self.state, &t, cx).into_any_element(),
            NavPage::Activity => {
                activity::render_activity_view(&self.state, &t, cx).into_any_element()
            }
            NavPage::Agents => agents::render_agents_view(&self.state, &t, cx).into_any_element(),
            NavPage::Security => {
                security::render_security_view(&self.state, &t, cx).into_any_element()
            }
            NavPage::CleanUp => {
                cleanup::render_cleanup_view(&self.state, &t, cx).into_any_element()
            }
        };

        let st_balance = self.state.clone();
        let st_undo = self.state.clone();
        let st_dismiss = self.state.clone();
        let refresh_cb = self.on_refresh.clone();

        // macOS HIG Unified Toolbar header
        let top_bar = div()
            .h(px(52.))
            .w_full()
            .flex_none()
            .px_6()
            .bg(t.toolbar)
            .border_b_1()
            .border_color(t.line)
            .flex()
            .items_center()
            .justify_between()
            .gap_4()
            .child(match score_pill {
                Some((score, verdict, fg, bg)) => div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(badge(format!("● {score} / 100"), fg, bg))
                    .child(div().font_weight(FontWeight::MEDIUM).child(verdict)),
                None => div().child(secondary(&t, "Reading macOS telemetry…")),
            })
            .child(
                div()
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap_2()
                    .when(has_undo, |d| {
                        d.child(ghost_button(
                            &t,
                            "topbar-undo",
                            "Undo Balance",
                            move |_, _, cx| {
                                st_undo.update(cx, |s, cx| s.perform_undo_balance(cx));
                            },
                        ))
                    })
                    .child(ghost_button(
                        &t,
                        "topbar-refresh",
                        "Refresh",
                        move |_, window, cx| (refresh_cb)(window, cx),
                    ))
                    .child(primary_button(
                        &t,
                        "topbar-balance",
                        "Balance the Load",
                        move |_, _, cx| {
                            st_balance.update(cx, |s, cx| s.open_balance_preview(cx));
                        },
                    )),
            );

        let toast_bar = toast.map(|(msg, is_err)| {
            let (fg, bg) = if is_err {
                (t.red, t.red_bg)
            } else {
                (t.green, t.green_bg)
            };
            div()
                .mx_6()
                .mt_3()
                .px_3()
                .py_2()
                .rounded_lg()
                .bg(bg)
                .border_1()
                .border_color(fg)
                .flex()
                .items_center()
                .justify_between()
                .child(div().text_xs().text_color(fg).child(msg))
                .child(
                    div()
                        .id("dismiss-toast")
                        .text_xs()
                        .text_color(fg)
                        .cursor_pointer()
                        .on_click(move |_, _, cx| {
                            st_dismiss.update(cx, |s, cx| {
                                s.toast = None;
                                cx.notify();
                            });
                        })
                        .child("✕"),
                )
        });

        // macOS System Settings style Sidebar
        let sidebar = div()
            .w(px(215.))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .justify_between()
            .p_3()
            .bg(t.sidebar)
            .border_r_1()
            .border_color(t.line)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .mb_1()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .child("MacHealth"),
                            )
                            .child(secondary(&t, "macOS Vitals & Care")),
                    )
                    .child(self.nav_item(&t, NavPage::Health, "♥", cx))
                    .child(self.nav_item(&t, NavPage::Activity, "▤", cx))
                    .child(self.nav_item(&t, NavPage::Agents, "◈", cx))
                    .child(self.nav_item(&t, NavPage::Security, "⛨", cx))
                    .child(self.nav_item(&t, NavPage::CleanUp, "◫", cx)),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .rounded_lg()
                    .bg(t.card)
                    .border_1()
                    .border_color(t.line)
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(div().text_xs().font_weight(FontWeight::MEDIUM).child("Local-First & Safe"))
                    .child(secondary(&t, "Quick Panel: Ctrl+Opt+Space")),
            );

        let modal = modals::render_modal_overlay(&self.state, &t, cx);

        div()
            .relative()
            .size_full()
            .flex()
            .bg(t.window)
            .text_color(t.text)
            .text_sm()
            .child(sidebar)
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(top_bar)
                    .children(toast_bar)
                    .child(
                        div()
                            .id("main-scroll-pane")
                            .flex_1()
                            .w_full()
                            .overflow_y_scroll()
                            .p_6()
                            .child(body),
                    ),
            )
            .children(modal)
    }
}
