use chrono::Utc;
use gpui::Context;
use machealth_core::agents::build_end_session_confirmation;
use machealth_core::processes::collector::{current_pid, current_uid, frontmost_app_pid};
use machealth_core::{
    ActivityTab, AgentSession, AgentsSnapshot, AppGroup, BalanceConfig, BalancePlan, BalanceResult,
    CleanUpCandidate, CleanUpConfirmation, CleanUpScanConfig, CleanUpScanReport, FindingAction,
    HealthReport, HistoryRange, HistorySample, HistorySeries, HistoryStore, HomeFilesSummary,
    MetricsReading, ProcessSampler, ProcessSnapshot, QuickSearchOptions, QuitConfirmation,
    SearchResult, SecurityAuditReport, SystemSnapshot, SystemTaskPolicyExecutor,
    TelemetryCollector, UndoLedger, apply_balance_plan, build_cleanup_confirmation,
    build_quit_confirmation, detect_ai_agents, end_agent_session, evaluate_health,
    move_items_to_trash, open_file, open_system_settings, quick_search, quit_app_group,
    reveal_in_finder, run_security_audit, scan_balance_candidates, scan_cleanup_default,
    scan_home_files_default, summarize_history, undo_balance,
};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavPage {
    Health,
    Activity,
    Agents,
    Security,
    CleanUp,
}

impl NavPage {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Health => "Health",
            Self::Activity => "Activity",
            Self::Agents => "Agents",
            Self::Security => "Security",
            Self::CleanUp => "Home & Clean Up",
        }
    }
}

/// Controls host isolation so test/ephemeral runs never persist anything to `~/Library/Application Support`.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// `None` keeps history & undo ledger strictly in-memory (no host writes at all).
    pub data_dir: Option<PathBuf>,
    pub allow_external_actions: bool,
}

impl RuntimeConfig {
    pub fn default_persistent() -> Self {
        let data_dir = std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|h| h.join("Library/Application Support/MacHealth"));
        Self {
            data_dir,
            allow_external_actions: true,
        }
    }

    pub fn ephemeral() -> Self {
        Self {
            data_dir: None,
            allow_external_actions: true,
        }
    }

    pub fn history_path(&self) -> Option<PathBuf> {
        self.data_dir.as_ref().map(|d| d.join("history.jsonl"))
    }

    pub fn undo_path(&self) -> Option<PathBuf> {
        self.data_dir.as_ref().map(|d| d.join("balance_undo.json"))
    }
}

#[derive(Debug, Clone)]
pub enum PendingModal {
    ConfirmQuitGroup {
        group: AppGroup,
        confirmation: QuitConfirmation,
    },
    ConfirmEndAgent {
        session: AgentSession,
        confirmation: QuitConfirmation,
    },
    BalancePreview {
        plan: BalancePlan,
        selected_idle_pids: HashSet<u32>,
    },
    BalanceOutcome {
        result: BalanceResult,
    },
    ConfirmCleanUp {
        items: Vec<CleanUpCandidate>,
        confirmation: CleanUpConfirmation,
    },
}

/// Snapshot payload computed off the UI thread so GPUI never stutters.
pub struct FastRefreshData {
    pub snapshot: SystemSnapshot,
    pub report: HealthReport,
    pub processes: ProcessSnapshot,
    pub agents: AgentsSnapshot,
    pub history_sample: HistorySample,
}

pub struct SlowRefreshData {
    pub security: SecurityAuditReport,
    pub home_files: HomeFilesSummary,
    pub cleanup: CleanUpScanReport,
}

pub struct AppState {
    pub config: RuntimeConfig,
    pub active_page: NavPage,
    pub activity_tab: ActivityTab,
    pub dashboard_top_tab: ActivityTab,
    pub history_range: HistoryRange,
    pub findings_expanded: bool,
    pub expanded_groups: HashSet<String>,
    pub expanded_agents: HashSet<String>,
    pub selected_cleanup_paths: HashSet<PathBuf>,

    pub snapshot: Option<SystemSnapshot>,
    pub report: Option<HealthReport>,
    pub processes: Option<ProcessSnapshot>,
    pub agents: Option<AgentsSnapshot>,
    pub security: Option<SecurityAuditReport>,
    pub home_files: Option<HomeFilesSummary>,
    pub cleanup: Option<CleanUpScanReport>,

    pub in_memory_history: Vec<HistorySample>,
    pub history_series: HistorySeries,
    pub undo_ledger: UndoLedger,

    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    pub search_error: Option<String>,

