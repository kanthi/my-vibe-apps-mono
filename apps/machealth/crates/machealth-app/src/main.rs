mod hotkey;
mod menubar;
mod platform;

use futures::StreamExt;
use gpui::{
    App, AppContext as _, Application, Bounds, Entity, Global, KeyBinding, Menu, MenuItem, Point,
    SystemMenuType, TitlebarOptions, WindowBounds, WindowHandle, WindowKind, WindowOptions,
    actions, px, size,
};
use hotkey::GlobalHotKey;
use machealth_core::{ProcessSampler, TelemetryCollector};
use machealth_ui::{
    AppState, MenuBarPopoverView, RootView, collect_fast_refresh, collect_slow_refresh,
};
use menubar::{MenuBarEvent, StatusBarController};
use platform::LaunchOptions;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

actions!(
    machealth,
    [OpenMain, TogglePopover, RefreshNow, CloseWindow, QuitApp]
);

struct SharedState(Entity<AppState>);
impl Global for SharedState {}

struct SharedSamplers {
    telemetry: Arc<Mutex<TelemetryCollector>>,
    processes: Arc<Mutex<ProcessSampler>>,
}
impl Global for SharedSamplers {}

struct MainWindow(Option<WindowHandle<RootView>>);
impl Global for MainWindow {}

struct PopoverWindow(Option<WindowHandle<MenuBarPopoverView>>);
impl Global for PopoverWindow {}

fn open_main(cx: &mut App) {
    if let Some(handle) = cx.try_global::<MainWindow>().and_then(|m| m.0) {
        if handle.update(cx, |_, window, _| window.activate_window()).is_ok() {
            cx.activate(true);
            return;
        }
    }

    let state = cx.global::<SharedState>().0.clone();
    // Size proportionally to the display while leaving room for the macOS Menu Bar and Dock.
    let (w, h) = cx
        .primary_display()
        .map(|d| {
            let b = d.bounds();
            let dw = f32::from(b.size.width);
            let dh = f32::from(b.size.height);
            let target_w = (dw * 0.78).clamp(960.0, 1260.0).min((dw - 40.0).max(860.0));
            let target_h = (dh * 0.78).clamp(640.0, 840.0).min((dh - 140.0).max(560.0));
            (target_w, target_h)
        })
        .unwrap_or((1100.0, 740.0));
    let bounds = Bounds::centered(None, size(px(w), px(h)), cx);
    let handle = cx
        .open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(880.), px(580.))),
                titlebar: Some(TitlebarOptions {
                    title: Some("MacHealth".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                ..Default::default()
            },
            move |window, cx| {
                cx.new(|cx| {
                    RootView::new(
                        state,
                        |_, cx| trigger_manual_refresh(cx),
                        window,
                        cx,
                    )
                })
            },
        )
        .ok();
    cx.set_global(MainWindow(handle));
    cx.activate(true);
}

fn toggle_popover(cx: &mut App) {
    if let Some(handle) = cx.try_global::<PopoverWindow>().and_then(|p| p.0) {
        if handle.update(cx, |_, window, _| window.remove_window()).is_ok() {
            cx.set_global(PopoverWindow(None));
            return;
        }
    }

    let state = cx.global::<SharedState>().0.clone();
    let pop_w = px(390.);
    let pop_h = px(560.);

    // Position near top-right (under the menu bar) if display bounds are available.
    let bounds = cx
        .primary_display()
        .map(|d| {
            let b = d.bounds();
            let x = (b.origin.x + b.size.width - pop_w - px(16.)).max(px(16.));
            let y = b.origin.y + px(36.);
            Bounds::new(Point { x, y }, size(pop_w, pop_h))
        })
        .unwrap_or_else(|| Bounds::centered(None, size(pop_w, pop_h), cx));

    let handle = cx
        .open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(360.), px(460.))),
                kind: WindowKind::PopUp,
                titlebar: Some(TitlebarOptions {
                    title: Some("MacHealth Quick Panel".into()),
                    appears_transparent: true,
                    traffic_light_position: None,
                }),
                ..Default::default()
            },
            move |window, cx| {
                cx.new(|cx| {
                    MenuBarPopoverView::new(
                        state,
                        |window, cx| {
                            window.remove_window();
                            cx.set_global(PopoverWindow(None));
                            open_main(cx);
                        },
                        window,
                        cx,
                    )
                })
            },
        )
        .ok();
    cx.set_global(PopoverWindow(handle));
    cx.activate(true);
}

