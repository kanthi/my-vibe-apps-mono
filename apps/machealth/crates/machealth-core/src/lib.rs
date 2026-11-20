pub mod agents;
pub mod balancer;
pub mod cleanup;
pub mod history;
pub mod processes;
pub mod scoring;
pub mod search;
pub mod security;
pub mod telemetry;

pub use agents::{AgentKind, AgentSession, AgentsSnapshot, detect_ai_agents, end_agent_session};
pub use balancer::{
    BalanceActionKind, BalanceConfig, BalancePlan, BalanceResult, BalanceTarget, MetricsReading,
    SystemTaskPolicyExecutor, TaskPolicyExecutor, ThrottledRecord, UndoLedger, UndoResult,
    apply_balance_plan, scan_balance_candidates, undo_balance,
};
pub use cleanup::{
    CleanUpCandidate, CleanUpCategory, CleanUpConfirmation, CleanUpScanConfig, CleanUpScanReport,
    FileEntrySummary, FileKind, FileKindBucket, HomeFilesSummary, TrashMoveOutcome, TrashSummary,
    build_cleanup_confirmation, move_items_to_trash, scan_cleanup_default, scan_cleanup_roots,
    scan_files_by_kind, scan_home_files_default, validate_trash_target,
};
pub use history::{
    HistoryPoint, HistoryRange, HistorySample, HistorySeries, HistoryStore, summarize_history,
};
pub use processes::{
    ActivityTab, AppGroup, GuardrailVerdict, ProcessInfo, ProcessSampler, ProcessSnapshot,
    QuitConfirmation, QuitOutcome, build_quit_confirmation, check_process_safety, quit_app_group,
    quit_process,
};
pub use scoring::{
    AreaKind, AreaScore, FindingAction, HealthFinding, HealthGrade, HealthReport, Severity,
    evaluate_health,
};
pub use search::{
    QuickSearchOptions, SearchResult, open_file, quick_search, reveal_in_finder,
};
pub use security::{
    AuditStatus, SecurityAuditReport, SecurityCheckItem, open_system_settings, run_security_audit,
};
pub use telemetry::{SystemSnapshot, TelemetryCollector};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn live_end_to_end_snapshot_and_health_score() {
        let mut collector = TelemetryCollector::new();
        let snapshot = collector
            .collect_once(Duration::from_millis(120))
            .expect("live system snapshot");
        let report = evaluate_health(&snapshot);

        assert_eq!(report.areas.len(), 8);
        assert!(report.overall_score <= 100);
        assert!(!report.verdict.is_empty());

        println!(
            "=== LIVE MACHEALTH SNAPSHOT ===\nScore: {}/100 ({})\nCPU: {:.1}% (P: {:?}, E: {:?}) | Thermal: {}\nMemory: {:.2}/{:.2} GiB (Swap: {:.2} GiB, Pressure: {:?})\nStorage: {:.1}/{:.1} GB free (SMART: {:?})\nBattery: present={}, charge={:?}%, health={:?}%, cycles={:?}\nSleep blockers: {} | Crashes(7d): {} | LaunchAgents: {}\nFindings ({}):",
            report.overall_score,
            report.verdict,
            snapshot.cpu.total_percent,
            snapshot.cpu.performance_percent,
            snapshot.cpu.efficiency_percent,
            snapshot.thermal.label,
            snapshot.memory.used_bytes as f64 / 1_073_741_824.0,
            snapshot.memory.total_bytes as f64 / 1_073_741_824.0,
            snapshot.memory.swap_used_bytes as f64 / 1_073_741_824.0,
            snapshot.memory.pressure,
            snapshot.storage.available_bytes as f64 / 1_000_000_000.0,
            snapshot.storage.total_bytes as f64 / 1_000_000_000.0,
            snapshot.storage.smart_status,
            snapshot.battery.has_battery,
            snapshot.battery.charge_percent,
            snapshot.battery.health_percent,
            snapshot.battery.cycle_count,
            snapshot.sleep.active_assertions.len(),
            snapshot.stability.recent_crash_count_7d,
            snapshot.startup_items.total_count,
            report.findings.len(),
        );
        for f in &report.findings {
            println!("  - [{:?}] {}: {}", f.severity, f.title, f.detail);
        }
    }

    #[test]
    fn live_end_to_end_processes_agents_and_balancer_scan() {
        let mut proc_sampler = ProcessSampler::new();
        let snap = proc_sampler.sample(false).expect("live process snapshot");
        assert!(!snap.groups.is_empty());

        let top_mem = snap.top_groups(ActivityTab::Memory, 5);
        let agents = detect_ai_agents(&snap.processes);
        let plan = scan_balance_candidates(
            &snap.groups,
            processes::collector::frontmost_app_pid(),
            &UndoLedger::default(),
            &BalanceConfig::default(),
            processes::collector::current_uid(),
            processes::collector::current_pid(),
        );

        println!(
            "=== LIVE PHASE 2 SNAPSHOT ===\nProcesses: {} | Grouped Apps: {}\nTop 5 by Memory:",
            snap.processes.len(),
            snap.groups.len()
        );
        for g in &top_mem {
            println!(
                "  - {} ({} procs, {}): {:.1}% CPU, {:.0} MiB",
                g.display_name,
                g.process_count,
                g.impact_label.label(),
                g.cpu_percent,
                g.memory_bytes as f64 / 1_048_576.0
            );
        }
        println!(
            "AI Agent Sessions: {} ({} total procs, {:.1}% CPU, {:.0} MiB)",
            agents.sessions.len(),
            agents.total_process_count,
            agents.total_cpu_percent,
            agents.total_memory_bytes as f64 / 1_048_576.0
        );
        for s in agents.sessions.iter().take(5) {
            println!(
                "  - [{}] {} (PID {}, +{} children): {:.1}% CPU, {:.0} MiB",
                s.kind.label(),
                s.display_name,
                s.root_pid,
                s.spawned_children.len(),
                s.cpu_percent,
                s.memory_bytes as f64 / 1_048_576.0
            );
        }
        println!("Balance Plan: {}", plan.summary);
    }

    #[test]
    fn live_end_to_end_security_and_cleanup_scan() {
        let audit = run_security_audit();
        assert_eq!(audit.checks.len(), 6);
        let cleanup = scan_cleanup_default(&CleanUpScanConfig::default());

        println!(
            "=== LIVE PHASE 3 SNAPSHOT ===\nSecurity: {}\nClean Up: {} (Trash: {} items, {:.1} MB)",
            audit.overall_summary,
            cleanup.summary,
            cleanup.trash_summary.item_count,
            cleanup.trash_summary.total_bytes as f64 / 1_000_000.0
        );
        for c in &audit.checks {
            println!("  - [{}] {}: {}", c.status.label(), c.title, c.summary);
        }
    }
}
