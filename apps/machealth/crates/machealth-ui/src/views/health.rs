use crate::components::*;
use crate::format::{format_bytes_si, format_duration_short, format_memory_gib, format_rate};
use crate::state::AppState;
use crate::theme::Theme;
use gpui::{Context, Entity, FontWeight, IntoElement, SharedString, div, prelude::*, px};
use machealth_core::telemetry::memory::MemoryPressure;
use machealth_core::telemetry::thermal::ThermalLevel;
use machealth_core::{ActivityTab, FindingAction, HistoryRange};

pub fn render_health_view<V: 'static>(
    state_entity: &Entity<AppState>,
    t: &Theme,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let state = state_entity.read(cx);
    let Some(ref snapshot) = state.snapshot else {
        return div()
            .w_full()
            .flex()
            .flex_col()
            .gap_3()
            .child(page_heading("Health"))
            .child(card(t).child(secondary(t, "Gathering macOS telemetry…")))
            .into_any_element();
    };
    let Some(ref report) = state.report else {
        return div().into_any_element();
    };

    let (grade_fg, grade_bg) = t.grade_colors(report.grade);
    let findings_expanded = state.findings_expanded;
    let top_tab = state.dashboard_top_tab;
    let history_range = state.history_range;

    // 1. Score & Findings hero card
    let toggle_state = state_entity.clone();
    let mut hero = card(t).child(
        div()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .gap_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .flex_none()
                            .w(px(68.))
                            .h(px(68.))
                            .rounded_full()
                            .bg(grade_bg)
                            .border_2()
                            .border_color(grade_fg)
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(grade_fg)
                                    .child(format!("{}", report.overall_score)),
                            )
                            .child(div().text_xs().text_color(grade_fg).child("/ 100")),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .child(report.verdict.clone()),
                            )
                            .child(secondary(
                                t,
                                format!(
                                    "Weighted across 8 macOS health areas · updated {}",
                                    snapshot.captured_at.format("%H:%M:%S UTC")
                                ),
                            )),
                    ),
            )
            .when(!report.findings.is_empty(), |d| {
                d.child(ghost_button(
                    t,
                    "toggle-findings",
                    if findings_expanded {
                        "Hide Findings"
                    } else {
                        "Show Findings"
                    },
                    move |_, _, cx| {
                        toggle_state.update(cx, |s, cx| {
                            s.findings_expanded = !s.findings_expanded;
                            cx.notify();
                        });
                    },
                ))
            }),
    );

    if findings_expanded && !report.findings.is_empty() {
        let mut findings_list = div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .pt_2()
            .border_t_1()
            .border_color(t.line);
        for (idx, finding) in report.findings.iter().enumerate() {
            let (fg, bg) = t.severity_colors(finding.severity);
            let action = finding.action.clone();
            let action_label: String = match &action {
                FindingAction::BalanceLoad => "Balance the Load".into(),
                FindingAction::OpenActivityView => "Inspect Activity".into(),
                FindingAction::OpenCleanUpView => "Review Clean Up".into(),
                FindingAction::OpenSecurityView => "Review Security".into(),
                FindingAction::InspectSleepAssertions => "Inspect Sleep".into(),
                FindingAction::OpenSystemSettings { label, .. } => label.clone(),
            };
            let st = state_entity.clone();
            findings_list = findings_list.child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_4()
                    .px_3()
                    .py_2()
                    .rounded_lg()
                    .bg(t.card_elevated)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .overflow_hidden()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(badge(format!("{:?}", finding.severity), fg, bg))
                                    .child(
                                        div()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(finding.title.clone()),
                                    ),
                            )
                            .child(secondary(t, finding.detail.clone())),
                    )
                    .child(ghost_button(
                        t,
                        SharedString::from(format!("finding-act-{idx}")),
                        action_label,
                        move |_, _, cx| {
                            let act = action.clone();
                            st.update(cx, |s, cx| s.trigger_finding_action(act, cx));
                        },
                    )),
            );
        }
        hero = hero.child(findings_list);
    }

    // 2. Live Vitals Cards ("Right Now") — 2 rows of 3 uniform cards
    let cpu = &snapshot.cpu;
    let pe_text = match (cpu.performance_percent, cpu.efficiency_percent) {
        (Some(p), Some(e)) => format!(
            "P-cores ({}): {:.0}%  ·  E-cores ({}): {:.0}%",
            cpu.topology.performance_cores, p, cpu.topology.efficiency_cores, e
        ),
        _ => format!("{} logical cores", cpu.topology.logical_cores),
    };
    let cpu_color = if cpu.total_percent >= 75.0 {
        t.red
    } else if cpu.total_percent >= 40.0 {
        t.amber
    } else {
        t.green
    };
    let cpu_card = card(t)
        .flex_1()
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child(section_title("Processor"))
                .child(badge(cpu.load_level.label(), cpu_color, t.card_elevated)),
        )
        .child(
            div()
                .text_2xl()
                .font_weight(FontWeight::BOLD)
                .child(format!("{:.1}%", cpu.total_percent)),
        )
        .child(progress_bar(t, cpu.total_percent, cpu_color))
        .child(secondary(t, pe_text))
        .child(secondary(t, cpu.summary.clone()));

    let heat_color = match snapshot.thermal.level {
        ThermalLevel::Normal => t.green,
        ThermalLevel::Warm => t.amber,
        ThermalLevel::Hot | ThermalLevel::VeryHot => t.red,
    };
    let batt_temp_line = snapshot
        .thermal
        .battery_temp_celsius
        .map(|c| format!("Battery temp: {c:.1}°C"))
        .unwrap_or_else(|| {
            if snapshot.battery.has_battery {
                format!(
                    "Charge: {}%  ·  Capacity: {}%",
                    snapshot.battery.charge_percent.unwrap_or(0),
                    snapshot.battery.health_percent.unwrap_or(0)
                )
            } else {
                "Wall power (Desktop Mac)".into()
            }
        });
    let heat_card = card(t)
        .flex_1()
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child(section_title("Heat & Battery"))
                .child(badge(snapshot.thermal.label.clone(), heat_color, t.card_elevated)),
        )
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::BOLD)
                .child(snapshot.thermal.summary.clone()),
        )
        .child(secondary(t, batt_temp_line))
        .child(secondary(t, snapshot.battery.summary.clone()));

    let mem = &snapshot.memory;
    let mem_color = match mem.pressure {
        MemoryPressure::Normal => t.green,
        MemoryPressure::Warning => t.amber,
        MemoryPressure::Critical => t.red,
    };
    let mem_card = card(t)
        .flex_1()
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child(section_title("Memory"))
                .child(badge(mem.pressure.label(), mem_color, t.card_elevated)),
        )
        .child(
            div()
                .text_2xl()
                .font_weight(FontWeight::BOLD)
                .child(format!(
                    "{} / {}",
                    format_memory_gib(mem.used_bytes),
                    format_memory_gib(mem.total_bytes)
                )),
        )
        .child(progress_bar(t, mem.used_percent, mem_color))
        .child(secondary(
            t,
            format!(
                "Apps {} · Wired {} · Compressed {} · Swap {}",
                format_memory_gib(mem.app_bytes),
                format_memory_gib(mem.wired_bytes),
                format_memory_gib(mem.compressed_bytes),
                format_memory_gib(mem.swap_used_bytes)
            ),
        ));

    let stg = &snapshot.storage;
    let stg_color = if stg.free_percent < 10.0 {
        t.red
    } else if stg.free_percent < 20.0 {
        t.amber
    } else {
        t.accent
    };
    let storage_card = card(t)
        .flex_1()
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child(section_title("Storage"))
                .child(badge(
                    format!("SMART {}", stg.smart_status.label()),
                    t.green,
                    t.card_elevated,
                )),
        )
        .child(
            div()
                .text_2xl()
                .font_weight(FontWeight::BOLD)
                .child(format!("{} free", format_bytes_si(stg.available_bytes))),
        )
        .child(progress_bar(t, stg.used_percent, stg_color))
        .child(secondary(
            t,
            format!(
                "{} of {} used · {} purgeable",
                format_bytes_si(stg.used_bytes),
                format_bytes_si(stg.total_bytes),
                format_bytes_si(stg.purgeable_bytes)
            ),
        ));

    let net = &snapshot.network;
    let net_card = card(t)
        .flex_1()
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child(section_title("Network"))
                .child(secondary(
                    t,
                    if net.active_interfaces.is_empty() {
                        "Offline".into()
                    } else {
                        net.active_interfaces.join(", ")
                    },
                )),
        )
        .child(
            div()
                .flex()
                .gap_6()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(secondary(t, "Down"))
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .child(format_rate(net.rx_bytes_per_sec)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(secondary(t, "Up"))
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .child(format_rate(net.tx_bytes_per_sec)),
                        ),
                ),
        )
        .child(secondary(
            t,
            format!(
                "Session total: ↓{} · ↑{}",
                format_bytes_si(net.total_rx_bytes),
                format_bytes_si(net.total_tx_bytes)
            ),
        ));

    let sleep = &snapshot.sleep;
    let user_blockers: Vec<_> = sleep
        .active_assertions
        .iter()
        .filter(|a| a.process_name != "powerd")
        .take(2)
        .collect();
    let sleep_card = card(t)
        .flex_1()
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child(section_title("Sleep & Wake"))
                .child(secondary(
                    t,
                    format!(
                        "Display {}m · Sleep {}m",
                        sleep.active_timers.display_sleep_minutes.unwrap_or(0),
                        sleep.active_timers.system_sleep_minutes.unwrap_or(0)
                    ),
                )),
        )
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::SEMIBOLD)
                .child(sleep.summary.clone()),
        )
        .children(user_blockers.into_iter().map(|a| {
            secondary(
                t,
                format!(
                    "• {} ({})",
                    a.process_name,
                    format_duration_short(a.duration_seconds)
                ),
            )
        }));

    // 3. 8-Area Score Grid (2 rows × 4 cards)
    let area_cards: Vec<_> = report
        .areas
        .iter()
        .map(|a| {
            let (fg, bg) = t.severity_colors(a.status);
            card(t)
                .flex_1()
                .p_3()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(div().text_xs().font_weight(FontWeight::SEMIBOLD).child(a.title.clone()))
                        .child(badge(format!("{}", a.score), fg, bg)),
                )
                .child(progress_bar(t, a.score as f32, fg))
                .child(secondary(t, a.summary.clone()))
        })
        .collect();

    let mut areas_row1 = div().w_full().flex().gap_3();
    let mut areas_row2 = div().w_full().flex().gap_3();
    for (i, c) in area_cards.into_iter().enumerate() {
        if i < 4 {
            areas_row1 = areas_row1.child(c);
        } else {
            areas_row2 = areas_row2.child(c);
        }
    }

    // 4. "Using the Most" Top 5 Apps Card
    let mut top_tabs = div().flex().gap_1();
    for tab in [
        ActivityTab::Cpu,
        ActivityTab::Memory,
        ActivityTab::Energy,
        ActivityTab::Disk,
    ] {
        let st = state_entity.clone();
        top_tabs = top_tabs.child(segment_button(
            t,
            SharedString::from(format!("dash-top-{}", tab.label())),
            tab.label(),
            top_tab == tab,
            move |_, _, cx| {
                st.update(cx, |s, cx| {
                    s.dashboard_top_tab = tab;
                    cx.notify();
                });
            },
        ));
    }

    let mut top_card = card(t).child(
        div()
            .flex()
            .items_center()
            .justify_between()
            .child(section_title("Using the Most"))
            .child(top_tabs),
    );
    if let Some(ref proc_snap) = state.processes {
        for group in proc_snap.top_groups(top_tab, 5) {
            let (fg, bg) = t.impact_colors(group.impact_label);
            let metric_str = match top_tab {
                ActivityTab::Cpu => format!("{:.1}% CPU", group.cpu_percent),
                ActivityTab::Memory => format_memory_gib(group.memory_bytes),
                ActivityTab::Energy => format!("{:.1} Energy", group.energy_impact),
                ActivityTab::Disk => format!(
                    "{} R/W",
                    format_bytes_si(group.disk_read_bytes + group.disk_write_bytes)
                ),
                ActivityTab::Network => format_bytes_si(group.net_rx_bytes + group.net_tx_bytes),
            };
            top_card = top_card.child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .py(px(6.))
                    .border_t_1()
                    .border_color(t.line)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().font_weight(FontWeight::MEDIUM).child(group.display_name))
                            .when(group.helper_count > 0, |d| {
                                d.child(secondary(t, format!("+{} helpers", group.helper_count)))
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(badge(group.impact_label.label(), fg, bg))
                            .child(
                                div()
                                    .w(px(95.))
                                    .text_right()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(metric_str),
                            ),
                    ),
            );
        }
    }

    // 5. History (Hour / Day / Week / Month)
    let mut range_tabs = div().flex().gap_1();
    for range in [
        HistoryRange::Hour,
        HistoryRange::Day,
        HistoryRange::Week,
        HistoryRange::Month,
    ] {
        let st = state_entity.clone();
        range_tabs = range_tabs.child(segment_button(
            t,
            SharedString::from(format!("hist-{}", range.label())),
            range.label(),
            history_range == range,
            move |_, _, cx| {
                st.update(cx, |s, cx| s.set_history_range(range, cx));
            },
        ));
    }
    let cpu_pts: Vec<f32> = state
        .history_series
        .points
        .iter()
        .map(|p| p.cpu_percent)
        .collect();
    let mem_pts: Vec<f32> = state
        .history_series
        .points
        .iter()
        .map(|p| p.memory_used_percent)
        .collect();

    let history_card = card(t)
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(section_title("History"))
                .child(range_tabs),
        )
        .child(secondary(t, state.history_series.summary.clone()))
        .child(
            div()
                .w_full()
                .flex()
                .gap_4()
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(secondary(t, "Processor (%)"))
                        .child(history_bars(t, &cpu_pts, t.accent)),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(secondary(t, "Memory (%)"))
                        .child(history_bars(t, &mem_pts, t.green)),
                ),
        );

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_4()
        .child(hero)
        .child(section_title("Right Now"))
        .child(div().w_full().flex().gap_3().child(cpu_card).child(heat_card).child(mem_card))
        .child(
            div()
                .w_full()
                .flex()
                .gap_3()
                .child(storage_card)
                .child(net_card)
                .child(sleep_card),
        )
        .child(section_title("8 Health Areas"))
        .child(areas_row1)
        .child(areas_row2)
        .child(top_card)
        .child(history_card)
        .into_any_element()
}