    pub pending_modal: Option<PendingModal>,
    pub toast: Option<(String, bool)>,
    pub is_refreshing: bool,
}

impl AppState {
    pub fn new(config: RuntimeConfig) -> Self {
        let undo_ledger = config
            .undo_path()
            .map(|p| UndoLedger::load_from(&p))
            .unwrap_or_default();
        let in_memory_history = config
            .history_path()
            .map(|p| HistoryStore::new(p).load_all())
            .unwrap_or_default();
        let history_series = summarize_history(&in_memory_history, HistoryRange::Hour, Utc::now());

        Self {
            config,
            active_page: NavPage::Health,
            activity_tab: ActivityTab::Cpu,
            dashboard_top_tab: ActivityTab::Cpu,
            history_range: HistoryRange::Hour,
            findings_expanded: true,
            expanded_groups: HashSet::new(),
            expanded_agents: HashSet::new(),
            selected_cleanup_paths: HashSet::new(),
            snapshot: None,
            report: None,
            processes: None,
            agents: None,
            security: None,
            home_files: None,
            cleanup: None,
            in_memory_history,
            history_series,
            undo_ledger,
            search_query: String::new(),
            search_results: Vec::new(),
            search_error: None,
            pending_modal: None,
            toast: None,
            is_refreshing: false,
        }
    }

    /// Runs one complete synchronous fast + slow refresh (used by `--smoke-test` and initial warm-up).
    pub fn refresh_blocking_all(&mut self) -> Result<(), String> {
        let mut telemetry = TelemetryCollector::new();
        let mut proc_sampler = ProcessSampler::new();
        let fast = collect_fast_refresh(&mut telemetry, &mut proc_sampler, false)?;
        self.apply_fast_refresh(fast);
        let slow = collect_slow_refresh();
        self.apply_slow_refresh(slow);
        Ok(())
    }

    pub fn apply_fast_refresh(&mut self, data: FastRefreshData) {
        if let Some(path) = self.config.history_path() {
            let _ = HistoryStore::new(path).append(&data.history_sample);
        }
        self.in_memory_history.push(data.history_sample);
        if self.in_memory_history.len() > 20_000 {
            let drain = self.in_memory_history.len() - 15_000;
            self.in_memory_history.drain(..drain);
        }
        self.history_series =
            summarize_history(&self.in_memory_history, self.history_range, Utc::now());

        self.snapshot = Some(data.snapshot);
        self.report = Some(data.report);
        self.processes = Some(data.processes);
        self.agents = Some(data.agents);
        self.is_refreshing = false;
    }

    pub fn apply_slow_refresh(&mut self, data: SlowRefreshData) {
        self.security = Some(data.security);
        self.home_files = Some(data.home_files);
        self.cleanup = Some(data.cleanup);
    }

    pub fn set_history_range(&mut self, range: HistoryRange, cx: &mut Context<Self>) {
        self.history_range = range;
        self.history_series = summarize_history(&self.in_memory_history, range, Utc::now());
        cx.notify();
    }

    pub fn set_toast(&mut self, message: impl Into<String>, is_error: bool) {
        self.toast = Some((message.into(), is_error));
    }

    pub fn open_balance_preview(&mut self, cx: &mut Context<Self>) {
        let Some(ref proc_snap) = self.processes else {
            self.set_toast("Waiting for process readings…", false);
            cx.notify();
            return;
        };
        let plan = scan_balance_candidates(
            &proc_snap.groups,
            frontmost_app_pid(),
            &self.undo_ledger,
            &BalanceConfig::default(),
            current_uid(),
            current_pid(),
        );
        self.pending_modal = Some(PendingModal::BalancePreview {
            plan,
            selected_idle_pids: HashSet::new(),
        });
        cx.notify();
    }

