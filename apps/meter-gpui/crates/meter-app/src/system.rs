//! Things that change the host: notifications and the login item. Both do nothing in a bare
//! executable (no bundle id) and nothing until the user turns the matching setting on.

use block2::RcBlock;
use meter_core::Alert;
use objc2::runtime::Bool;
use objc2_foundation::{NSBundle, NSError, NSString};
use objc2_service_management::SMAppService;
use objc2_user_notifications::{
    UNAuthorizationOptions, UNMutableNotificationContent, UNNotificationRequest, UNUserNotificationCenter,
};

/// True when running from a real .app bundle. Without a bundle id the notification center aborts.
pub fn has_bundle() -> bool {
    NSBundle::mainBundle().bundleIdentifier().is_some()
}

/// Alert delivery is behind a trait so tests use a fake and never reach macOS notifications.
pub trait Notifier {
    fn deliver(&self, alert: &Alert);
}

pub struct UserNotifier;

impl UserNotifier {
    /// Asked only when the first alert is switched on, never at launch.
    pub fn request_authorization(&self) {
        if !has_bundle() {
            return;
        }
        let center = UNUserNotificationCenter::currentNotificationCenter();
        let handler = RcBlock::new(|_granted: Bool, _error: *mut NSError| {});
        center.requestAuthorizationWithOptions_completionHandler(
            UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound,
            &handler,
        );
    }
}

impl Notifier for UserNotifier {
    fn deliver(&self, alert: &Alert) {
        if !has_bundle() {
            return;
        }
        let center = UNUserNotificationCenter::currentNotificationCenter();
        let content = UNMutableNotificationContent::new();
        content.setTitle(&NSString::from_str(&alert.title));
        content.setBody(&NSString::from_str(&alert.body));
        content.setThreadIdentifier(&NSString::from_str(&format!("{}/{}/{}", alert.tool.raw(), alert.account_id, alert.label)));
        let id = NSString::from_str(&format!("meter-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)));
        let request = UNNotificationRequest::requestWithIdentifier_content_trigger(&id, &content, None);
        center.addNotificationRequest_withCompletionHandler(&request, None);
    }
}

/// Registers or removes the Login Items entry. Only called when the user flips the setting.
pub fn set_launch_at_login(on: bool) -> Result<(), String> {
    if !has_bundle() {
        return Ok(());
    }
    let service = unsafe { SMAppService::mainAppService() };
    let result = unsafe { if on { service.registerAndReturnError() } else { service.unregisterAndReturnError() } };
    result.map_err(|e| e.localizedDescription().to_string())
}

/// Collects alerts instead of showing them. Used by tests.
#[cfg(test)]
#[derive(Default)]
pub struct RecordingNotifier {
    pub delivered: std::cell::RefCell<Vec<Alert>>,
}

#[cfg(test)]
impl Notifier for RecordingNotifier {
    fn deliver(&self, alert: &Alert) {
        self.delivered.borrow_mut().push(alert.clone());
    }
}
