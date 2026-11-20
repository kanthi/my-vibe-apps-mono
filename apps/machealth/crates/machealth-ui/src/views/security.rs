use crate::components::*;
use crate::state::AppState;
use crate::theme::Theme;
use gpui::{Context, Entity, FontWeight, IntoElement, SharedString, div, prelude::*, px};
use machealth_core::open_system_settings;

pub fn render_security_view<V: 'static>(
    state_entity: &Entity<AppState>,
    t: &Theme,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let state = state_entity.read(cx);
    let mut root = div().w_full().flex().flex_col().gap_4().child(
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(page_heading("Security & Maintenance"))
            .child(secondary(
                t,
                "Live non-privileged checks with 1-click System Settings links",
            )),
    );

    let Some(ref audit) = state.security else {
        return root
            .child(card(t).child(secondary(t, "Running security audits…")))
            .into_any_element();
    };

    root = root.child(
        card(t).child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(audit.overall_summary.clone()),
                )
                .child(badge(
                    format!("{} / {} Passed", audit.pass_count, audit.checks.len()),
                    if audit.attention_count == 0 {
                        t.green
                    } else {
                        t.amber
                    },
                    if audit.attention_count == 0 {
                        t.green_bg
                    } else {
                        t.amber_bg
                    },
                )),
        ),
    );

    let mut list = card(t).gap_1();
    for (idx, item) in audit.checks.iter().enumerate() {
        let (fg, bg) = t.audit_colors(item.status);
        let url = item.settings_url.clone();
        let allow_ext = state.config.allow_external_actions;
        let st = state_entity.clone();

        list = list.child(
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .gap_4()
                .py_3()
                .when(idx > 0, |d| d.border_t_1().border_color(t.line))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .overflow_hidden()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(item.title.clone()),
                                )
                                .child(badge(item.status.label(), fg, bg)),
                        )
                        .child(div().text_sm().child(item.summary.clone()))
                        .child(secondary(t, item.detail.clone())),
                )
                .child(ghost_button(
                    t,
                    SharedString::from(format!("sec-open-{idx}")),
                    item.settings_label.clone(),
                    move |_, _, cx| {
                        if !allow_ext {
                            return;
                        }
                        if let Err(err) = open_system_settings(&url) {
                            st.update(cx, |s, cx| {
                                s.set_toast(err, true);
                                cx.notify();
                            });
                        }
                    },
                )),
        );
    }
    root = root.child(list);

    // User LaunchAgents list
    let mut agents_card = card(t).child(section_title(format!(
        "User Startup Items ({})",
        audit.startup_items.total_count
    )));
    if audit.startup_items.user_launch_agents.is_empty() {
        agents_card = agents_card.child(secondary(
            t,
            "No user LaunchAgents found in ~/Library/LaunchAgents.",
        ));
    } else {
        let mut grid = div().w_full().flex().flex_col().gap_1();
        for label in &audit.startup_items.user_launch_agents {
            grid = grid.child(
                div()
                    .px_3()
                    .py(px(6.))
                    .rounded_md()
                    .bg(t.card_elevated)
                    .text_xs()
                    .child(format!("•  {label}")),
            );
        }
        agents_card = agents_card.child(grid);
    }

    root.child(agents_card).into_any_element()
}