    pub fn confirm_balance_plan(&mut self, cx: &mut Context<Self>) {
        let Some(PendingModal::BalancePreview {
            plan,
            selected_idle_pids,
        }) = self.pending_modal.take()
        else {
            return;
        };

        let before = self
            .snapshot
            .as_ref()
            .map(MetricsReading::from_snapshot)
            .unwrap_or_else(|| MetricsReading {
                captured_at: Utc::now(),
                cpu_percent: 0.0,
                thermal_level: machealth_core::telemetry::thermal::ThermalLevel::Normal,
                memory_used_bytes: 0,
                memory_pressure: machealth_core::telemetry::memory::MemoryPressure::Normal,
            });

        let selected_quits: Vec<_> = plan
            .idle_quit_candidates
            .iter()
            .filter(|t| selected_idle_pids.contains(&t.pid))
            .cloned()
            .collect();

        let ledger_path = self.config.undo_path();
        let result = apply_balance_plan(
            &plan.throttle_targets,
            &selected_quits,
            before,
            || {
                let mut tc = TelemetryCollector::new();
                tc.collect_once(Duration::from_millis(120))
                    .map(|s| MetricsReading::from_snapshot(&s))
                    .unwrap_or_else(|_| MetricsReading {
                        captured_at: Utc::now(),
                        cpu_percent: 0.0,
                        thermal_level: machealth_core::telemetry::thermal::ThermalLevel::Normal,
                        memory_used_bytes: 0,
                        memory_pressure: machealth_core::telemetry::memory::MemoryPressure::Normal,
                    })
            },
            &SystemTaskPolicyExecutor,
            &mut self.undo_ledger,
            ledger_path.as_deref(),
            current_uid(),
            current_pid(),
        );

        self.set_toast(result.summary.clone(), !result.failures.is_empty());
        self.pending_modal = Some(PendingModal::BalanceOutcome { result });
        cx.notify();
    }

    pub fn perform_undo_balance(&mut self, cx: &mut Context<Self>) {
        let ledger_path = self.config.undo_path();
        let outcome = undo_balance(
            None,
            &SystemTaskPolicyExecutor,
            &mut self.undo_ledger,
            ledger_path.as_deref(),
        );
        self.set_toast(outcome.summary, !outcome.failures.is_empty());
        cx.notify();
    }

    pub fn request_quit_group(&mut self, group: AppGroup, force: bool, cx: &mut Context<Self>) {
        let pids: Vec<u32> = group.processes.iter().map(|p| p.pid).collect();
        let confirmation = build_quit_confirmation(&group.display_name, &pids, force);
        self.pending_modal = Some(PendingModal::ConfirmQuitGroup {
            group,
            confirmation,
        });
        cx.notify();
    }

    pub fn confirm_quit_group(&mut self, cx: &mut Context<Self>) {
        let Some(PendingModal::ConfirmQuitGroup {
            group,
            confirmation,
        }) = self.pending_modal.take()
        else {
            return;
        };
        match quit_app_group(&group, confirmation.force, current_uid(), current_pid()) {
            Ok(outcome) => {
                let msg = if outcome.refused.is_empty() {
                    format!(
                        "{} sent to {} ({} processes).",
                        if outcome.force { "Force Quit" } else { "Quit" },
                        outcome.target_name,
                        outcome.signaled_pids.len()
                    )
                } else {
                    format!(
                        "Sent to {} processes; {} protected processes skipped.",
                        outcome.signaled_pids.len(),
                        outcome.refused.len()
                    )
                };
                self.set_toast(msg, false);
            }
            Err(reason) => self.set_toast(reason, true),
        }
        cx.notify();
    }

    pub fn request_end_agent(&mut self, session: AgentSession, force: bool, cx: &mut Context<Self>) {
        let confirmation = build_end_session_confirmation(&session, force);
        self.pending_modal = Some(PendingModal::ConfirmEndAgent {
            session,
            confirmation,
        });
        cx.notify();
    }

    pub fn confirm_end_agent(&mut self, cx: &mut Context<Self>) {
        let Some(PendingModal::ConfirmEndAgent {
            session,
            confirmation,
        }) = self.pending_modal.take()
        else {
            return;
        };
        match end_agent_session(&session, confirmation.force, current_uid(), current_pid()) {
            Ok(outcome) => {
                self.set_toast(
                    format!(
                        "Ended {} session ({} processes signaled).",
                        outcome.target_name,
                        outcome.signaled_pids.len()
                    ),
                    false,
                );
            }
            Err(reason) => self.set_toast(reason, true),
        }
        cx.notify();
    }

    pub fn request_cleanup_trash(&mut self, cx: &mut Context<Self>) {
        let Some(ref report) = self.cleanup else {
            return;
        };
        let items: Vec<CleanUpCandidate> = report
            .candidates
            .iter()
            .filter(|c| self.selected_cleanup_paths.contains(&c.path))
            .cloned()
            .collect();
        if items.is_empty() {
            self.set_toast("Select one or more items first.", false);
            cx.notify();
            return;
        }
        let confirmation = build_cleanup_confirmation(&items);
        self.pending_modal = Some(PendingModal::ConfirmCleanUp {
            items,
            confirmation,
        });
        cx.notify();
    }

