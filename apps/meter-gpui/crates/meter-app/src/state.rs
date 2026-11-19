use crate::settings::Settings;
use crate::statusbar::MenuView;
use crate::system::{self, Notifier, UserNotifier};
use chrono::{DateTime, Duration, FixedOffset, Local, NaiveTime, Offset, TimeZone, Utc};
use gpui::{Context, EventEmitter};
use meter_core::accounts::{Account, AccountStore};
use meter_core::alerts::AlertEngine;
use meter_core::dashboard::{self, AccountRun, Dashboard};
use meter_core::rollup::{self, DayUsage, RollupStore};
use meter_core::ToolId;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Overview,
    Usage,
    Accounts,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Metric {
    Tokens,
    Turns,
}

/// Everything the window and the menu bar item show, plus the work that keeps it fresh.
pub struct AppState {
    pub page: Page,
    pub accounts: Vec<Account>,
    pub dashboard: Dashboard,
    pub settings: Settings,
    pub is_refreshing: bool,
    pub last_refresh: Option<DateTime<Utc>>,
    pub launch_error: Option<String>,
    pub add_tool: ToolId,
    pub usage_metric: Metric,
    pub usage_range_days: i64,
    pub tz: FixedOffset,

    runs: HashMap<String, AccountRun>,
    days: Vec<DayUsage>,
    account_store: AccountStore,
    rollup_store: RollupStore,
    settings_path: PathBuf,
    engine: AlertEngine,
    notifier: Rc<dyn Notifier>,
    /// Only the real notifier needs a permission request.
    real_notifier: bool,
}

pub struct Refreshed;
impl EventEmitter<Refreshed> for AppState {}

impl AppState {
    /// `support` is injected so tests never touch the real Application Support folder.
    pub fn new(support: PathBuf, notifier: Option<Rc<dyn Notifier>>) -> Self {
        let settings_path = support.join("settings.json");
        let settings = Settings::load(&settings_path);
        let account_store = AccountStore::new(support.join("accounts.json"));
        let rollup_store = RollupStore::new(support.join("rollup.json"));
        let tz = Local::now().offset().fix();
        let accounts = account_store.effective();
        let days = rollup_store.load();
        let real_notifier = notifier.is_none();
        let notifier: Rc<dyn Notifier> = notifier.unwrap_or_else(|| Rc::new(UserNotifier));
        let mut s = AppState {
            page: Page::Overview,
            dashboard: Dashboard::default(),
            engine: AlertEngine::new(vec![settings.threshold_low, settings.threshold_high]),
            accounts,
            settings,
            is_refreshing: false,
            last_refresh: None,
            launch_error: None,
            add_tool: ToolId::Codex,
            usage_metric: Metric::Tokens,
            usage_range_days: 14,
            tz,
            runs: HashMap::new(),
            days,
            account_store,
            rollup_store,
            settings_path,
            notifier,
            real_notifier,
        };
        s.rebuild(Utc::now());
        s
    }

    pub fn default_support_dir() -> PathBuf {
        meter_core::util::home_dir().join("Library/Application Support/MeterGPUI")
    }

