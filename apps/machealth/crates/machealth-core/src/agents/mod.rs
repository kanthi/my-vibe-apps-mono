use crate::processes::actions::{
    GuardrailVerdict, QuitConfirmation, QuitOutcome, build_quit_confirmation, check_process_safety,
    send_signal,
};
use crate::processes::descriptions::ImpactLabel;
use crate::processes::ProcessInfo;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentKind {
    Claude,
    ChatGpt,
    Cursor,
    Codex,
    Ollama,
    LmStudio,
    Antigravity,
    McpServer,
    CodingAgent,
    LocalAiWorker,
}

impl AgentKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Claude => "Claude",
            Self::ChatGpt => "ChatGPT",
            Self::Cursor => "Cursor",
            Self::Codex => "Codex",
            Self::Ollama => "Ollama",
            Self::LmStudio => "LM Studio",
            Self::Antigravity => "Antigravity",
            Self::McpServer => "MCP Server",
            Self::CodingAgent => "Coding Agent",
            Self::LocalAiWorker => "AI Worker",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub kind: AgentKind,
    pub display_name: String,
    pub root_pid: u32,
    pub root_process: ProcessInfo,
    pub spawned_children: Vec<ProcessInfo>,
    pub total_process_count: usize,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub energy_impact: f32,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
    pub impact_label: ImpactLabel,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentsSnapshot {
    pub sessions: Vec<AgentSession>,
    pub total_cpu_percent: f32,
    pub total_memory_bytes: u64,
    pub total_energy_impact: f32,
    pub total_process_count: usize,
}

/// Identifies if a single process matches a known local AI app, coding agent, MCP server, or worker.
pub fn match_agent_signature(proc_info: &ProcessInfo) -> Option<(AgentKind, String)> {
    let name = proc_info.name.to_ascii_lowercase();
    let bundle = proc_info
        .app_bundle_name
        .as_deref()
        .unwrap_or("")
        .to_ascii_lowercase();
    let cmd = proc_info.command_line.to_ascii_lowercase();

    // Never match MacHealth itself even if its command line or path mentions "agent".
    if name.contains("machealth") {
        return None;
    }

    if bundle == "claude" || name == "claude" || cmd.contains("@anthropic-ai/claude-code") || cmd.contains("/claude-code") {
        return Some((AgentKind::Claude, "Claude".to_string()));
    }
    if bundle == "chatgpt" || name == "chatgpt" {
        return Some((AgentKind::ChatGpt, "ChatGPT".to_string()));
    }
    if bundle == "cursor" || name == "cursor" || name == "cursor-agent" || cmd.contains("cursor.app/") {
        return Some((AgentKind::Cursor, "Cursor".to_string()));
    }
    if name == "codex" || cmd.contains("@openai/codex") || cmd.contains("/codex ") || cmd.ends_with("/codex") {
        return Some((AgentKind::Codex, "Codex".to_string()));
    }
    if name == "ollama" || name == "ollama_llama_server" || bundle == "ollama" {
        return Some((AgentKind::Ollama, "Ollama".to_string()));
    }
    if bundle == "lm studio" || name == "lm studio" || name == "lms" || name == "llama-server" {
        return Some((AgentKind::LmStudio, "LM Studio".to_string()));
    }
    if bundle.contains("antigravity")
        || name.contains("antigravity")
        || name == "agy"
        || cmd.contains("antigravity-acp")
    {
        return Some((AgentKind::Antigravity, "Antigravity".to_string()));
    }
    if bundle == "windsurf"
        || name == "aider"
        || name == "goose"
        || name == "opencode"
        || cmd.contains("/opencode")
        || cmd.contains("cline")
        || cmd.contains("copilot-agent")
    {
        let label = proc_info
            .app_bundle_name
            .clone()
            .unwrap_or_else(|| proc_info.name.clone());
        return Some((AgentKind::CodingAgent, label));
    }
    if cmd.contains("mcp-server")
        || cmd.contains("mcp_server")
        || cmd.contains("@modelcontextprotocol/")
        || cmd.contains("fastmcp")
        || name.starts_with("mcp-")
        || name.ends_with("-mcp")
    {
        return Some((AgentKind::McpServer, format!("MCP ({})", proc_info.name)));
    }
    if (name.starts_with("python") || name == "node" || name == "bun" || name == "uv")
        && (cmd.contains("langchain")
            || cmd.contains("langgraph")
            || cmd.contains("crewai")
            || cmd.contains("autogen")
            || cmd.contains("mlx_lm")
            || cmd.contains("vllm")
            || cmd.contains("comfyui")
            || cmd.contains("openai_agents")
            || cmd.contains("agent_worker"))
    {
        return Some((AgentKind::LocalAiWorker, format!("AI Worker ({})", proc_info.name)));
    }

    None
}