fn trigger_manual_refresh(cx: &mut App) {
    let state = cx.global::<SharedState>().0.clone();
    let telemetry = cx.global::<SharedSamplers>().telemetry.clone();
    let processes = cx.global::<SharedSamplers>().processes.clone();
    let include_net = state.read(cx).activity_tab == machealth_core::ActivityTab::Network;

    cx.spawn(async move |cx| {
        let fast = cx
            .background_executor()
            .spawn(async move {
                let mut t = telemetry.lock().ok()?;
                let mut p = processes.lock().ok()?;
                collect_fast_refresh(&mut t, &mut p, include_net).ok()
            })
            .await;
        if let Some(data) = fast {
            let _ = state.update(cx, |s, cx| {
                s.apply_fast_refresh(data);
                cx.notify();
            });
        }
        let slow = cx
            .background_executor()
            .spawn(async move { collect_slow_refresh() })
            .await;
        let _ = state.update(cx, |s, cx| {
            s.apply_slow_refresh(slow);
            cx.notify();
        });
    })
    .detach();
}

/// Runs a completely isolated, host-clean smoke test without opening windows, registering
/// menu bar items, registering hotkeys, or writing any files to the host.
fn run_smoke_test(opts: &LaunchOptions) -> i32 {
    let mut state = AppState::new(opts.to_runtime_config());
    match state.refresh_blocking_all() {
        Ok(()) => {
            let score = state.report.as_ref().map(|r| r.overall_score).unwrap_or(0);
            let verdict = state
                .report
                .as_ref()
                .map(|r| r.verdict.as_str())
                .unwrap_or("unknown");
            let groups = state.processes.as_ref().map(|p| p.groups.len()).unwrap_or(0);
            let agents = state.agents.as_ref().map(|a| a.sessions.len()).unwrap_or(0);
            let sec = state
                .security
                .as_ref()
                .map(|s| s.overall_summary.as_str())
                .unwrap_or("n/a");
            let clean = state
                .cleanup
                .as_ref()
                .map(|c| c.summary.as_str())
                .unwrap_or("n/a");
            println!(
                "MacHealth smoke-test OK (ephemeral, zero host writes/registrations)\n\
                 - Health Score: {score}/100 ({verdict})\n\
                 - App Groups: {groups} | AI Agent Sessions: {agents}\n\
                 - Security: {sec}\n\
                 - Clean Up: {clean}"
            );
            0
        }
        Err(err) => {
            eprintln!("MacHealth smoke-test failed: {err}");
            1
        }
    }
}

