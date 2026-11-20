use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ImpactLabel {
    Light,
    Moderate,
    Heavy,
}

impl ImpactLabel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Moderate => "Moderate",
            Self::Heavy => "Heavy",
        }
    }

    /// Plain-English load classification combining CPU %, Memory footprint, and Energy impact.
    pub fn classify(cpu_percent: f32, memory_bytes: u64, energy_impact: f32) -> Self {
        const MIB: u64 = 1_048_576;
        if cpu_percent >= 50.0 || memory_bytes >= 2048 * MIB || energy_impact >= 45.0 {
            Self::Heavy
        } else if cpu_percent >= 15.0 || memory_bytes >= 512 * MIB || energy_impact >= 15.0 {
            Self::Moderate
        } else {
            Self::Light
        }
    }
}

/// Returns a short, plain-English description of what common macOS services and apps do.
pub fn describe_process(name: &str, app_bundle_name: Option<&str>, command_line: &str) -> String {
    let lower = name.to_ascii_lowercase();
    let cmd_lower = command_line.to_ascii_lowercase();

    match lower.as_str() {
        "kernel_task" => "Core macOS kernel managing hardware, memory, and thermal control".into(),
        "launchd" => "macOS system manager that starts and supervises background services".into(),
        "windowserver" => "Draws every window, menu, and animation on your display".into(),
        "finder" => "macOS desktop and file browser".into(),
        "dock" => "Manages the Dock, Mission Control, and app switching".into(),
        "controlcenter" => "Menu bar Control Center and quick toggles".into(),
        "mds" | "mds_stores" | "mdworker" | "mdworker_shared" => {
            "Spotlight search indexing service".into()
        }
        "coreaudiod" => "macOS audio engine for speakers, microphones, and Bluetooth audio".into(),
        "powerd" => "Manages sleep, wake, and power assertions".into(),
        "cloudd" | "bird" | "nsurlsessiond" => "iCloud sync and background downloads".into(),
        "timemachine" | "backupd" | "backupd-helper" => "Time Machine background backup".into(),
        "photos" | "photoanalysisd" | "mediaanalysisd" => {
            "Photos library indexing and on-device image analysis".into()
        }
        "softwareupdated" => "Checks for and stages macOS software updates".into(),
        "mdnsresponder" => "Handles DNS lookups and local network discovery (Bonjour)".into(),
        "logd" => "Records macOS system and app diagnostics".into(),
        "fseventsd" => "Watches file changes for Spotlight, Time Machine, and developer tools".into(),
        "spotlight" => "Spotlight search interface".into(),
        "safari" => "Apple web browser".into(),
        "google chrome" | "chrome" => "Google Chrome web browser".into(),
        "firefox" => "Mozilla Firefox web browser".into(),
        "arc" => "Arc web browser".into(),
        "brave browser" => "Brave web browser".into(),
        "xcode" => "Apple IDE for building macOS and iOS apps".into(),
        "code" | "visual studio code" | "electron" => "Code editor or Electron desktop app".into(),
        "cursor" => "Cursor AI code editor".into(),
        "claude" => "Claude AI desktop app or CLI agent".into(),
        "chatgpt" => "ChatGPT desktop app".into(),
        "codex" => "Codex AI coding agent".into(),
        "ollama" | "ollama_llama_server" => "Local LLM inference server (Ollama)".into(),
        "lm studio" | "llama-server" => "Local LLM inference runtime".into(),
        "docker" | "com.docker.backend" | "orbstack" => "Local container runtime".into(),
        "rust-analyzer" => "Rust code intelligence server".into(),
        "cargo" | "rustc" | "clang" | "swiftc" => "Compiler building code locally".into(),
        "node" | "bun" | "deno" => "JavaScript/TypeScript runtime".into(),
        "python" | "python3" => "Python interpreter".into(),
        "caffeinate" => "Command-line tool temporarily keeping your Mac awake".into(),
        _ => {
            if lower.contains("webkit") {
                return "Web page rendering engine (WebKit)".into();
            }
            if lower.contains("helper") || lower.contains("renderer") || lower.contains("gpu") {
                if let Some(app) = app_bundle_name {
                    return format!("Background helper process for {app}");
                }
                return "Background helper process for an application".into();
            }
            if cmd_lower.contains("mcp") {
                return "Model Context Protocol (MCP) tool server".into();
            }
            if let Some(app) = app_bundle_name {
                return format!("{app} application");
            }
            if lower.ends_with('d') && command_line.starts_with("/System/") {
                return "macOS background system service".into();
            }
            if command_line.starts_with("/usr/libexec/") {
                return "macOS built-in background helper".into();
            }
            "Running process".into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_impact_levels() {
        assert_eq!(ImpactLabel::classify(2.0, 100 * 1_048_576, 3.0), ImpactLabel::Light);
        assert_eq!(ImpactLabel::classify(25.0, 200 * 1_048_576, 8.0), ImpactLabel::Moderate);
        assert_eq!(ImpactLabel::classify(5.0, 3000 * 1_048_576, 2.0), ImpactLabel::Heavy);
        assert_eq!(ImpactLabel::classify(65.0, 100 * 1_048_576, 50.0), ImpactLabel::Heavy);
    }

    #[test]
    fn describes_known_and_helper_processes() {
        assert!(describe_process("WindowServer", None, "/System/...").contains("display"));
        assert!(
            describe_process(
                "Google Chrome Helper (Renderer)",
                Some("Google Chrome"),
                "/Applications/Google Chrome.app/..."
            )
            .contains("Google Chrome")
        );
    }
}
