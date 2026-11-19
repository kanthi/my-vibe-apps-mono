//! Meter (GPUI edition): a Mac window plus a menu bar item that read the usage records AI tools
//! already keep on this Mac. The window is GPUI; the menu bar item, notifications and login item
//! are native AppKit calls.

mod autotest;
mod settings;
mod state;
mod statusbar;
mod system;
mod theme;
mod ui;
mod widgets;
mod window;

use futures::StreamExt;
use gpui::{
    actions, AppContext as _, px, size, App, Application, Bounds, Entity, Global, KeyBinding, Menu, MenuItem, SystemMenuType,
    TitlebarOptions, WindowBounds, WindowHandle, WindowOptions,
};
use state::{AppState, Page};
use statusbar::{MenuEvent, StatusBar};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use ui::RootView;

actions!(meter, [OpenMain, OpenSettings, RefreshNow, CloseWindow, QuitMeter]);

struct SharedState(Entity<AppState>);
impl Global for SharedState {}

struct MainWindow(Option<WindowHandle<RootView>>);
impl Global for MainWindow {}

/// Shows the main window, creating it if it was closed. Closing the window never quits the app.
fn open_main(cx: &mut App) {
    let handle = cx.try_global::<MainWindow>().and_then(|m| m.0);
    if let Some(h) = handle {
        if h.update(cx, |_, window, _| window.activate_window()).is_ok() {
            cx.activate(true);
            return;
        }
    }
    let state = cx.global::<SharedState>().0.clone();
    // Size from the screen the window opens on; fall back to a fixed size if no display is reported.
    let (w, h) = cx
        .primary_display()
        .map(|d| {
            let b = d.bounds();
            window::initial_size(f32::from(b.size.width), f32::from(b.size.height))
        })
        .unwrap_or((1000.0, 680.0));
    let (min_w, min_h) = window::min_size((w, h));
    let bounds = Bounds::centered(None, size(px(w), px(h)), cx);
    let handle = cx
        .open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(min_w), px(min_h))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Meter".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                ..Default::default()
            },
            move |window, cx| cx.new(|cx| RootView::new(state, window, cx)),
        )
        .ok();
    cx.set_global(MainWindow(handle));
    cx.activate(true);
}

fn open_page(page: Page, cx: &mut App) {
    cx.global::<SharedState>().0.clone().update(cx, |s, cx| {
        s.page = page;
        cx.notify();
    });
    open_main(cx);
}

fn refresh_now(cx: &mut App) {
    cx.global::<SharedState>().0.clone().update(cx, |s, cx| s.refresh(cx));
}

fn support_dir() -> PathBuf {
    // METER_SUPPORT_DIR lets a test run keep every file Meter writes inside a throwaway folder.
    std::env::var_os("METER_SUPPORT_DIR").map(PathBuf::from).unwrap_or_else(AppState::default_support_dir)
}

fn main() {
    let app = Application::new();
    // Clicking the Dock icon while no window is open brings the window back.
    app.on_reopen(|cx| open_main(cx));
    app.run(|cx: &mut App| {
        let state = cx.new(|_| AppState::new(support_dir(), None));
        cx.set_global(SharedState(state.clone()));

        // Closing the last window leaves Meter running in the menu bar: nothing quits on close.
        cx.on_action(|_: &OpenMain, cx| open_main(cx));
        cx.on_action(|_: &OpenSettings, cx| open_page(Page::Settings, cx));
        cx.on_action(|_: &RefreshNow, cx| refresh_now(cx));
        cx.on_action(|_: &QuitMeter, cx| cx.quit());
        cx.on_action(|_: &CloseWindow, cx| {
            if let Some(w) = cx.active_window() {
                w.update(cx, |_, window, _| window.remove_window()).ok();
            }
        });
        cx.bind_keys([
            KeyBinding::new("cmd-q", QuitMeter, None),
            KeyBinding::new("cmd-w", CloseWindow, None),
            KeyBinding::new("cmd-r", RefreshNow, None),
            KeyBinding::new("cmd-,", OpenSettings, None),
        ]);

        cx.set_menus(vec![
            Menu {
                name: "Meter".into(),
                items: vec![
                    MenuItem::action("Settings…", OpenSettings),
                    MenuItem::separator(),
                    MenuItem::os_submenu("Services", SystemMenuType::Services),
                    MenuItem::separator(),
                    MenuItem::action("Quit Meter", QuitMeter),
                ],
            },
            Menu { name: "View".into(), items: vec![MenuItem::action("Refresh Now", RefreshNow)] },
            Menu { name: "Window".into(), items: vec![MenuItem::action("Close Window", CloseWindow)] },
        ]);
        cx.set_dock_menu(vec![MenuItem::action("Open Meter", OpenMain), MenuItem::action("Refresh Now", RefreshNow)]);

        // The menu bar item: clicks arrive over a channel and are handled on GPUI's main thread.
        let (tx, mut rx) = futures::channel::mpsc::unbounded();
        if let Some(bar) = StatusBar::new(tx) {
            let bar = Rc::new(bar);
            bar.update(&state.read(cx).menu_view());
            let observed = bar.clone();
            cx.observe(&state, move |state, cx| observed.update(&state.read(cx).menu_view())).detach();
            if let Some(after) = autotest::env_ms("METER_AUTOTEST_OPEN_MENU_MS") {
                cx.spawn(async move |cx| {
                    cx.background_executor().timer(after).await;
                    cx.update(|_| bar.open_menu()).ok();
                })
                .detach();
            }
        }
        cx.spawn(async move |cx| {
            while let Some(event) = rx.next().await {
                cx.update(|cx| match event {
                    MenuEvent::OpenMain => open_main(cx),
                    MenuEvent::Refresh => refresh_now(cx),
                    MenuEvent::OpenSettings => open_page(Page::Settings, cx),
                    MenuEvent::Quit => cx.quit(),
                })
                .ok();
            }
        })
        .detach();

        // Refresh on a timer for as long as Meter runs, window or no window.
        let loop_state = state.clone();
        cx.spawn(async move |cx| loop {
            loop_state.update(cx, |s, cx| s.refresh(cx)).ok();
            let secs = loop_state.read_with(cx, |s, _| s.settings.refresh_seconds).unwrap_or(60).max(15);
            cx.background_executor().timer(Duration::from_secs(secs)).await;
        })
        .detach();

        if let Some(page) = autotest::start_page() {
            state.update(cx, |s, _| s.page = page);
        }
        open_main(cx);

        if let Some(after) = autotest::env_ms("METER_AUTOTEST_CLOSE_WINDOW_MS") {
            cx.spawn(async move |cx| {
                cx.background_executor().timer(after).await;
                cx.update(|cx| {
                    if let Some(w) = cx.active_window() {
                        w.update(cx, |_, window, _| window.remove_window()).ok();
                    }
                })
                .ok();
            })
            .detach();
        }
    });
}