/// Scans all running processes, detects local AI agents, and attributes every spawned child
/// process (recursively via the process tree) to its owning root agent session.
pub fn detect_ai_agents(processes: &[ProcessInfo]) -> AgentsSnapshot {
    let by_pid: HashMap<u32, &ProcessInfo> = processes.iter().map(|p| (p.pid, p)).collect();
    let mut children_map: HashMap<u32, Vec<u32>> = HashMap::new();
    for p in processes {
        if p.ppid != p.pid {
            children_map.entry(p.ppid).or_default().push(p.pid);
        }
    }

    // 1. Identify all matching agent PIDs.
    let mut matched: HashMap<u32, (AgentKind, String)> = HashMap::new();
    for p in processes {
        if let Some(sig) = match_agent_signature(p) {
            matched.insert(p.pid, sig);
        }
    }

    // 2. Keep only root agent PIDs (those with no ancestor that is ALSO a matched agent).
    //    That way helpers, MCP servers, and sub-workers spawned by Claude/Cursor/Antigravity
    //    roll up cleanly under their parent agent session with zero double-counting.
    let mut root_pids: Vec<u32> = matched
        .keys()
        .copied()
        .filter(|&pid| !has_matched_ancestor(pid, &by_pid, &matched))
        .collect();
    root_pids.sort_unstable();

    let mut sessions = Vec::new();
    let mut claimed: HashSet<u32> = HashSet::new();

    for root_pid in root_pids {
        let Some(&root_proc) = by_pid.get(&root_pid) else {
            continue;
        };
        let Some((kind, display_name)) = matched.get(&root_pid).cloned() else {
            continue;
        };

        // BFS through all spawned descendants.
        let mut queue = VecDeque::new();
        let mut descendants = Vec::new();
        claimed.insert(root_pid);

        if let Some(kids) = children_map.get(&root_pid) {
            for &kid in kids {
                if claimed.insert(kid) {
                    queue.push_back(kid);
                }
            }
        }
        while let Some(curr) = queue.pop_front() {
            if let Some(&p) = by_pid.get(&curr) {
                descendants.push(p.clone());
            }
            if let Some(kids) = children_map.get(&curr) {
                for &kid in kids {
                    if claimed.insert(kid) {
                        queue.push_back(kid);
                    }
                }
            }
        }

        // Sort descendants by CPU descending, then memory descending.
        descendants.sort_by(|a, b| {
            b.cpu_percent
                .partial_cmp(&a.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.memory_bytes.cmp(&a.memory_bytes))
        });

        let cpu_percent: f32 =
            root_proc.cpu_percent + descendants.iter().map(|d| d.cpu_percent).sum::<f32>();
        let memory_bytes: u64 =
            root_proc.memory_bytes + descendants.iter().map(|d| d.memory_bytes).sum::<u64>();
        let energy_impact: f32 =
            root_proc.energy_impact + descendants.iter().map(|d| d.energy_impact).sum::<f32>();
        let disk_read_bytes: u64 =
            root_proc.disk_read_bytes + descendants.iter().map(|d| d.disk_read_bytes).sum::<u64>();
        let disk_write_bytes: u64 = root_proc.disk_write_bytes
            + descendants.iter().map(|d| d.disk_write_bytes).sum::<u64>();
        let net_rx_bytes: u64 =
            root_proc.net_rx_bytes + descendants.iter().map(|d| d.net_rx_bytes).sum::<u64>();
        let net_tx_bytes: u64 =
            root_proc.net_tx_bytes + descendants.iter().map(|d| d.net_tx_bytes).sum::<u64>();

        let total_process_count = 1 + descendants.len();
        let impact_label = ImpactLabel::classify(cpu_percent, memory_bytes, energy_impact);
        let summary = if descendants.is_empty() {
            format!("{} ({} impact)", display_name, impact_label.label())
        } else {
            format!(
                "{} + {} spawned child {} ({} impact)",
                display_name,
                descendants.len(),
                if descendants.len() == 1 { "process" } else { "processes" },
                impact_label.label()
            )
        };

        sessions.push(AgentSession {
            id: format!("agent:{root_pid}"),
            kind,
            display_name,
            root_pid,
            root_process: root_proc.clone(),
            spawned_children: descendants,
            total_process_count,
            cpu_percent,
            memory_bytes,
            energy_impact,
            disk_read_bytes,
            disk_write_bytes,
            net_rx_bytes,
            net_tx_bytes,
            impact_label,
            summary,
        });
    }

    sessions.sort_by(|a, b| {
        b.cpu_percent
            .partial_cmp(&a.cpu_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.memory_bytes.cmp(&a.memory_bytes))
    });

    let total_cpu_percent = sessions.iter().map(|s| s.cpu_percent).sum();
    let total_memory_bytes = sessions.iter().map(|s| s.memory_bytes).sum();
    let total_energy_impact = sessions.iter().map(|s| s.energy_impact).sum();
    let total_process_count = sessions.iter().map(|s| s.total_process_count).sum();

    AgentsSnapshot {
        sessions,
        total_cpu_percent,
        total_memory_bytes,
        total_energy_impact,
        total_process_count,
    }
}

