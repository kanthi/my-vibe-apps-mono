//! The menu bar item, built directly on AppKit (NSStatusItem + NSMenu) because GPUI has no
//! menu-bar-extra API. The menu is a plain native menu, as the macOS guidelines ask: lines of
//! text, then a few commands. Clicks are sent to GPUI over a channel.

use futures::channel::mpsc::UnboundedSender;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObjectProtocol};
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSImage, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem, NSVariableStatusItemLength};
use objc2_foundation::{NSObject, NSString};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuEvent {
    OpenMain,
    Refresh,
    OpenSettings,
    Quit,
}

/// What the menu shows. Built from the dashboard; nothing here is AppKit.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MenuView {
    pub lines: Vec<String>,
    pub warning: Option<String>,
    pub percent: Option<f64>,
    pub show_percent: bool,
}

impl MenuView {
    /// The text beside the icon: the tightest percent, or nothing.
    pub fn title(&self) -> String {
        match (self.show_percent, self.percent) {
            (true, Some(p)) => format!(" {:.0}%", p),
            _ => String::new(),
        }
    }

    /// A change of symbol, not of color, signals a filling limit; the menu bar draws template images.
    pub fn symbol(&self) -> &'static str {
        match self.percent {
            Some(p) if p >= 90.0 => "exclamationmark.gauge",
            Some(p) if p >= 60.0 => "gauge.with.dots.needle.67percent",
            _ => "gauge.with.dots.needle.33percent",
        }
    }

    pub fn accessibility_label(&self) -> String {
        match self.percent {
            Some(p) => format!("Meter, tightest limit {:.0} percent used", p),
            None => "Meter".into(),
        }
    }
}

struct Ivars {
    tx: UnboundedSender<MenuEvent>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "MeterMenuTarget"]
    #[ivars = Ivars]
    struct MenuTarget;

    unsafe impl NSObjectProtocol for MenuTarget {}

    impl MenuTarget {
        #[unsafe(method(openMeter:))]
        fn open_meter(&self, _sender: Option<&AnyObject>) {
            let _ = self.ivars().tx.unbounded_send(MenuEvent::OpenMain);
        }
        #[unsafe(method(refreshNow:))]
        fn refresh_now(&self, _sender: Option<&AnyObject>) {
            let _ = self.ivars().tx.unbounded_send(MenuEvent::Refresh);
        }
        #[unsafe(method(openSettings:))]
        fn open_settings(&self, _sender: Option<&AnyObject>) {
            let _ = self.ivars().tx.unbounded_send(MenuEvent::OpenSettings);
        }
        #[unsafe(method(quitMeter:))]
        fn quit_meter(&self, _sender: Option<&AnyObject>) {
            let _ = self.ivars().tx.unbounded_send(MenuEvent::Quit);
        }
    }
);

impl MenuTarget {
    fn new(mtm: MainThreadMarker, tx: UnboundedSender<MenuEvent>) -> Retained<Self> {
        let this = mtm.alloc::<MenuTarget>().set_ivars(Ivars { tx });
        unsafe { msg_send![super(this), init] }
    }
}

pub struct StatusBar {
    mtm: MainThreadMarker,
    item: Retained<NSStatusItem>,
    menu: Retained<NSMenu>,
    target: Retained<MenuTarget>,
}

impl StatusBar {
    /// Must be called on the main thread (GPUI's `run` callback is).
    pub fn new(tx: UnboundedSender<MenuEvent>) -> Option<Self> {
        let mtm = MainThreadMarker::new()?;
        let bar = NSStatusBar::systemStatusBar();
        let item = bar.statusItemWithLength(NSVariableStatusItemLength);
        let menu = NSMenu::new(mtm);
        menu.setAutoenablesItems(false);
        item.setMenu(Some(&menu));
        let target = MenuTarget::new(mtm, tx);
        let s = StatusBar { mtm, item, menu, target };
        s.update(&MenuView::default());
        Some(s)
    }

    /// Test hook: opens the menu as if the item were clicked. Blocks until the menu is dismissed.
    pub fn open_menu(&self) {
        if let Some(button) = self.item.button(self.mtm) {
            unsafe { button.performClick(None) };
        }
    }

    pub fn update(&self, view: &MenuView) {
        if let Some(button) = self.item.button(self.mtm) {
            let symbol = NSString::from_str(view.symbol());
            let label = NSString::from_str(&view.accessibility_label());
            let image = NSImage::imageWithSystemSymbolName_accessibilityDescription(&symbol, Some(&label));
            if let Some(img) = &image {
                img.setTemplate(true);
            }
            button.setImage(image.as_deref());
            button.setTitle(&NSString::from_str(&view.title()));
        }
        self.rebuild_menu(view);
    }

    fn rebuild_menu(&self, view: &MenuView) {
        self.menu.removeAllItems();
        if view.lines.is_empty() {
            self.menu.addItem(&self.text_item("No usage found yet"));
        }
        for line in &view.lines {
            self.menu.addItem(&self.text_item(line));
        }
        if let Some(w) = &view.warning {
            self.menu.addItem(&self.text_item(&format!("⚠︎ {w}")));
        }
        self.menu.addItem(&NSMenuItem::separatorItem(self.mtm));
        self.menu.addItem(&self.command("Open Meter…", sel!(openMeter:), ""));
        self.menu.addItem(&self.command("Refresh Now", sel!(refreshNow:), "r"));
        self.menu.addItem(&self.command("Settings…", sel!(openSettings:), ","));
        self.menu.addItem(&NSMenuItem::separatorItem(self.mtm));
        self.menu.addItem(&self.command("Quit Meter", sel!(quitMeter:), "q"));
    }

    /// A disabled item: plain text in the menu, no action.
    fn text_item(&self, text: &str) -> Retained<NSMenuItem> {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                self.mtm.alloc(),
                &NSString::from_str(text),
                None,
                &NSString::from_str(""),
            )
        };
        item.setEnabled(false);
        item
    }

    fn command(&self, title: &str, action: objc2::runtime::Sel, key: &str) -> Retained<NSMenuItem> {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                self.mtm.alloc(),
                &NSString::from_str(title),
                Some(action),
                &NSString::from_str(key),
            )
        };
        unsafe { item.setTarget(Some(&self.target)) };
        item.setEnabled(true);
        item
    }
}
