use crate::cleanup::FileKind;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuickSearchOptions {
    pub limit: usize,
    pub only_in_dir: Option<PathBuf>,
    pub kind_filter: Option<FileKind>,
    pub filename_only: bool,
}

impl Default for QuickSearchOptions {
    fn default() -> Self {
        Self {
            limit: 25,
            only_in_dir: None,
            kind_filter: None,
            filename_only: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: PathBuf,
    pub name: String,
    pub kind: FileKind,
    pub size_bytes: u64,
    pub modified_at: Option<DateTime<Utc>>,
    pub parent_display: String,
}

/// Instant Spotlight file search powered by macOS `/usr/bin/mdfind`.
pub fn quick_search(query: &str, options: &QuickSearchOptions) -> Result<Vec<SearchResult>, String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut cmd = Command::new("/usr/bin/mdfind");
    if let Some(ref dir) = options.only_in_dir {
        cmd.arg("-onlyin").arg(dir);
    }

    if options.filename_only {
        cmd.arg("-name").arg(trimmed);
    } else {
        let escaped = sanitize_spotlight_literal(trimmed);
        let expr = format!(
            "kMDItemFSName == '*{escaped}*'cd || kMDItemTextContent == '*{escaped}*'cd"
        );
        cmd.arg(expr);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("failed to run mdfind: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("mdfind exited with {}: {}", output.status, stderr.trim()));
    }

    // Important: only read stdout; mdfind prints locale parser notes to stderr.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let results = parse_mdfind_results(&stdout, trimmed, options);
    Ok(results)
}

pub fn sanitize_spotlight_literal(raw: &str) -> String {
    raw.chars()
        .filter(|&c| c != '\0' && c != '\n' && c != '\r')
        .flat_map(|c| match c {
            '\\' | '\'' | '"' | '*' => vec!['\\', c],
            other => vec![other],
        })
        .collect()
}

fn parse_mdfind_results(
    stdout: &str,
    query: &str,
    options: &QuickSearchOptions,
) -> Vec<SearchResult> {
    let home_str = std::env::var("HOME").ok();
    let query_lower = query.to_ascii_lowercase();
    let max_inspect = (options.limit * 6).max(100);

    let mut items = Vec::new();
    for line in stdout.lines().take(max_inspect) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let path = PathBuf::from(line);
        let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_string) else {
            continue;
        };
        let kind = FileKind::from_path(&path);
        if let Some(filter) = options.kind_filter
            && kind != filter
        {
            continue;
        }
        let meta = fs::metadata(&path).ok();
        let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified_at = meta
            .and_then(|m| m.modified().ok())
            .map(DateTime::<Utc>::from);

        let parent_display = path
            .parent()
            .map(|p| {
                let s = p.to_string_lossy();
                if let Some(ref h) = home_str
                    && let Some(stripped) = s.strip_prefix(h.as_str())
                {
                    return format!("~{stripped}");
                }
                s.into_owned()
            })
            .unwrap_or_default();

        items.push(SearchResult {
            path,
            name,
            kind,
            size_bytes,
            modified_at,
            parent_display,
        });
    }

    // Rank exact / prefix filename matches first, then most recently modified.
    items.sort_by(|a, b| {
        let rank = |name: &str| -> u8 {
            let lower = name.to_ascii_lowercase();
            if lower == query_lower {
                0
            } else if lower.starts_with(&query_lower) {
                1
            } else if lower.contains(&query_lower) {
                2
            } else {
                3
            }
        };
        rank(&a.name)
            .cmp(&rank(&b.name))
            .then_with(|| b.modified_at.cmp(&a.modified_at))
            .then_with(|| a.name.cmp(&b.name))
    });

    items.truncate(options.limit);
    items
}

/// Opens a file with its default macOS app (`/usr/bin/open`).
pub fn open_file(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    let status = Command::new("/usr/bin/open")
        .arg(path)
        .status()
        .map_err(|e| format!("failed to open file: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("open exited with status {status}"))
    }
}

/// Reveals a file in Finder (`/usr/bin/open -R`).
pub fn reveal_in_finder(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    let status = Command::new("/usr/bin/open")
        .arg("-R")
        .arg(path)
        .status()
        .map_err(|e| format!("failed to reveal in Finder: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("open -R exited with status {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_spotlight_query_literals() {
        assert_eq!(sanitize_spotlight_literal("foo'bar*\\baz"), "foo\\'bar\\*\\\\baz");
    }

    #[test]
    fn live_spotlight_quick_search_finds_machealth_design_doc() {
        let repo_app_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("apps/machealth")
            .to_path_buf();

        let results = quick_search(
            "PRODUCT_DESIGN",
            &QuickSearchOptions {
                limit: 10,
                only_in_dir: Some(repo_app_dir),
                kind_filter: None,
                filename_only: true,
            },
        )
        .expect("mdfind search");

        assert!(
            results.iter().any(|r| r.name == "PRODUCT_DESIGN.md"),
            "expected PRODUCT_DESIGN.md in {:?}",
            results
        );
    }
}