    pub fn confirm_cleanup_trash(&mut self, cx: &mut Context<Self>) {
        let Some(PendingModal::ConfirmCleanUp { items, .. }) = self.pending_modal.take() else {
            return;
        };
        let paths: Vec<PathBuf> = items.into_iter().map(|i| i.path).collect();
        let outcome = move_items_to_trash(&paths);
        for moved in &outcome.moved_to_trash {
            self.selected_cleanup_paths.remove(moved);
        }
        if let Some(ref mut report) = self.cleanup {
            report
                .candidates
                .retain(|c| !outcome.moved_to_trash.contains(&c.path));
            report.total_reclaimable_bytes = report.candidates.iter().map(|c| c.size_bytes).sum();
        }
        let msg = if outcome.refused_or_failed.is_empty() {
            format!(
                "Moved {} item(s) to macOS Trash ({}).",
                outcome.moved_to_trash.len(),
                crate::format::format_bytes_si(outcome.reclaimed_bytes)
            )
        } else {
            format!(
                "Moved {} item(s) to Trash; {} failed ({}).",
                outcome.moved_to_trash.len(),
                outcome.refused_or_failed.len(),
                outcome.refused_or_failed[0].1
            )
        };
        self.set_toast(msg, !outcome.refused_or_failed.is_empty());
        cx.notify();
    }

    pub fn trigger_finding_action(&mut self, action: FindingAction, cx: &mut Context<Self>) {
        match action {
            FindingAction::BalanceLoad => self.open_balance_preview(cx),
            FindingAction::OpenActivityView => {
                self.active_page = NavPage::Activity;
                cx.notify();
            }
            FindingAction::OpenCleanUpView => {
                self.active_page = NavPage::CleanUp;
                cx.notify();
            }
            FindingAction::OpenSecurityView => {
                self.active_page = NavPage::Security;
                cx.notify();
            }
            FindingAction::InspectSleepAssertions => {
                self.active_page = NavPage::Health;
                self.set_toast("Check the Sleep & Wake card below for active blockers.", false);
                cx.notify();
            }
            FindingAction::OpenSystemSettings { url, .. } => {
                if self.config.allow_external_actions {
                    if let Err(err) = open_system_settings(&url) {
                        self.set_toast(err, true);
                    } else {
                        self.set_toast("Opened System Settings.", false);
                    }
                    cx.notify();
                }
            }
        }
    }

    pub fn run_quick_search(&mut self, cx: &mut Context<Self>) {
        let query = self.search_query.trim().to_string();
        if query.is_empty() {
            self.search_results.clear();
            self.search_error = None;
            cx.notify();
            return;
        }
        let home = std::env::var_os("HOME").map(PathBuf::from);
        match quick_search(
            &query,
            &QuickSearchOptions {
                limit: 12,
                only_in_dir: home,
                kind_filter: None,
                filename_only: true,
            },
        ) {
            Ok(results) => {
                self.search_results = results;
                self.search_error = None;
            }
            Err(err) => {
                self.search_error = Some(err);
            }
        }
        cx.notify();
    }

    pub fn open_search_result(&mut self, path: &PathBuf, reveal: bool, cx: &mut Context<Self>) {
        if !self.config.allow_external_actions {
            return;
        }
        let res = if reveal {
            reveal_in_finder(path)
        } else {
            open_file(path)
        };
        if let Err(err) = res {
            self.set_toast(err, true);
            cx.notify();
        }
    }
}

pub fn collect_fast_refresh(
    telemetry: &mut TelemetryCollector,
    proc_sampler: &mut ProcessSampler,
    include_network: bool,
) -> Result<FastRefreshData, String> {
    let snapshot = telemetry.collect_once(Duration::from_millis(100))?;
    let report = evaluate_health(&snapshot);
    let processes = proc_sampler.sample(include_network)?;
    let agents = detect_ai_agents(&processes.processes);
    let busiest_app = processes
        .top_groups(ActivityTab::Cpu, 1)
        .first()
        .map(|g| g.display_name.clone());
    let history_sample = HistorySample::from_snapshot(&snapshot, report.overall_score, busiest_app);

    Ok(FastRefreshData {
        snapshot,
        report,
        processes,
        agents,
        history_sample,
    })
}

pub fn collect_slow_refresh() -> SlowRefreshData {
    let security = run_security_audit();
    let home_files = scan_home_files_default(15_000);
    let cleanup = scan_cleanup_default(&CleanUpScanConfig::default());
    SlowRefreshData {
        security,
        home_files,
        cleanup,
    }
}
