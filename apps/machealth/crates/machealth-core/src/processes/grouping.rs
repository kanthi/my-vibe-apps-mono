use super::descriptions::{ImpactLabel, describe_process};
use super::{ActivityTab, AppGroup, ProcessInfo};
use std::collections::HashMap;

/// Extracts the outermost `.app` bundle name and root bundle path from an executable path.
///
/// Example:
/// `/Applications/Google Chrome.app/Contents/Frameworks/.../Google Chrome Helper.app/Contents/MacOS/Google Chrome Helper`
/// returns `(Some("Google Chrome"), Some("/Applications/Google Chrome.app"), true)` (helper = true).
pub fn extract_outer_app_bundle(path: &str) -> (Option<String>, Option<String>, bool) {
    let Some(first_app_idx) = path.find(".app/") else {
        if path.ends_with(".app") {
            let name = path
                .trim_end_matches(".app")
                .rsplit('/')
                .next()
                .map(str::to_string);
            return (name, Some(path.to_string()), false);
        }
        return (None, None, false);
    };

    let bundle_end = first_app_idx + 4; // include ".app"
    let bundle_path = &path[..bundle_end];
    let app_name = bundle_path
        .trim_end_matches(".app")
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let after_outer = &path[bundle_end..];
    // Nested `.app/` or `.xpc/` or `Frameworks/` inside the outer bundle indicates a helper process.
    let is_nested_helper = after_outer.contains(".app/")
        || after_outer.contains(".xpc/")
        || after_outer.contains("/Frameworks/")
        || after_outer.contains("/XPCServices/")
        || after_outer.contains("/PlugIns/")
        || after_outer.contains("/Helpers/");

    (app_name, Some(bundle_path.to_string()), is_nested_helper)
}

pub fn looks_like_helper_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("helper")
        || lower.contains("renderer")
        || lower.contains("gpu")
        || lower.contains("crashpad")
        || lower.contains("webcontent")
        || lower.contains("xpc")
        || lower.contains("plugin")
        || lower.ends_with("agent")
}