fn has_matched_ancestor(
    pid: u32,
    by_pid: &HashMap<u32, &ProcessInfo>,
    matched: &HashMap<u32, (AgentKind, String)>,
) -> bool {
    let mut cursor = by_pid.get(&pid).map(|p| p.ppid).unwrap_or(0);
    let mut depth = 0;
    while cursor > 1 && depth < 32 {
        if matched.contains_key(&cursor) {
            return true;
        }
        let Some(parent) = by_pid.get(&cursor) else {
            break;
        };
        if parent.ppid == cursor {
            break;
        }
        cursor = parent.ppid;
        depth += 1;
    }
    false
}

/// Builds a confirmation prompt before ending an AI agent session and its spawned children.
pub fn build_end_session_confirmation(session: &AgentSession, force: bool) -> QuitConfirmation {
    let mut pids = vec![session.root_pid];
    pids.extend(session.spawned_children.iter().map(|c| c.pid));
    build_quit_confirmation(&session.display_name, &pids, force)
}

/// Ends an AI agent session and its spawned children (children first, then root), respecting guardrails.
pub fn end_agent_session(
    session: &AgentSession,
    force: bool,
    current_uid: u32,
    current_pid: u32,
) -> Result<QuitOutcome, String> {
    let mut signaled_pids = Vec::new();
    let mut refused = Vec::new();

    // Signal spawned children first (reverse BFS order) so they don't orphan, then root.
    let targets = session
        .spawned_children
        .iter()
        .rev()
        .chain(std::iter::once(&session.root_process));

    for proc_info in targets {
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
        target_name: session.display_name.clone(),
        signaled_pids,
        refused,
        force,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(pid: u32, ppid: u32, name: &str, cmd: &str, cpu: f32, mem_mib: u64) -> ProcessInfo {
        ProcessInfo {
            pid,
            ppid,
            uid: 501,
            name: name.to_string(),
            executable_path: Some(cmd.to_string()),
            command_line: cmd.to_string(),
            app_bundle_name: None,
            app_bundle_path: None,
            is_helper: false,
            cpu_percent: cpu,
            memory_bytes: mem_mib * 1_048_576,
            energy_impact: cpu,
            disk_read_bytes: 1000,
            disk_write_bytes: 500,
            disk_read_bytes_per_sec: 0.0,
            disk_write_bytes_per_sec: 0.0,
            net_rx_bytes: 0,
            net_tx_bytes: 0,
            net_rx_bytes_per_sec: 0.0,
            net_tx_bytes_per_sec: 0.0,
            uptime_seconds: 120,
            impact_label: ImpactLabel::Light,
            description: String::new(),
        }
    }

    #[test]
    fn attributes_full_spawned_process_tree_to_parent_agent() {
        let procs = vec![
            // Claude Code root
            mk(1000, 1, "claude", "/opt/homebrew/bin/claude", 15.0, 250),
            // MCP server spawned by Claude
            mk(1001, 1000, "node", "node /usr/local/lib/node_modules/@modelcontextprotocol/server-fs", 5.0, 120),
            // Shell + ripgrep spawned by Claude (grandchild!)
            mk(1002, 1000, "zsh", "/bin/zsh -c rg foo", 1.0, 10),
            mk(1003, 1002, "rg", "/opt/homebrew/bin/rg foo", 40.0, 80),
            // Separate Ollama session
            mk(2000, 1, "ollama", "/usr/local/bin/ollama serve", 2.0, 100),
            mk(2001, 2000, "ollama_llama_server", "/usr/local/bin/ollama_llama_server --model llama3", 120.0, 4200),
            // Unrelated app
            mk(3000, 1, "Notes", "/System/Applications/Notes.app/Contents/MacOS/Notes", 0.2, 60),
        ];

        let snap = detect_ai_agents(&procs);
        assert_eq!(snap.sessions.len(), 2);

        // Ollama has higher combined CPU (122% vs 61%) so it sorts first.
        let ollama = &snap.sessions[0];
        assert_eq!(ollama.kind, AgentKind::Ollama);
        assert_eq!(ollama.total_process_count, 2);
        assert!((ollama.cpu_percent - 122.0).abs() < 0.01);
        assert_eq!(ollama.memory_bytes, 4300 * 1_048_576);
        assert_eq!(ollama.impact_label, ImpactLabel::Heavy);

        let claude = &snap.sessions[1];
        assert_eq!(claude.kind, AgentKind::Claude);
        assert_eq!(claude.root_pid, 1000);
        // Should include MCP server (1001), zsh (1002), and grandchild rg (1003) without double-counting!
        assert_eq!(claude.total_process_count, 4);
        assert!((claude.cpu_percent - 61.0).abs() < 0.01);
        assert_eq!(claude.memory_bytes, 460 * 1_048_576);
    }
}