fn main() {
    let opts = LaunchOptions::from_env_and_args();
    if opts.smoke_test {
        std::process::exit(run_smoke_test(&opts));
    }

    let app = Application::new();
    app.on_reopen(|cx| open_main(cx));

    app.run(move |cx: &mut App| {
        let state = cx.new(|_| AppState::new(opts.to_runtime_config()));
        cx.set_global(SharedState(state.clone()));
        cx.set_global(SharedSamplers {
            telemetry: Arc::new(Mutex::new(TelemetryCollector::new())),
            processes: Arc::new(Mutex::new(ProcessSampler::new())),
        });
        cx.set_global(MainWindow(None));
        cx.set_global(PopoverWindow(None));

        cx.on_action(|_: &OpenMain, cx| open_main(cx));
        cx.on_action(|_: &TogglePopover, cx| toggle_popover(cx));
        cx.on_action(|_: &RefreshNow, cx| trigger_manual_refresh(cx));
        cx.on_action(|_: &QuitApp, cx| cx.quit());
        cx.on_action(|_: &CloseWindow, cx| {
            if let Some(w) = cx.active_window() {
                let _ = w.update(cx, |_, window, _| window.remove_window());
            }
        });

        cx.bind_keys([
            KeyBinding::new("cmd-q", QuitApp, None),
            KeyBinding::new("cmd-w", CloseWindow, None),
            KeyBinding::new("cmd-r", RefreshNow, None),
            KeyBinding::new("cmd-k", TogglePopover, None),
        ]);

        cx.set_menus(vec![
            Menu {
                name: "MacHealth".into(),
                items: vec![
                    MenuItem::action("Open Quick Panel", TogglePopover),
                    MenuItem::separator(),
                    MenuItem::os_submenu("Services", SystemMenuType::Services),
                    MenuItem::separator(),
                    MenuItem::action("Quit MacHealth", QuitApp),
                ],
            },
            Menu {
                name: "View".into(),
                items: vec![MenuItem::action("Refresh Now", RefreshNow)],
            },
            Menu {
                name: "Window".into(),
                items: vec![MenuItem::action("Close Window", CloseWindow)],
            },
        ]);

        let (tx, mut rx) = futures::channel::mpsc::unbounded::<MenuBarEvent>();

        // Process-scoped NSStatusItem menu bar badge (disabled via --no-menubar).
        let status_bar = if opts.enable_menubar {
            StatusBarController::new(tx.clone()).map(Rc::new)
        } else {
            None
        };
        if let Some(bar) = status_bar.clone() {
            cx.observe(&state, move |state, cx| {
                let score = state.read(cx).report.as_ref().map(|r| r.overall_score);
                bar.update_score(score);
            })
            .detach();
        }

        // Process-scoped Ctrl+Opt+Space global hotkey (disabled via --no-hotkey).
        let _hotkey = if opts.enable_hotkey {
            GlobalHotKey::register_ctrl_opt_space(tx)
        } else {
            None
        };

        // Keep `_hotkey` and `status_bar` alive for the lifetime of the app event loop.
        cx.spawn(async move |cx| {
            let _keep_bar = status_bar;
            let _keep_hk = _hotkey;
            while let Some(event) = rx.next().await {
                let _ = cx.update(|cx| match event {
                    MenuBarEvent::TogglePopover => toggle_popover(cx),
                });
            }
        })
        .detach();

        // Live 2-second fast telemetry polling loop (runs off the main UI thread).
        let fast_state = state.clone();
        let fast_telemetry = cx.global::<SharedSamplers>().telemetry.clone();
        let fast_processes = cx.global::<SharedSamplers>().processes.clone();
        cx.spawn(async move |cx| loop {
            let include_net = fast_state
                .read_with(cx, |s, _| s.activity_tab == machealth_core::ActivityTab::Network)
                .unwrap_or(false);
            let t_arc = fast_telemetry.clone();
            let p_arc = fast_processes.clone();
            let result = cx
                .background_executor()
                .spawn(async move {
                    let mut t = t_arc.lock().ok()?;
                    let mut p = p_arc.lock().ok()?;
                    collect_fast_refresh(&mut t, &mut p, include_net).ok()
                })
                .await;
            if let Some(data) = result {
                let _ = fast_state.update(cx, |s, cx| {
                    s.apply_fast_refresh(data);
                    cx.notify();
                });
            }
            cx.background_executor().timer(Duration::from_secs(2)).await;
        })
        .detach();

        // Background slow audit loop (Security, Home Files by Kind, Clean Up) every 5 minutes.
        let slow_state = state.clone();
        cx.spawn(async move |cx| loop {
            let slow = cx
                .background_executor()
                .spawn(async move { collect_slow_refresh() })
                .await;
            let _ = slow_state.update(cx, |s, cx| {
                s.apply_slow_refresh(slow);
                cx.notify();
            });
            cx.background_executor().timer(Duration::from_secs(300)).await;
        })
        .detach();

        if opts.open_popover_on_start {
            toggle_popover(cx);
        } else {
            open_main(cx);
        }
    });
}
