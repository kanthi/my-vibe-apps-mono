//! Process-scoped global hotkey (`Control-Option-Space`) via macOS Carbon `RegisterEventHotKey`.
//!
//! Uses Carbon's public event hotkey API so:
//! - **Zero Accessibility / TCC permissions** are required or registered.
//! - **Zero persistence** on macOS — unregistered cleanly in `Drop` and on process exit.

use crate::menubar::MenuBarEvent;
use futures::channel::mpsc::UnboundedSender;
use std::ffi::c_void;
use std::ptr;

const K_EVENT_CLASS_KEYBOARD: u32 = u32::from_be_bytes(*b"keyb");
const K_EVENT_HOT_KEY_PRESSED: u32 = 5;
const K_VK_SPACE: u32 = 49;
const OPTION_KEY: u32 = 1 << 11; // 0x0800
const CONTROL_KEY: u32 = 1 << 12; // 0x1000
const HOTKEY_SIGNATURE: u32 = u32::from_be_bytes(*b"MHLT");
const HOTKEY_ID: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct EventTypeSpec {
    event_class: u32,
    event_kind: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EventHotKeyID {
    signature: u32,
    id: u32,
}

type EventHandlerProc = unsafe extern "C" fn(
    next_handler: *mut c_void,
    event: *mut c_void,
    user_data: *mut c_void,
) -> i32;

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    fn GetApplicationEventTarget() -> *mut c_void;
    fn InstallEventHandler(
        target: *mut c_void,
        handler: EventHandlerProc,
        num_types: u32,
        list: *const EventTypeSpec,
        user_data: *mut c_void,
        out_ref: *mut *mut c_void,
    ) -> i32;
    fn RemoveEventHandler(handler_ref: *mut c_void) -> i32;
    fn RegisterEventHotKey(
        hot_key_code: u32,
        hot_key_modifiers: u32,
        hot_key_id: EventHotKeyID,
        target: *mut c_void,
        options: u32,
        out_ref: *mut *mut c_void,
    ) -> i32;
    fn UnregisterEventHotKey(hot_key_ref: *mut c_void) -> i32;
}

unsafe extern "C" fn hotkey_callback(
    _next_handler: *mut c_void,
    _event: *mut c_void,
    user_data: *mut c_void,
) -> i32 {
    if !user_data.is_null() {
        let tx = unsafe { &*(user_data as *const UnboundedSender<MenuBarEvent>) };
        let _ = tx.unbounded_send(MenuBarEvent::TogglePopover);
    }
    0 // noErr
}

pub struct GlobalHotKey {
    hotkey_ref: *mut c_void,
    handler_ref: *mut c_void,
    user_data_box: *mut UnboundedSender<MenuBarEvent>,
}

impl GlobalHotKey {
    /// Registers `Control-Option-Space` for the lifetime of this process only.
    pub fn register_ctrl_opt_space(tx: UnboundedSender<MenuBarEvent>) -> Option<Self> {
        let boxed = Box::into_raw(Box::new(tx));
        let target = unsafe { GetApplicationEventTarget() };
        if target.is_null() {
            unsafe { drop(Box::from_raw(boxed)) };
            return None;
        }

        let spec = EventTypeSpec {
            event_class: K_EVENT_CLASS_KEYBOARD,
            event_kind: K_EVENT_HOT_KEY_PRESSED,
        };
        let mut handler_ref: *mut c_void = ptr::null_mut();
        let status = unsafe {
            InstallEventHandler(
                target,
                hotkey_callback,
                1,
                &spec,
                boxed as *mut c_void,
                &mut handler_ref,
            )
        };
        if status != 0 || handler_ref.is_null() {
            unsafe { drop(Box::from_raw(boxed)) };
            return None;
        }

        let hk_id = EventHotKeyID {
            signature: HOTKEY_SIGNATURE,
            id: HOTKEY_ID,
        };
        let mut hotkey_ref: *mut c_void = ptr::null_mut();
        let reg_status = unsafe {
            RegisterEventHotKey(
                K_VK_SPACE,
                CONTROL_KEY | OPTION_KEY,
                hk_id,
                target,
                0,
                &mut hotkey_ref,
            )
        };
        if reg_status != 0 || hotkey_ref.is_null() {
            unsafe {
                let _ = RemoveEventHandler(handler_ref);
                drop(Box::from_raw(boxed));
            }
            return None;
        }

        Some(Self {
            hotkey_ref,
            handler_ref,
            user_data_box: boxed,
        })
    }
}

impl Drop for GlobalHotKey {
    fn drop(&mut self) {
        unsafe {
            if !self.hotkey_ref.is_null() {
                let _ = UnregisterEventHotKey(self.hotkey_ref);
            }
            if !self.handler_ref.is_null() {
                let _ = RemoveEventHandler(self.handler_ref);
            }
            if !self.user_data_box.is_null() {
                drop(Box::from_raw(self.user_data_box));
            }
        }
    }
}
