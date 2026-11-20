use crate::theme::Theme;
use gpui::{
    App, ClickEvent, Div, ElementId, FontWeight, IntoElement, Rgba, SharedString, Window, div,
    prelude::*, px,
};

/// macOS System Settings inset grouped card container.
/// Note: never apply `.min_w_0()` to vertical card containers — in Taffy 0.9 / GPUI 0.2.2,
/// zero min-width causes multi-line text children inside `items_center` rows to measure at
/// 1-char width and offset vertically.
pub fn card(t: &Theme) -> Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_3()
        .p_4()
        .rounded_xl()
        .bg(t.card)
        .border_1()
        .border_color(t.line)
}

pub fn section_title(text: impl Into<SharedString>) -> Div {
    div()
        .text_sm()
        .font_weight(FontWeight::SEMIBOLD)
        .child(text.into())
}

pub fn page_heading(text: impl Into<SharedString>) -> Div {
    div()
        .text_xl()
        .font_weight(FontWeight::BOLD)
        .child(text.into())
}

pub fn secondary(t: &Theme, text: impl Into<SharedString>) -> Div {
    div().text_xs().text_color(t.secondary).child(text.into())
}

pub fn badge(label: impl Into<SharedString>, fg: Rgba, bg: Rgba) -> Div {
    div()
        .flex_none()
        .px_2()
        .py(px(2.))
        .rounded_md()
        .bg(bg)
        .text_color(fg)
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .child(label.into())
}

pub fn color_dot(color: Rgba) -> Div {
    div().flex_none().w(px(10.)).h(px(10.)).rounded_full().bg(color)
}

pub fn progress_bar(t: &Theme, percent: f32, color: Rgba) -> Div {
    let p = percent.clamp(0.0, 100.0);
    div()
        .h(px(6.))
        .w_full()
        .rounded_full()
        .bg(t.track)
        .overflow_hidden()
        .child(
            div()
                .h_full()
                .w(gpui::relative((p / 100.0).max(0.02)))
                .rounded_full()
                .bg(color),
        )
}

pub fn primary_button(
    t: &Theme,
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let accent = t.accent;
    div()
        .id(id)
        .flex_none()
        .px_3()
        .py(px(5.))
        .rounded_md()
        .bg(accent)
        .text_color(gpui::white())
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer()
        .hover(move |s| s.opacity(0.9))
        .on_click(on_click)
        .child(label.into())
}

pub fn danger_button(
    t: &Theme,
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let red = t.red;
    let red_bg = t.red_bg;
    div()
        .id(id)
        .flex_none()
        .px(px(10.))
        .py(px(4.))
        .rounded_md()
        .border_1()
        .border_color(red)
        .bg(red_bg)
        .text_color(red)
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer()
        .hover(move |s| s.bg(red).text_color(gpui::white()))
        .on_click(on_click)
        .child(label.into())
}

pub fn ghost_button(
    t: &Theme,
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let hover = t.row_hover;
    div()
        .id(id)
        .flex_none()
        .px(px(10.))
        .py(px(4.))
        .rounded_md()
        .border_1()
        .border_color(t.line)
        .bg(t.card_elevated)
        .text_color(t.text)
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .on_click(on_click)
        .child(label.into())
}

pub fn segment_button(
    t: &Theme,
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    selected: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let (bg, fg, border) = if selected {
        (t.accent, gpui::white(), t.accent)
    } else {
        (t.card_elevated, t.text.into(), t.line)
    };
    let hover = t.row_hover;
    div()
        .id(id)
        .flex_none()
        .px_3()
        .py(px(4.))
        .rounded_md()
        .border_1()
        .border_color(border)
        .bg(bg)
        .text_color(fg)
        .text_xs()
        .font_weight(if selected {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        })
        .cursor_pointer()
        .when(!selected, move |d| d.hover(move |s| s.bg(hover)))
        .on_click(on_click)
        .child(label.into())
}

/// Compact bar chart used for CPU & Memory history series.
pub fn history_bars(t: &Theme, values: &[f32], color: Rgba) -> Div {
    if values.is_empty() {
        return div()
            .h(px(64.))
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .bg(t.card_elevated)
            .child(secondary(t, "Collecting samples…"));
    }

    div()
        .h(px(68.))
        .w_full()
        .px_2()
        .py_1()
        .rounded_md()
        .bg(t.card_elevated)
        .flex()
        .items_end()
        .gap(px(2.))
        .children(values.iter().map(|&v| {
            let h = (v.clamp(0.0, 100.0) / 100.0 * 56.0).max(3.0);
            div().flex_1().h(px(h)).rounded_t(px(2.)).bg(color)
        }))
}
