use crate::components::*;
use crate::format::{format_bytes_si, format_memory_gib, format_rate};
use crate::state::AppState;
use crate::theme::Theme;
use gpui::{Context, Entity, FontWeight, IntoElement, SharedString, div, prelude::*, px};
use machealth_core::ActivityTab;

pub fn render_activity_view<V: 'static>(
    state_entity: &Entity<AppState>,
    t: &Theme,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let state = state_entity.read(cx);
    let active_tab = state.activity_tab;

    let mut tabs = div().flex().gap_1();
    for tab in [
        ActivityTab::Cpu,
        ActivityTab::Memory,
        ActivityTab::Energy,
        ActivityTab::Disk,
        ActivityTab::Network,
    ] {
        let st = state_entity.clone();
        tabs = tabs.child(segment_button(
            t,
            SharedString::from(format!("act-tab-{}", tab.label())),
            tab.label(),
            active_tab == tab,
            move |_, _, cx| {
                st.update(cx, |s, cx| {
                    s.activity_tab = tab;
                    cx.notify();
                });
            },
        ));
    }

    let root = div().w_full().flex().flex_col().gap_4().child(
        div()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(page_heading("Activity"))
                    .child(secondary(
                        t,
                        "Apps rolled up with their helper processes · click any app row to inspect helpers",
                    )),
            )
            .child(tabs),
    );

    let Some(ref proc_snap) = state.processes else {
        return root
            .child(card(t).child(secondary(t, "Sampling running processes…")))
            .into_any_element();
    };

    let groups = proc_snap.top_groups(active_tab, 40);

    // macOS Activity Monitor-style table header
    let header_row = div()
        .w_full()
        .px_2()
        .pb_2()
        .flex()
        .items_center()
        .justify_between()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(t.secondary)
        .child(div().child("Application"))
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(div().w(px(72.)).text_right().child("Impact"))
                .child(div().w(px(110.)).text_right().child(active_tab.label()))
                .child(div().w(px(80.)).text_right().child("Memory"))
                .child(div().w(px(116.)).text_right().child("Actions")),
        );

    let mut table = card(t).gap_0().child(header_row);

    for (idx, group) in groups.into_iter().enumerate() {
        let is_expanded = state.expanded_groups.contains(&group.id);
        let (fg, bg) = t.impact_colors(group.impact_label);

        let primary_metric = match active_tab {
            ActivityTab::Cpu => format!("{:.1}%", group.cpu_percent),
            ActivityTab::Memory => format_memory_gib(group.memory_bytes),
            ActivityTab::Energy => format!("{:.1}", group.energy_impact),
            ActivityTab::Disk => format!(
                "{} / {}",
                format_bytes_si(group.disk_read_bytes),
                format_bytes_si(group.disk_write_bytes)
            ),
            ActivityTab::Network => format!(
                "↓{} ↑{}",
                format_rate(group.net_rx_bytes_per_sec),
                format_rate(group.net_tx_bytes_per_sec)
            ),
        };

        let group_id = group.id.clone();
        let st_expand = state_entity.clone();
        let st_quit = state_entity.clone();
        let st_force = state_entity.clone();
        let group_for_quit = group.clone();
        let group_for_force = group.clone();
        let hover_bg = t.row_hover;

        // Important: avoid `.min_w_0()` on inner text columns so Taffy never measures text at 0px width.
        let mut row_block = div()
            .w_full()
            .flex()
            .flex_col()
            .border_t_1()
            .border_color(t.line)
            .child(
                div()
                    .w_full()
                    .px_2()
                    .py_2()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .hover(move |s| s.bg(hover_bg))
                    .child(
                        div()
                            .id(SharedString::from(format!("expand-grp-{idx}")))
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .overflow_hidden()
                            .cursor_pointer()
                            .on_click(move |_, _, cx| {
                                let id = group_id.clone();
                                st_expand.update(cx, |s, cx| {
                                    if !s.expanded_groups.remove(&id) {
                                        s.expanded_groups.insert(id);
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
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(group.display_name.clone()),
                                    )
                                    .when(group.helper_count > 0, |d| {
                                        d.child(secondary(
                                            t,
                                            format!(
                                                "{} {} helpers",
                                                if is_expanded { "▾" } else { "▸" },
                                                group.helper_count
                                            ),
                                        ))
                                    }),
                            )
                            .child(secondary(t, group.description.clone())),
                    )
                    .child(
                        div()
                            .flex_none()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .w(px(72.))
                                    .flex()
                                    .justify_end()
                                    .child(badge(group.impact_label.label(), fg, bg)),
                            )
                            .child(
                                div()
                                    .w(px(110.))
                                    .text_right()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(primary_metric),
                            )
                            .child(
                                div()
                                    .w(px(80.))
                                    .text_right()
                                    .text_xs()
                                    .text_color(t.secondary)
                                    .child(format_memory_gib(group.memory_bytes)),
                            )
                            .child(
                                div()
                                    .w(px(116.))
                                    .flex()
                                    .justify_end()
                                    .gap_1()
                                    .child(ghost_button(
                                        t,
                                        SharedString::from(format!("quit-grp-{idx}")),
                                        "Quit",
                                        move |_, _, cx| {
                                            let g = group_for_quit.clone();
                                            st_quit.update(cx, |s, cx| {
                                                s.request_quit_group(g, false, cx)
                                            });
                                        },
                                    ))
                                    .child(danger_button(
                                        t,
                                        SharedString::from(format!("fquit-grp-{idx}")),
                                        "Force",
                                        move |_, _, cx| {
                                            let g = group_for_force.clone();
                                            st_force.update(cx, |s, cx| {
                                                s.request_quit_group(g, true, cx)
                                            });
                                        },
                                    )),
                            ),
                    ),
            );

        if is_expanded {
            let mut helpers = div()
                .w_full()
                .flex()
                .flex_col()
                .gap_1()
                .px_4()
                .py_2()
                .mb_2()
                .rounded_md()
                .bg(t.card_elevated);
            for p in group.processes.iter().take(16) {
                helpers = helpers.child(
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
                                .child(div().text_color(t.text).child(p.name.clone()))
                                .child(secondary(t, format!("PID {}", p.pid))),
                        )
                        .child(secondary(
                            t,
                            format!(
                                "{:.1}% CPU  ·  {}",
                                p.cpu_percent,
                                format_memory_gib(p.memory_bytes)
                            ),
                        )),
                );
            }
            row_block = row_block.child(helpers);
        }

        table = table.child(row_block);
    }

    root.child(table).into_any_element()
}
