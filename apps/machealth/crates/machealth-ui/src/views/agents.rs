use crate::components::*;
use crate::format::{format_bytes_si, format_memory_gib};
use crate::state::AppState;
use crate::theme::Theme;
use gpui::{Context, Entity, FontWeight, IntoElement, SharedString, div, prelude::*, px};

pub fn render_agents_view<V: 'static>(
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
            .child(page_heading("AI Agents"))
            .child(secondary(
                t,
                "Tracks local AI apps, coding agents, MCP servers, and every child process they spawned",
            )),
    );

    let Some(ref agents) = state.agents else {
        return root
            .child(card(t).child(secondary(t, "Scanning local AI agent trees…")))
            .into_any_element();
    };

    // Totals summary bar
    root = root.child(
        div()
            .w_full()
            .flex()
            .gap_3()
            .child(
                card(t)
                    .flex_1()
                    .child(secondary(t, "Active Agent Sessions"))
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .child(format!("{}", agents.sessions.len())),
                    )
                    .child(secondary(
                        t,
                        format!("{} total processes (including children)", agents.total_process_count),
                    )),
            )
            .child(
                card(t)
                    .flex_1()
                    .child(secondary(t, "Combined CPU"))
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .child(format!("{:.1}%", agents.total_cpu_percent)),
                    )
                    .child(secondary(
                        t,
                        format!("Energy impact: {:.1}", agents.total_energy_impact),
                    )),
            )
            .child(
                card(t)
                    .flex_1()
                    .child(secondary(t, "Combined Memory"))
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .child(format_memory_gib(agents.total_memory_bytes)),
                    )
                    .child(secondary(t, "Rolled up across spawned process trees")),
            ),
    );

    if agents.sessions.is_empty() {
        return root
            .child(
                card(t).child(
                    div()
                        .py_4()
                        .text_center()
                        .text_color(t.secondary)
                        .child("No local AI agents or MCP servers are running right now."),
                ),
            )
            .into_any_element();
    }

    for (idx, session) in agents.sessions.iter().enumerate() {
        let (fg, bg) = t.impact_colors(session.impact_label);
        let is_expanded = state.expanded_agents.contains(&session.id);
        let session_id = session.id.clone();
        let st_expand = state_entity.clone();
        let st_end = state_entity.clone();
        let session_for_end = session.clone();

        let mut scard = card(t).child(
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .gap_4()
                .child(
                    div()
                        .id(SharedString::from(format!("agent-exp-{idx}")))
                        .flex()
                        .flex_col()
                        .gap_1()
                        .overflow_hidden()
                        .cursor_pointer()
                        .on_click(move |_, _, cx| {
                            let id = session_id.clone();
                            st_expand.update(cx, |s, cx| {
                                if !s.expanded_agents.remove(&id) {
                                    s.expanded_agents.insert(id);
                                }
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(session.display_name.clone()),
                                )
                                .child(badge(session.kind.label(), t.accent, t.card_elevated))
                                .child(badge(session.impact_label.label(), fg, bg))
                                .when(!session.spawned_children.is_empty(), |d| {
                                    d.child(secondary(
                                        t,
                                        format!(
                                            "{} {} children",
                                            if is_expanded { "▾" } else { "▸" },
                                            session.spawned_children.len()
                                        ),
                                    ))
                                }),
                        )
                        .child(secondary(
                            t,
                            format!(
                                "Root PID {}  ·  {}  ·  Disk R/W {}",
                                session.root_pid,
                                session.summary,
                                format_bytes_si(session.disk_read_bytes + session.disk_write_bytes)
                            ),
                        )),
                )
                .child(
                    div()
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .w(px(110.))
                                .flex()
                                .flex_col()
                                .items_end()
                                .child(
                                    div()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(format!("{:.1}% CPU", session.cpu_percent)),
                                )
                                .child(secondary(t, format_memory_gib(session.memory_bytes))),
                        )
                        .child(danger_button(
                            t,
                            SharedString::from(format!("end-agent-{idx}")),
                            "End Session",
                            move |_, _, cx| {
                                let sess = session_for_end.clone();
                                st_end.update(cx, |s, cx| s.request_end_agent(sess, false, cx));
                            },
                        )),
                ),
        );

        if is_expanded && !session.spawned_children.is_empty() {
            let mut kids = div()
                .w_full()
                .flex()
                .flex_col()
                .gap_1()
                .p_3()
                .rounded_lg()
                .bg(t.card_elevated);
            for child in session.spawned_children.iter().take(20) {
                kids = kids.child(
                    div()
                        .w_full()
                        .flex()
                        .items_center()
                        .justify_between()
                        .text_xs()
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .child(div().font_weight(FontWeight::MEDIUM).child(child.name.clone()))
                                .child(secondary(t, format!("PID {}", child.pid))),
                        )
                        .child(secondary(
                            t,
                            format!(
                                "{:.1}% CPU  ·  {}",
                                child.cpu_percent,
                                format_memory_gib(child.memory_bytes)
                            ),
                        )),
                );
            }
            scard = scard.child(kids);
        }

        root = root.child(scard);
    }

    root.into_any_element()
}