/// Rolls up helper and child processes under their parent application bundle.
pub fn group_processes(processes: &[ProcessInfo]) -> Vec<AppGroup> {
    let by_pid: HashMap<u32, &ProcessInfo> = processes.iter().map(|p| (p.pid, p)).collect();

    // Determine effective group key for each process:
    // 1. Outermost `.app` bundle path on the process itself.
    // 2. Otherwise walk up `ppid` chain (stopping at PID 1 / launchd) to see if an ancestor
    //    is a GUI `.app` bundle — rolls up spawned helpers/XPC/subprocesses to that app!
    // 3. Otherwise standalone group keyed by process name/PID.
    let mut group_members: HashMap<String, (String, Option<String>, Vec<ProcessInfo>)> =
        HashMap::new();

    for proc_info in processes {
        let (group_key, display_name, bundle_path) = resolve_group_owner(proc_info, &by_pid);
        let entry = group_members
            .entry(group_key)
            .or_insert_with(|| (display_name, bundle_path, Vec::new()));
        entry.2.push(proc_info.clone());
    }

    let mut groups: Vec<AppGroup> = group_members
        .into_iter()
        .map(|(id, (display_name, bundle_path, mut members))| {
            // Sort members: main non-helper process first, then highest CPU + memory.
            members.sort_by(|a, b| {
                a.is_helper
                    .cmp(&b.is_helper)
                    .then_with(|| {
                        b.cpu_percent
                            .partial_cmp(&a.cpu_percent)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .then_with(|| b.memory_bytes.cmp(&a.memory_bytes))
                    .then_with(|| a.pid.cmp(&b.pid))
            });

            let primary_pid = members.first().map(|m| m.pid).unwrap_or(0);
            let process_count = members.len();
            let helper_count = process_count.saturating_sub(1);

            let cpu_percent: f32 = members.iter().map(|m| m.cpu_percent).sum();
            let memory_bytes: u64 = members.iter().map(|m| m.memory_bytes).sum();
            let energy_impact: f32 = members.iter().map(|m| m.energy_impact).sum();
            let disk_read_bytes: u64 = members.iter().map(|m| m.disk_read_bytes).sum();
            let disk_write_bytes: u64 = members.iter().map(|m| m.disk_write_bytes).sum();
            let disk_read_bytes_per_sec: f64 =
                members.iter().map(|m| m.disk_read_bytes_per_sec).sum();
            let disk_write_bytes_per_sec: f64 =
                members.iter().map(|m| m.disk_write_bytes_per_sec).sum();
            let net_rx_bytes: u64 = members.iter().map(|m| m.net_rx_bytes).sum();
            let net_tx_bytes: u64 = members.iter().map(|m| m.net_tx_bytes).sum();
            let net_rx_bytes_per_sec: f64 = members.iter().map(|m| m.net_rx_bytes_per_sec).sum();
            let net_tx_bytes_per_sec: f64 = members.iter().map(|m| m.net_tx_bytes_per_sec).sum();

            let impact_label = ImpactLabel::classify(cpu_percent, memory_bytes, energy_impact);
            let primary_cmd = members
                .first()
                .map(|m| m.command_line.as_str())
                .unwrap_or("");
            let description = describe_process(
                &display_name,
                bundle_path.as_ref().map(|_| display_name.as_str()),
                primary_cmd,
            );

            AppGroup {
                id,
                display_name,
                primary_pid,
                bundle_path,
                description,
                impact_label,
                process_count,
                helper_count,
                cpu_percent,
                memory_bytes,
                energy_impact,
                disk_read_bytes,
                disk_write_bytes,
                disk_read_bytes_per_sec,
                disk_write_bytes_per_sec,
                net_rx_bytes,
                net_tx_bytes,
                net_rx_bytes_per_sec,
                net_tx_bytes_per_sec,
                processes: members,
            }
        })
        .collect();

    sort_groups(&mut groups, ActivityTab::Cpu);
    groups
}

fn resolve_group_owner(
    proc_info: &ProcessInfo,
    by_pid: &HashMap<u32, &ProcessInfo>,
) -> (String, String, Option<String>) {
    // Special case: Safari WebKit helper processes live under `/System/Library/Frameworks/WebKit.framework`
    // and often have launchd (PID 1) as ppid. Group them cleanly under Safari when they belong to WebKit.
    if proc_info.name.starts_with("com.apple.WebKit.") {
        return (
            "app:Safari".to_string(),
            "Safari".to_string(),
            Some("/Applications/Safari.app".to_string()),
        );
    }

    if let Some(ref bundle_name) = proc_info.app_bundle_name {
        return (
            format!("app:{bundle_name}"),
            bundle_name.clone(),
            proc_info.app_bundle_path.clone(),
        );
    }

    // Walk up ancestor chain (bounded to 16 hops to prevent cycles) to attribute helpers
    // spawned by a parent GUI app, skipping terminals/shells so CLI tools stay inspectable.
    let mut cursor = proc_info.ppid;
    let mut depth = 0;
    while cursor > 1 && depth < 16 {
        let Some(parent) = by_pid.get(&cursor) else {
            break;
        };
        if let Some(ref bundle_name) = parent.app_bundle_name {
            if !is_terminal_app(bundle_name) {
                return (
                    format!("app:{bundle_name}"),
                    bundle_name.clone(),
                    parent.app_bundle_path.clone(),
                );
            }
            break;
        }
        if parent.ppid == cursor {
            break;
        }
        cursor = parent.ppid;
        depth += 1;
    }

    (
        format!("proc:{}", proc_info.name),
        proc_info.name.clone(),
        None,
    )
}

fn is_terminal_app(bundle_name: &str) -> bool {
    matches!(
        bundle_name,
        "Terminal" | "iTerm2" | "Warp" | "Ghostty" | "Alacritty" | "kitty" | "WezTerm"
    )
}

pub fn sort_groups(groups: &mut [AppGroup], tab: ActivityTab) {
    groups.sort_by(|a, b| {
        let primary = match tab {
            ActivityTab::Cpu => b
                .cpu_percent
                .partial_cmp(&a.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal),
            ActivityTab::Memory => b.memory_bytes.cmp(&a.memory_bytes),
            ActivityTab::Energy => b
                .energy_impact
                .partial_cmp(&a.energy_impact)
                .unwrap_or(std::cmp::Ordering::Equal),
            ActivityTab::Disk => {
                let b_rate = b.disk_read_bytes_per_sec + b.disk_write_bytes_per_sec;
                let a_rate = a.disk_read_bytes_per_sec + a.disk_write_bytes_per_sec;
                b_rate
                    .partial_cmp(&a_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| {
                        (b.disk_read_bytes + b.disk_write_bytes)
                            .cmp(&(a.disk_read_bytes + a.disk_write_bytes))
                    })
            }
            ActivityTab::Network => {
                let b_rate = b.net_rx_bytes_per_sec + b.net_tx_bytes_per_sec;
                let a_rate = a.net_rx_bytes_per_sec + a.net_tx_bytes_per_sec;
                b_rate
                    .partial_cmp(&a_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| (b.net_rx_bytes + b.net_tx_bytes).cmp(&(a.net_rx_bytes + a.net_tx_bytes)))
            }
        };
        primary
            .then_with(|| b.memory_bytes.cmp(&a.memory_bytes))
            .then_with(|| a.display_name.cmp(&b.display_name))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_proc(
        pid: u32,
        ppid: u32,
        name: &str,
        path: &str,
        cpu: f32,
        mem_mib: u64,
    ) -> ProcessInfo {
        let (bundle_name, bundle_path, nested) = extract_outer_app_bundle(path);
        ProcessInfo {
            pid,
            ppid,
            uid: 501,
            name: name.to_string(),
            executable_path: Some(path.to_string()),
            command_line: path.to_string(),
            app_bundle_name: bundle_name,
            app_bundle_path: bundle_path,
            is_helper: nested || looks_like_helper_name(name),
            cpu_percent: cpu,
            memory_bytes: mem_mib * 1_048_576,
            energy_impact: cpu * 0.8,
            disk_read_bytes: 0,
            disk_write_bytes: 0,
            disk_read_bytes_per_sec: 0.0,
            disk_write_bytes_per_sec: 0.0,
            net_rx_bytes: 0,
            net_tx_bytes: 0,
            net_rx_bytes_per_sec: 0.0,
            net_tx_bytes_per_sec: 0.0,
            uptime_seconds: 600,
            impact_label: ImpactLabel::Light,
            description: String::new(),
        }
    }

    #[test]
    fn rolls_up_chrome_helpers_and_spawned_children() {
        let procs = vec![
            mock_proc(
                100,
                1,
                "Google Chrome",
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                8.0,
                300,
            ),
            mock_proc(
                101,
                100,
                "Google Chrome Helper (Renderer)",
                "/Applications/Google Chrome.app/Contents/Frameworks/Google Chrome Framework.framework/Helpers/Google Chrome Helper (Renderer).app/Contents/MacOS/Google Chrome Helper (Renderer)",
                24.0,
                900,
            ),
            mock_proc(
                102,
                100,
                "chrome_crashpad_handler",
                "/usr/local/bin/chrome_crashpad_handler",
                1.0,
                20,
            ),
            mock_proc(
                200,
                1,
                "Finder",
                "/System/Library/CoreServices/Finder.app/Contents/MacOS/Finder",
                0.5,
                120,
            ),
        ];

        let groups = group_processes(&procs);
        assert_eq!(groups.len(), 2);
        let chrome = groups.iter().find(|g| g.display_name == "Google Chrome").unwrap();
        assert_eq!(chrome.primary_pid, 100);
        assert_eq!(chrome.process_count, 3);
        assert_eq!(chrome.helper_count, 2);
        assert!((chrome.cpu_percent - 33.0).abs() < 0.01);
        assert_eq!(chrome.memory_bytes, 1220 * 1_048_576);
    }
}
