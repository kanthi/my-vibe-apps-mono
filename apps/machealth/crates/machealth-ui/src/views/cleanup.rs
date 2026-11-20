use crate::components::*;
use crate::format::{format_bytes_si, shorten_home_path};
use crate::state::AppState;
use crate::theme::Theme;
use gpui::{Context, Entity, FontWeight, IntoElement, SharedString, div, prelude::*, px};

pub fn render_cleanup_view<V: 'static>(
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
            .child(page_heading("Home & Clean Up"))
            .child(secondary(
                t,
                "Inspect storage breakdown by file kind and move rebuildable caches or old downloads to macOS Trash",
            )),
    );

    // 1. Home — Files by Kind (macOS System Settings -> General -> Storage segmented bar + 2-column grid)
    if let Some(ref home) = state.home_files {
        let total = home.total_bytes.max(1) as f32;

        // Multi-segment horizontal storage bar matching macOS System Settings -> Storage
        let mut storage_bar = div()
            .h(px(14.))
            .w_full()
            .rounded_full()
            .bg(t.track)
            .overflow_hidden()
            .flex()
            .gap(px(1.));

        for bucket in &home.buckets {
            if bucket.total_bytes == 0 {
                continue;
            }
            let share = (bucket.total_bytes as f32 / total).clamp(0.015, 1.0);
            storage_bar = storage_bar.child(
                div()
                    .h_full()
                    .w(gpui::relative(share))
                    .bg(t.file_kind_color(bucket.kind)),
            );
        }

        // 2-column legend rows
        let non_empty: Vec<_> = home.buckets.iter().filter(|b| b.file_count > 0).collect();
        let mut legend_grid = div().w_full().flex().flex_col().gap_2().pt_1();
        for pair in non_empty.chunks(2) {
            let mut row = div().w_full().flex().gap_4();
            for bucket in pair {
                let color = t.file_kind_color(bucket.kind);
                row = row.child(
                    div()
                        .flex_1()
                        .px_3()
                        .py_2()
                        .rounded_lg()
                        .bg(t.card_elevated)
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(color_dot(color))
                                .child(
                                    div()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(bucket.label.clone()),
                                )
                                .child(secondary(t, format!("{} files", bucket.file_count))),
                        )
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(format_bytes_si(bucket.total_bytes)),
                        ),
                );
            }
            if pair.len() == 1 {
                row = row.child(div().flex_1());
            }
            legend_grid = legend_grid.child(row);
        }

        let kind_card = card(t)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(section_title("Home — Files by Kind"))
                    .child(secondary(
                        t,
                        format!(
                            "{} files · {}",
                            home.total_files,
                            format_bytes_si(home.total_bytes)
                        ),
                    )),
            )
            .child(storage_bar)
            .child(legend_grid);

        root = root.child(kind_card);
    }

    // 2. Safe Clean Up candidates (macOS inset list with single-line truncated tilde paths)
    let Some(ref cleanup) = state.cleanup else {
        return root
            .child(
                card(t).child(secondary(
                    t,
                    "Scanning Downloads, rebuildable caches, and Xcode DerivedData…",
                )),
            )
            .into_any_element();
    };

    let selected_bytes: u64 = cleanup
        .candidates
        .iter()
        .filter(|c| state.selected_cleanup_paths.contains(&c.path))
        .map(|c| c.size_bytes)
        .sum();
    let selected_count = state.selected_cleanup_paths.len();

    let st_trash = state_entity.clone();
    let st_sel_all = state_entity.clone();
    let all_paths: Vec<_> = cleanup.candidates.iter().map(|c| c.path.clone()).collect();
    let all_selected = !all_paths.is_empty() && selected_count == all_paths.len();

    let mut clean_card = card(t).child(
        div()
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
            .pb_1()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(section_title("Safe Clean Up"))
                    .child(secondary(
                        t,
                        format!(
                            "{} · Trash currently holds {} items ({})",
                            cleanup.summary,
                            cleanup.trash_summary.item_count,
                            format_bytes_si(cleanup.trash_summary.total_bytes)
                        ),
                    )),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .when(!all_paths.is_empty(), |d| {
                        d.child(ghost_button(
                            t,
                            "cleanup-toggle-all",
                            if all_selected {
                                "Deselect All"
                            } else {
                                "Select All"
                            },
                            move |_, _, cx| {
                                let paths = all_paths.clone();
                                st_sel_all.update(cx, |s, cx| {
                                    if s.selected_cleanup_paths.len() == paths.len() {
                                        s.selected_cleanup_paths.clear();
                                    } else {
                                        s.selected_cleanup_paths = paths.into_iter().collect();
                                    }
                                    cx.notify();
                                });
                            },
                        ))
                    })
                    .child(primary_button(
                        t,
                        "cleanup-move-trash",
                        if selected_count > 0 {
                            format!(
                                "Move {} to Trash ({})…",
                                selected_count,
                                format_bytes_si(selected_bytes)
                            )
                        } else {
                            "Move Selected to Trash…".into()
                        },
                        move |_, _, cx| {
                            st_trash.update(cx, |s, cx| s.request_cleanup_trash(cx));
                        },
                    )),
            ),
    );

    if cleanup.candidates.is_empty() {
        clean_card = clean_card.child(secondary(
            t,
            "No old installers, forgotten large downloads, or large caches detected.",
        ));
    } else {
        let mut rows_list = div().w_full().flex().flex_col();
        for (idx, item) in cleanup.candidates.iter().take(40).enumerate() {
            let checked = state.selected_cleanup_paths.contains(&item.path);
            let path = item.path.clone();
            let display_path = shorten_home_path(&item.path, 68);
            let st_chk = state_entity.clone();
            let hover_bg = t.row_hover;

            // Crucial layout rule: give the row a predictable structure without `.min_w_0()`
            // on the inner text column so Taffy never measures text at 0px width.
            rows_list = rows_list.child(
                div()
                    .id(SharedString::from(format!("cleanup-row-{idx}")))
                    .w_full()
                    .px_2()
                    .py_2()
                    .rounded_md()
                    .border_t_1()
                    .border_color(t.line)
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_4()
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .on_click(move |_, _, cx| {
                        let p = path.clone();
                        st_chk.update(cx, |s, cx| {
                            if !s.selected_cleanup_paths.remove(&p) {
                                s.selected_cleanup_paths.insert(p);
                            }
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .overflow_hidden()
                            .child(
                                div()
                                    .flex_none()
                                    .w(px(18.))
                                    .h(px(18.))
                                    .rounded(px(5.))
                                    .border_1()
                                    .border_color(if checked { t.accent } else { t.secondary })
                                    .bg(if checked { t.accent } else { t.card_elevated })
                                    .text_color(gpui::white())
                                    .text_xs()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(if checked { "✓" } else { "" }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .child(item.name.clone()),
                                            )
                                            .child(badge(
                                                item.category.label(),
                                                t.accent,
                                                t.card_elevated,
                                            )),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(t.secondary)
                                            .child(format!("{display_path}  ·  {}", item.reason)),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex_none()
                            .w(px(90.))
                            .text_right()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format_bytes_si(item.size_bytes)),
                    ),
            );
        }
        clean_card = clean_card.child(rows_list);
    }

    root.child(clean_card).into_any_element()
}
