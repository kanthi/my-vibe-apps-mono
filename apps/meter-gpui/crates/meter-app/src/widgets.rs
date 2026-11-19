use crate::theme::Theme;
use gpui::{div, prelude::*, px, App, ClickEvent, ElementId, IntoElement, Rgba, SharedString, Window};

pub fn card(t: &Theme) -> gpui::Div {
    div().flex().flex_col().gap_2().p_4().min_w_0().rounded_lg().bg(t.card).border_1().border_color(t.line)
}

pub fn heading(text: impl Into<SharedString>) -> gpui::Div {
    div().text_xl().font_weight(gpui::FontWeight::SEMIBOLD).child(text.into())
}

pub fn secondary(t: &Theme, text: impl Into<SharedString>) -> gpui::Div {
    div().text_xs().text_color(t.secondary).child(text.into())
}

/// A plain capsule bar, so color does not depend on window focus. A word always sits beside it.
pub fn usage_bar(t: &Theme, percent: f64, color: Rgba) -> gpui::Div {
    let p = percent.clamp(0.0, 100.0) as f32;
    div()
        .h(px(8.))
        .w_full()
        .rounded_full()
        .bg(t.track)
        .child(div().h_full().w(gpui::relative((p / 100.0).max(0.03))).rounded_full().bg(color))
}

pub fn button(
    t: &Theme,
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let hover = t.line;
    div()
        .id(id)
        .px_3()
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(t.line)
        .bg(t.card)
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .on_click(on_click)
        .child(label.into())
}

/// One choice of a segmented row. The selected one is filled.
pub fn segment(
    t: &Theme,
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    selected: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let (bg, fg) = if selected { (t.accent, gpui::white()) } else { (t.card, t.text.into()) };
    let hover = t.line;
    div()
        .id(id)
        .px_3()
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(if selected { t.accent } else { t.line })
        .bg(bg)
        .text_color(fg)
        .cursor_pointer()
        .when(!selected, move |s| s.hover(move |s| s.bg(hover)))
        .on_click(on_click)
        .child(label.into())
}

/// An on/off switch. The state is also shown as text by the caller where it matters.
pub fn toggle(
    t: &Theme,
    id: impl Into<ElementId>,
    on: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .w(px(40.))
        .h(px(22.))
        .rounded_full()
        .bg(if on { t.accent } else { t.track })
        .border_1()
        .border_color(t.line)
        .cursor_pointer()
        .flex()
        .items_center()
        .when(on, |d| d.justify_end())
        .px(px(2.))
        .on_click(on_click)
        .child(div().size(px(16.)).rounded_full().bg(gpui::white()))
}

/// A settings row: title and detail on the left, a control on the right.
pub fn row(t: &Theme, title: impl Into<SharedString>, detail: Option<String>, control: impl IntoElement) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .py_2()
        .child(
            div()
                .flex()
                .flex_col()
                .child(div().child(title.into()))
                .children(detail.map(|d| secondary(t, d))),
        )
        .child(control)
}
