use super::{AppGroup, ProcessInfo};
use serde::{Deserialize, Serialize};

/// Critical macOS processes that MacHealth will always refuse to quit or throttle.
const PROTECTED_PROCESS_NAMES: &[&str] = &[
    "kernel_task",
    "launchd",
    "windowserver",
    "loginwindow",
    "finder",
    "dock",
    "systemuiserver",
    "controlcenter",
    "coreaudiod",
    "powerd",
    "logd",
    "configd",
    "distnoted",
    "cfprefsd",
    "usereventagent",
    "fseventsd",
    "opendirectoryd",
    "securityd",
    "trustd",
    "tccd",
    "lsd",
    "runningboardd",
    "bluetoothd",
    "airportd",
    "mdnsresponder",
    "notifyd",
    "diskarbitrationd",
    "corebrightnessd",
    "hidd",
    "watchdogd",
    "syspolicyd",
    "endpointsecurityd",
    "thermalmonitord",
    "mds",
    "mds_stores",
    "machealth",
    "machealth-app",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardrailVerdict {
    Allowed,
    Refused { reason: String },
}

impl GuardrailVerdict {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuitConfirmation {
    pub title: String,
    pub target_name: String,
    pub pids: Vec<u32>,
    pub force: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuitOutcome {
    pub target_name: String,
    pub signaled_pids: Vec<u32>,
    pub refused: Vec<(u32, String)>,
    pub force: bool,
}

pub fn check_process_safety(
    proc_info: &ProcessInfo,
    current_uid: u32,
    current_pid: u32,
) -> GuardrailVerdict {
    if proc_info.pid == 0 || proc_info.pid == 1 {
        return GuardrailVerdict::Refused {
            reason: format!(
                "{} (PID {}) is a core macOS process and cannot be modified.",
                proc_info.name, proc_info.pid
            ),
        };
    }

    if proc_info.pid == current_pid {
        return GuardrailVerdict::Refused {
            reason: "MacHealth cannot quit or throttle itself.".to_string(),
        };
    }

    if proc_info.uid == 0 {
        return GuardrailVerdict::Refused {
            reason: format!(
                "{} runs as macOS system (root) and is protected.",
                proc_info.name
            ),
        };
    }

    if proc_info.uid != current_uid {
        return GuardrailVerdict::Refused {
            reason: format!(
                "{} belongs to another user account (UID {}) and is protected.",
                proc_info.name, proc_info.uid
            ),
        };
    }

    let lower_name = proc_info.name.to_ascii_lowercase();
    if PROTECTED_PROCESS_NAMES.contains(&lower_name.as_str()) {
        return GuardrailVerdict::Refused {
            reason: format!(
                "{} is a protected macOS system service required for normal operation.",
                proc_info.name
            ),
        };
    }

    if let Some(ref path) = proc_info.executable_path {
        if path.starts_with("/System/Library/CoreServices/")
            || path.starts_with("/System/Library/PrivateFrameworks/")
            || path.starts_with("/usr/libexec/")
            || path.starts_with("/sbin/")
        {
            return GuardrailVerdict::Refused {
                reason: format!(
                    "{} is a built-in macOS system component and is protected.",
                    proc_info.name
                ),
            };
        }
    }

    GuardrailVerdict::Allowed
}

/// Builds a human-readable confirmation dialog payload before quitting an app or process.
pub fn build_quit_confirmation(
    target_name: &str,
    pids: &[u32],
    force: bool,
) -> QuitConfirmation {
    let action = if force { "Force Quit" } else { "Quit" };
    let helper_note = if pids.len() > 1 {
        format!(" and its {} helper processes", pids.len() - 1)
    } else {
        String::new()
    };
    let warning = if force {
        "Any unsaved changes will be lost immediately."
    } else {
        "The app will be asked to close cleanly."
    };

    QuitConfirmation {
        title: format!("{action} {target_name}?"),
        target_name: target_name.to_string(),
        pids: pids.to_vec(),
        force,
        message: format!("{action} {target_name}{helper_note}. {warning}"),
    }
}

/// Safely sends `SIGTERM` (Quit) or `SIGKILL` (Force Quit) to a single process after guardrail checks.
pub fn quit_process(
    proc_info: &ProcessInfo,
    force: bool,
    current_uid: u32,
    current_pid: u32,
) -> Result<(), String> {
    match check_process_safety(proc_info, current_uid, current_pid) {
        GuardrailVerdict::Allowed => {}
        GuardrailVerdict::Refused { reason } => return Err(reason),
    }

    send_signal(proc_info.pid, force)
}

/// Safely quits all allowed processes in an `AppGroup`, refusing any protected members with clear reasons.
pub fn quit_app_group(
    group: &AppGroup,
    force: bool,
    current_uid: u32,
    current_pid: u32,
) -> Result<QuitOutcome, String> {
    let mut signaled_pids = Vec::new();
    let mut refused = Vec::new();

    for proc_info in &group.processes {
        match check_process_safety(proc_info, current_uid, current_pid) {
            GuardrailVerdict::Allowed => {
                if let Err(err) = send_signal(proc_info.pid, force) {
                    refused.push((proc_info.pid, err));
                } else {
                    signaled_pids.push(proc_info.pid);
                }
            }
            GuardrailVerdict::Refused { reason } => {
                refused.push((proc_info.pid, reason));
            }
        }
    }

    if signaled_pids.is_empty() && !refused.is_empty() {
        return Err(refused[0].1.clone());
    }

    Ok(QuitOutcome {
        target_name: group.display_name.clone(),
        signaled_pids,
        refused,
        force,
    })
}

pub(crate) fn send_signal(pid: u32, force: bool) -> Result<(), String> {
    let sig = if force { libc::SIGKILL } else { libc::SIGTERM };
    let ret = unsafe { libc::kill(pid as libc::pid_t, sig) };
    if ret == 0 {
        Ok(())
    } else {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::ESRCH) {
            // Process already exited.
            Ok(())
        } else {
            Err(format!("Could not signal PID {pid}: {err}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processes::descriptions::ImpactLabel;
    use std::process::Command;

    fn sample_proc(pid: u32, uid: u32, name: &str, path: &str) -> ProcessInfo {
        ProcessInfo {
            pid,
            ppid: 1,
            uid,
            name: name.to_string(),
            executable_path: Some(path.to_string()),
            command_line: path.to_string(),
            app_bundle_name: None,
            app_bundle_path: None,
            is_helper: false,
            cpu_percent: 1.0,
            memory_bytes: 10_000_000,
            energy_impact: 1.0,
            disk_read_bytes: 0,
            disk_write_bytes: 0,
            disk_read_bytes_per_sec: 0.0,
            disk_write_bytes_per_sec: 0.0,
            net_rx_bytes: 0,
            net_tx_bytes: 0,
            net_rx_bytes_per_sec: 0.0,
            net_tx_bytes_per_sec: 0.0,
            uptime_seconds: 60,
            impact_label: ImpactLabel::Light,
            description: String::new(),
        }
    }

    #[test]
    fn guardrails_refuse_protected_processes() {
        let uid = 501;
        let self_pid = 4242;

        // PID 0 / 1
        assert!(!check_process_safety(&sample_proc(0, 0, "kernel_task", ""), uid, self_pid).is_allowed());
        assert!(!check_process_safety(&sample_proc(1, 0, "launchd", "/sbin/launchd"), uid, self_pid).is_allowed());

        // Self
        assert!(!check_process_safety(&sample_proc(self_pid, uid, "machealth", "/Applications/MacHealth.app/Contents/MacOS/machealth"), uid, self_pid).is_allowed());

        // Root / other user
        assert!(!check_process_safety(&sample_proc(500, 0, "some_daemon", "/usr/local/bin/some_daemon"), uid, self_pid).is_allowed());
        assert!(!check_process_safety(&sample_proc(501, 502, "Safari", "/Applications/Safari.app/Contents/MacOS/Safari"), uid, self_pid).is_allowed());

        // Protected macOS UI / system services owned by current user
        assert!(!check_process_safety(&sample_proc(600, uid, "WindowServer", "/System/Library/PrivateFrameworks/SkyLight.framework/Resources/WindowServer"), uid, self_pid).is_allowed());
        assert!(!check_process_safety(&sample_proc(601, uid, "Dock", "/System/Library/CoreServices/Dock.app/Contents/MacOS/Dock"), uid, self_pid).is_allowed());
        assert!(!check_process_safety(&sample_proc(602, uid, "Finder", "/System/Library/CoreServices/Finder.app/Contents/MacOS/Finder"), uid, self_pid).is_allowed());

        // Normal user app is allowed
        assert!(check_process_safety(&sample_proc(700, uid, "Google Chrome", "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"), uid, self_pid).is_allowed());
    }

    #[test]
    fn gracefully_quits_spawned_child_process() {
        let mut child = Command::new("/bin/sleep")
            .arg("30")
            .spawn()
            .expect("spawn sleep");
        let child_pid = child.id();
        let proc_info = sample_proc(
            child_pid,
            crate::processes::collector::current_uid(),
            "sleep",
            "/bin/sleep",
        );

        quit_process(
            &proc_info,
            false,
            crate::processes::collector::current_uid(),
            crate::processes::collector::current_pid(),
        )
        .expect("SIGTERM child");

        let status = child.wait().expect("wait child");
        assert!(!status.success());
    }
}