    // ---- refresh

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        if self.is_refreshing {
            return;
        }
        self.is_refreshing = true;
        cx.notify();
        let accounts = self.accounts.clone();
        let previous = self.runs.clone();
        let now = Utc::now();
        let since = self.window_start(now);
        let task = cx
            .background_executor()
            .spawn(async move { dashboard::refresh(&accounts, &previous, now, since) });
        cx.spawn(async move |this, cx| {
            let runs = task.await;
            this.update(cx, |s, cx| s.apply(runs, now, since, cx)).ok();
        })
        .detach();
    }

    /// Local midnight 35 days ago, so every refreshed day is a whole day.
    fn window_start(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        let day = (now.with_timezone(&self.tz) - Duration::days(35)).date_naive();
        self.tz
            .from_local_datetime(&day.and_time(NaiveTime::MIN))
            .single()
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or(now - Duration::days(35))
    }

    fn apply(&mut self, runs: HashMap<String, AccountRun>, now: DateTime<Utc>, since: DateTime<Utc>, cx: &mut Context<Self>) {
        self.runs = runs;
        // Hidden accounts keep accumulating history; only the display and alerts skip them.
        let sessions: Vec<_> = self.runs.values().flat_map(|r| r.output.sessions.clone()).collect();
        let from_day = rollup::day_key(since, self.tz);
        let fresh: Vec<DayUsage> =
            rollup::days_from(&sessions, self.tz).into_iter().filter(|d| d.day.as_str() >= from_day.as_str()).collect();
        self.days = self.rollup_store.merge(fresh, &ToolId::ALL, &from_day).unwrap_or_else(|_| self.rollup_store.load());
        self.last_refresh = Some(now);
        self.rebuild(now);
        self.deliver_alerts(now);
        crate::autotest::write_summary(self);
        self.is_refreshing = false;
        cx.emit(Refreshed);
        cx.notify();
    }

    fn rebuild(&mut self, now: DateTime<Utc>) {
        self.dashboard = Dashboard::build(&self.accounts, &self.runs, &self.days, self.tz, now);
    }

    fn deliver_alerts(&mut self, now: DateTime<Utc>) {
        let alerts = self.engine.evaluate(&self.dashboard.visible_quotas);
        for a in alerts.iter().filter(|a| self.settings.alert_policy.allows(a, now, self.tz)) {
            self.notifier.deliver(a);
        }
    }

    // ---- accounts

    pub fn add_account(&mut self, tool: ToolId, label: String, data_root: String, cx: &mut Context<Self>) {
        let id = format!("{}-{}", tool.raw(), Utc::now().timestamp_millis());
        let account = Account { id, tool, label, data_root: Some(data_root), is_hidden: false };
        let _ = self.account_store.add(account);
        self.accounts = self.account_store.effective();
        cx.notify();
        self.refresh(cx);
    }

    pub fn set_hidden(&mut self, hidden: bool, account: &Account, cx: &mut Context<Self>) {
        let _ = self.account_store.set_hidden(hidden, account.tool, &account.id);
        self.accounts = self.account_store.effective();
        self.rebuild(Utc::now());
        cx.emit(Refreshed);
        cx.notify();
    }

    pub fn remove_account(&mut self, account: &Account, cx: &mut Context<Self>) {
        if account.id == "default" {
            return;
        }
        let _ = self.account_store.remove(account.tool, &account.id);
        self.runs.remove(&account.key());
        self.accounts = self.account_store.effective();
        self.rebuild(Utc::now());
        cx.emit(Refreshed);
        cx.notify();
    }

    // ---- settings

    /// Applies a change, saves it, and runs any side effect it needs. The login item and the
    /// notification permission are touched only here, only when the user flipped the setting.
    pub fn update_settings(&mut self, change: impl FnOnce(&mut Settings), cx: &mut Context<Self>) {
        let old = self.settings.clone();
        change(&mut self.settings);
        if self.settings == old {
            return;
        }
        self.settings.save(&self.settings_path);
        self.engine.thresholds = {
            let mut t = vec![self.settings.threshold_low, self.settings.threshold_high];
            t.sort_by(f64::total_cmp);
            t
        };
        if self.settings.launch_at_login != old.launch_at_login {
            match system::set_launch_at_login(self.settings.launch_at_login) {
                Ok(()) => self.launch_error = None,
                Err(_) => {
                    self.launch_error = Some("Could not change Open Meter at login.".into());
                    self.settings.launch_at_login = old.launch_at_login;
                    self.settings.save(&self.settings_path);
                }
            }
        }
        if old.alert_policy.enabled_kinds.is_empty() && !self.settings.alert_policy.enabled_kinds.is_empty() && self.real_notifier {
            UserNotifier.request_authorization();
        }
        cx.emit(Refreshed);
        cx.notify();
    }

    /// Test support: set the data directly instead of reading it.
    #[cfg(test)]
    pub fn load_for_test(&mut self, accounts: Vec<Account>, runs: HashMap<String, AccountRun>, days: Vec<DayUsage>, now: DateTime<Utc>) {
        self.accounts = accounts;
        self.runs = runs;
        self.days = days;
        self.last_refresh = Some(now);
        self.rebuild(now);
    }

    // ---- menu bar

    pub fn menu_view(&self) -> MenuView {
        MenuView {
            lines: self.dashboard.menu.lines.clone(),
            warning: self.dashboard.menu.warning.clone(),
            percent: self.dashboard.menu.tightest_percent,
            show_percent: self.settings.show_percent_in_menu_bar,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::RecordingNotifier;

    #[test]
    fn menu_view_title_and_symbol_follow_the_percent_and_setting() {
        let mut v = MenuView { show_percent: true, percent: Some(62.4), ..Default::default() };
        assert_eq!((v.title().as_str(), v.symbol()), (" 62%", "gauge.with.dots.needle.67percent"));
        v.percent = Some(95.0);
        assert_eq!(v.symbol(), "exclamationmark.gauge");
        v.show_percent = false;
        assert_eq!(v.title(), "");
        assert_eq!(MenuView::default().symbol(), "gauge.with.dots.needle.33percent");
        assert_eq!(MenuView::default().accessibility_label(), "Meter");
        assert!(v.accessibility_label().contains("95 percent"));
    }

    #[test]
    fn new_state_uses_only_the_injected_folder() {
        let dir = tempfile::tempdir().unwrap();
        let s = AppState::new(dir.path().to_path_buf(), Some(Rc::new(RecordingNotifier::default())));
        assert_eq!(s.accounts.len(), 5);
        assert_eq!(s.page, Page::Overview);
        assert!(!s.settings.launch_at_login && s.settings.alert_policy.enabled_kinds.is_empty());
    }

    #[test]
    fn window_start_is_local_midnight_35_days_back() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = AppState::new(dir.path().to_path_buf(), Some(Rc::new(RecordingNotifier::default())));
        s.tz = FixedOffset::east_opt(5 * 3600 + 1800).unwrap();
        let now = meter_core::util::parse_date("2026-10-24T12:00:00Z").unwrap();
        let start = s.window_start(now);
        assert_eq!(rollup::day_key(start, s.tz), "2026-09-19");
        assert_eq!(start.with_timezone(&s.tz).format("%H:%M").to_string(), "00:00");
    }
}
