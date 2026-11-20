//! Process-scoped native macOS `NSStatusItem` menu bar badge.
//!
//! Automatically removed by macOS when MacHealth exits — registers nothing persistently
//! on the host system.

use futures::channel::mpsc::UnboundedSender;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObjectProtocol};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{NSImage, NSStatusBar, NSStatusItem, NSVariableStatusItemLength};
use objc2_foundation::{NSObject, NSString};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuBarEvent {
    TogglePopover,
}

struct TargetIvars {
    tx: UnboundedSender<MenuBarEvent>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "MacHealthStatusBarTarget"]
    #[ivars = TargetIvars]
    struct StatusBarTarget;

    unsafe impl NSObjectProtocol for StatusBarTarget {}

    impl StatusBarTarget {
        #[unsafe(method(togglePopover:))]
        fn toggle_popover(&self, _sender: Option<&AnyObject>) {
            let _ = self.ivars().tx.unbounded_send(MenuBarEvent::TogglePopover);
        }
    }
);

impl StatusBarTarget {
    fn new(mtm: MainThreadMarker, tx: UnboundedSender<MenuBarEvent>) -> Retained<Self> {
        let this = mtm.alloc::<StatusBarTarget>().set_ivars(TargetIvars { tx });
        unsafe { msg_send![super(this), init] }
    }
}

pub struct StatusBarController {
    mtm: MainThreadMarker,
    item: Retained<NSStatusItem>,
    _target: Retained<StatusBarTarget>,
}

impl StatusBarController {
    /// Creates a process-scoped `NSStatusItem` on the main thread.
    pub fn new(tx: UnboundedSender<MenuBarEvent>) -> Option<Self> {
        let mtm = MainThreadMarker::new()?;
        let bar = NSStatusBar::systemStatusBar();
        let item = bar.statusItemWithLength(NSVariableStatusItemLength);
        let target = StatusBarTarget::new(mtm, tx);

        if let Some(button) = item.button(mtm) {
            unsafe {
                button.setTarget(Some(&target));
                button.setAction(Some(sel!(togglePopover:)));
            }
        }

        let ctrl = Self {
            mtm,
            item,
            _target: target,
        };
        ctrl.update_score(None);
        Some(ctrl)
    }

    pub fn update_score(&self, score: Option<u8>) {
        let Some(button) = self.item.button(self.mtm) else {
            return;
        };

        let symbol_name = match score {
            Some(s) if s < 65 => "exclamationmark.triangle",
            Some(_) => "heart.text.square",
            None => "heart.text.square",
        };
        let label = match score {
            Some(s) => format!("MacHealth score {s} out of 100"),
            None => "MacHealth".to_string(),
        };
        let title = match score {
            Some(s) => format!(" {s}"),
            None => String::new(),
        };

        let symbol_ns = NSString::from_str(symbol_name);
        let label_ns = NSString::from_str(&label);
        let image = NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &symbol_ns,
            Some(&label_ns),
        );
        if let Some(ref img) = image {
            img.setTemplate(true);
        }
        button.setImage(image.as_deref());
        button.setTitle(&NSString::from_str(&title));
    }
}

impl Drop for StatusBarController {
    fn drop(&mut self) {
        let bar = NSStatusBar::systemStatusBar();
        bar.removeStatusItem(&self.item);
    }
}
