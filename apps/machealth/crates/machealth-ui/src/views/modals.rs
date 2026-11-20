use crate::components::*;
use crate::format::format_memory_gib;
use crate::state::{AppState, PendingModal};
use crate::theme::Theme;
use gpui::{Context, Entity, FontWeight, IntoElement, SharedString, div, prelude::*, px, rgba};

pub fn render_modal_overlay<V: 'static>(
    state_entity: &Entity<AppState>,
    t: &Theme,
    cx: &mut Context<V>,
) -> Option<impl IntoElement> {
    let modal = state_entity.read(cx).pending_modal.clone()?;
    let st_cancel = state_entity.clone();

    let content = match modal {
        PendingModal::ConfirmQuitGroup { confirmation, .. } => {
            let st_ok = state_entity.clone();
            dialog_shell(t, &confirmation.title, &confirmation.message)
                .child(
                    div()
                        .flex()
                        .justify_end()
                        .gap_2()
                        .pt_2()
                        .child(ghost_button(t, "modal-cancel", "Cancel", move |_, _, cx| {
                            st_cancel.update(cx, |s, cx| {
                                s.pending_modal = None;
                                cx.notify();
                            });
                        }))
                        .child(danger_button(
                            t,
                            "modal-confirm-quit",
                            if confirmation.force {
                                "Force Quit"
                            } else {
                                "Quit App"
                            },
                            move |_, _, cx| {
                                st_ok.update(cx, |s, cx| s.confirm_quit_group(cx));
                            },
                        )),
                )
        }
        PendingModal::ConfirmEndAgent { confirmation, .. } => {
            let st_ok = state_entity.clone();
            dialog_shell(t, &confirmation.title, &confirmation.message)
                .child(
                    div()
                        .flex()
                        .justify_end()
                        .gap_2()
                        .pt_2()
                        .child(ghost_button(t, "modal-cancel", "Cancel", move |_, _, cx| {
                            st_cancel.update(cx, |s, cx| {
                                s.pending_modal = None;
                                cx.notify();
                            });
                        }))
                        .child(danger_button(
                            t,
                            "modal-confirm-agent",
                            "End Agent Session",
                            move |_, _, cx| {
                                st_ok.update(cx, |s, cx| s.confirm_end_agent(cx));
                            },
                        )),
                )
        }
        PendingModal::BalancePreview {
            plan,
            selected_idle_pids,
        } => {
            let st_apply = state_entity.clone();
            let mut body = dialog_shell(t, "Balance the Load", &plan.summary);

            if !plan.throttle_targets.is_empty() {
                body = body.child(secondary(t, "Will move to Apple Silicon Efficiency cores (Undoable):"));
                for item in &plan.throttle_targets {
                    body = body.child(
                        div()
                            .flex()
                            .justify_between()
                            .text_xs()
                            .p_2()
                            .rounded_md()
                            .bg(t.card_elevated)
                            .child(div().font_weight(FontWeight::MEDIUM).child(item.display_name.clone()))
                            .child(secondary(t, format!("{:.0}% CPU · PID {}", item.cpu_percent, item.pid))),
                    );
                }
            }

            if !plan.idle_quit_candidates.is_empty() {
                body = body.child(secondary(t, "Optional — click to also quit long-idle apps:"));
                for (idx, item) in plan.idle_quit_candidates.iter().enumerate() {
                    let pid = item.pid;
                    let checked = selected_idle_pids.contains(&pid);
                    let st_tog = state_entity.clone();
                    body = body.child(
                        div()
                            .id(SharedString::from(format!("bal-idle-{idx}")))
                            .flex()
                            .items_center()
                            .justify_between()
                            .text_xs()
                            .p_2()
                            .rounded_md()
                            .bg(if checked { t.selected } else { t.card_elevated })
                            .cursor_pointer()
                            .on_click(move |_, _, cx| {
                                st_tog.update(cx, |s, cx| {
                                    if let Some(PendingModal::BalancePreview {
                                        selected_idle_pids,
                                        ..
                                    }) = s.pending_modal.as_mut()
                                    {
                                        if !selected_idle_pids.remove(&pid) {
                                            selected_idle_pids.insert(pid);
                                        }
                                        cx.notify();
                                    }
                                });
                            })
                            .child(
                                div()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(format!("{} {}", if checked { "☑" } else { "☐" }, item.display_name)),
                            )
                            .child(secondary(t, format_memory_gib(item.memory_bytes))),
                    );
                }
            }

            body.child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .pt_2()
                    .child(ghost_button(t, "modal-cancel", "Cancel", move |_, _, cx| {
                        st_cancel.update(cx, |s, cx| {
                            s.pending_modal = None;
                            cx.notify();
                        });
                    }))
                    .when(!plan.is_empty(), |d| {
                        d.child(primary_button(
                            t,
                            "modal-apply-balance",
                            "Apply Balance",
                            move |_, _, cx| {
                                st_apply.update(cx, |s, cx| s.confirm_balance_plan(cx));
                            },
                        ))
                    }),
            )
        }
        PendingModal::BalanceOutcome { result } => {
            let st_undo = state_entity.clone();
            dialog_shell(t, "Load Balanced — Before & After", &result.summary)
                .child(
                    div()
                        .flex()
                        .gap_3()
                        .child(
                            div()
                                .flex_1()
                                .p_2()
                                .rounded_md()
                                .bg(t.card_elevated)
                                .child(secondary(t, "Before"))
                                .child(div().font_weight(FontWeight::BOLD).child(format!(
                                    "{:.1}% CPU · {}",
                                    result.before.cpu_percent,
                                    format_memory_gib(result.before.memory_used_bytes)
                                ))),
                        )
                        .child(
                            div()
                                .flex_1()
                                .p_2()
                                .rounded_md()
                                .bg(t.card_elevated)
                                .child(secondary(t, "After"))
                                .child(div().font_weight(FontWeight::BOLD).child(format!(
                                    "{:.1}% CPU ({:+.1}%) · {}",
                                    result.after.cpu_percent,
                                    result.cpu_delta_percent,
                                    format_memory_gib(result.after.memory_used_bytes)
                                ))),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .justify_end()
                        .gap_2()
                        .pt_2()
                        .when(!result.throttled.is_empty(), |d| {
                            d.child(ghost_button(t, "modal-undo-now", "Undo Now", move |_, _, cx| {
                                st_undo.update(cx, |s, cx| {
                                    s.pending_modal = None;
                                    s.perform_undo_balance(cx);
                                });
                            }))
                        })
                        .child(primary_button(t, "modal-done", "Done", move |_, _, cx| {
                            st_cancel.update(cx, |s, cx| {
                                s.pending_modal = None;
                                cx.notify();
                            });
                        })),
                )
        }
        PendingModal::ConfirmCleanUp { confirmation, .. } => {
            let st_ok = state_entity.clone();
            let preview_names = confirmation
                .target_names
                .iter()
                .take(6)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            dialog_shell(t, &confirmation.title, &confirmation.message)
                .child(secondary(t, format!("Items: {preview_names}")))
                .child(
                    div()
                        .flex()
                        .justify_end()
                        .gap_2()
                        .pt_2()
                        .child(ghost_button(t, "modal-cancel", "Cancel", move |_, _, cx| {
                            st_cancel.update(cx, |s, cx| {
                                s.pending_modal = None;
                                cx.notify();
                            });
                        }))
                        .child(danger_button(
                            t,
                            "modal-confirm-trash",
                            "Move to Trash",
                            move |_, _, cx| {
                                st_ok.update(cx, |s, cx| s.confirm_cleanup_trash(cx));
                            },
                        )),
                )
        }
    };

    Some(
        div()
            .absolute()
            .inset_0()
            .bg(rgba(0x00000088))
            .flex()
            .items_center()
            .justify_center()
            .p_6()
            .child(content),
    )
}

fn dialog_shell(t: &Theme, title: &str, message: &str) -> gpui::Div {
    div()
        .w(px(460.))
        .max_w_full()
        .flex()
        .flex_col()
        .gap_3()
        .p_5()
        .rounded_xl()
        .bg(t.card)
        .border_1()
        .border_color(t.line)
        .child(div().text_base().font_weight(FontWeight::BOLD).child(title.to_string()))
        .child(div().text_sm().text_color(t.secondary).child(message.to_string()))
}
