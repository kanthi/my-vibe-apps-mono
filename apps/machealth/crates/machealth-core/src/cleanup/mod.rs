use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::{CString, c_void};
use std::fs;
use std::mem;
use std::path::{Path, PathBuf};
use std::ptr;
use std::time::{Duration, SystemTime};

#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {
    fn objc_getClass(name: *const libc::c_char) -> *mut c_void;
    fn sel_registerName(name: *const libc::c_char) -> *mut c_void;
    fn objc_msgSend();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileKind {
    Documents,
    Images,
    Videos,
    Audio,
    Archives,
    AppsAndInstallers,
    DeveloperAndCode,
    Other,
}

impl FileKind {
    pub const ALL: [FileKind; 8] = [
        FileKind::Documents,
        FileKind::Images,
        FileKind::Videos,
        FileKind::Audio,
        FileKind::Archives,
        FileKind::AppsAndInstallers,
        FileKind::DeveloperAndCode,
        FileKind::Other,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Documents => "Documents",
            Self::Images => "Images",
            Self::Videos => "Videos",
            Self::Audio => "Audio",
            Self::Archives => "Archives",
            Self::AppsAndInstallers => "Apps & Installers",
            Self::DeveloperAndCode => "Developer & Code",
            Self::Other => "Other",
        }
    }

    pub fn from_path(path: &Path) -> Self {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match ext.as_str() {
            "pdf" | "doc" | "docx" | "pages" | "txt" | "md" | "markdown" | "rtf" | "xls"
            | "xlsx" | "numbers" | "ppt" | "pptx" | "key" | "csv" | "epub" | "odt" => {
                Self::Documents
            }
            "png" | "jpg" | "jpeg" | "heic" | "heif" | "webp" | "gif" | "svg" | "tif" | "tiff"
            | "bmp" | "ico" | "raw" | "cr2" | "nef" | "dng" | "psd" | "ai" | "fig" | "sketch" => {
                Self::Images
            }
            "mp4" | "mov" | "mkv" | "avi" | "webm" | "m4v" | "wmv" | "flv" | "mpeg" | "mpg" => {
                Self::Videos
            }
            "mp3" | "wav" | "flac" | "m4a" | "aac" | "ogg" | "aiff" | "aif" | "opus" | "wma" => {
                Self::Audio
            }
            "zip" | "tar" | "gz" | "tgz" | "xz" | "bz2" | "7z" | "rar" | "zst" | "iso" => {
                Self::Archives
            }
            "dmg" | "pkg" | "app" | "ipa" | "xip" | "mpkg" => Self::AppsAndInstallers,
            "rs" | "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "py" | "swift" | "go" | "c"
            | "h" | "cc" | "cpp" | "hpp" | "m" | "mm" | "java" | "kt" | "rb" | "sh" | "zsh"
            | "bash" | "json" | "yaml" | "yml" | "toml" | "xml" | "sql" | "html" | "css"
            | "scss" | "nix" | "lock" | "wasm" => Self::DeveloperAndCode,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntrySummary {
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
    pub kind: FileKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileKindBucket {
    pub kind: FileKind,
    pub label: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub largest_files: Vec<FileEntrySummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomeFilesSummary {
    pub scanned_roots: Vec<PathBuf>,
    pub total_files: usize,
    pub total_bytes: u64,
    pub truncated: bool,
    pub buckets: Vec<FileKindBucket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanUpCategory {
    OldInstaller,
    ForgottenLargeDownload,
    SafeAppCache,
    XcodeDerivedData,
}

impl CleanUpCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::OldInstaller => "Old Installer",
            Self::ForgottenLargeDownload => "Forgotten Download",
            Self::SafeAppCache => "Safe App Cache",
            Self::XcodeDerivedData => "Xcode DerivedData",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanUpCandidate {
    pub path: PathBuf,
    pub name: String,
    pub category: CleanUpCategory,
    pub size_bytes: u64,
    pub modified_at: Option<DateTime<Utc>>,
    pub is_directory: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TrashSummary {
    pub item_count: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanUpScanReport {
    pub candidates: Vec<CleanUpCandidate>,
    pub total_reclaimable_bytes: u64,
    pub trash_summary: TrashSummary,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanUpConfirmation {
    pub title: String,
    pub item_count: usize,
    pub total_bytes: u64,
    pub target_names: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrashMoveOutcome {
    pub moved_to_trash: Vec<PathBuf>,
    pub reclaimed_bytes: u64,
    pub refused_or_failed: Vec<(PathBuf, String)>,
}

#[derive(Debug, Clone, Copy)]
pub struct CleanUpScanConfig {
    pub forgotten_download_min_age_days: u64,
    pub forgotten_download_min_bytes: u64,
    pub safe_cache_min_bytes: u64,
}

impl Default for CleanUpScanConfig {
    fn default() -> Self {
        Self {
            forgotten_download_min_age_days: 30,
            forgotten_download_min_bytes: 25 * 1_000_000, // 25 MB
            safe_cache_min_bytes: 10 * 1_000_000,         // 10 MB
        }
    }
}

const SKIP_DIR_NAMES: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    "target",
    ".venv",
    "venv",
    "__pycache__",
    ".Trash",
];

/// Scans standard user home folders (`Desktop`, `Documents`, `Downloads`, `Movies`, `Music`, `Pictures`)
/// and categorizes files by kind.
pub fn scan_home_files_default(max_entries: usize) -> HomeFilesSummary {
    let Some(home) = home_dir() else {
        return HomeFilesSummary {
            scanned_roots: vec![],
            total_files: 0,
            total_bytes: 0,
            truncated: false,
            buckets: vec![],
        };
    };

    let roots: Vec<PathBuf> = [
        "Desktop",
        "Documents",
        "Downloads",
        "Movies",
        "Music",
        "Pictures",
    ]
    .into_iter()
    .map(|sub| home.join(sub))
    .filter(|p| p.is_dir())
    .collect();

    scan_files_by_kind(&roots, max_entries, 6)
}

pub fn scan_files_by_kind(
    roots: &[PathBuf],
    max_entries: usize,
    max_depth: usize,
) -> HomeFilesSummary {
    let mut bucket_map: HashMap<FileKind, (usize, u64, Vec<FileEntrySummary>)> = FileKind::ALL
        .into_iter()
        .map(|k| (k, (0, 0, Vec::new())))
        .collect();

    let mut visited = 0usize;
    let mut truncated = false;
    let mut total_files = 0usize;
    let mut total_bytes = 0u64;

    let mut stack: Vec<(PathBuf, usize)> = roots.iter().cloned().map(|r| (r, 0)).collect();

    while let Some((dir, depth)) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            visited += 1;
            if visited > max_entries {
                truncated = true;
                break;
            }
            let path = entry.path();
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.file_type().is_symlink() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().into_owned();
            if meta.is_dir() {
                // Treat `.app` bundles as single Apps & Installers entries without descending deeply.
                if name.ends_with(".app") {
                    let size = directory_size_bounded(&path, 500);
                    record_file(&mut bucket_map, FileKind::AppsAndInstallers, path, name, size);
                    total_files += 1;
                    total_bytes = total_bytes.saturating_add(size);
                    continue;
                }
                if depth < max_depth && !name.starts_with('.') && !SKIP_DIR_NAMES.contains(&name.as_str()) {
                    stack.push((path, depth + 1));
                }
            } else if meta.is_file() {
                if name == ".DS_Store" {
                    continue;
                }
                let size = meta.len();
                let kind = FileKind::from_path(&path);
                record_file(&mut bucket_map, kind, path, name, size);
                total_files += 1;
                total_bytes = total_bytes.saturating_add(size);
            }
        }
        if truncated {
            break;
        }
    }

    let mut buckets: Vec<FileKindBucket> = FileKind::ALL
        .into_iter()
        .map(|kind| {
            let (file_count, bucket_bytes, mut largest) = bucket_map.remove(&kind).unwrap_or_default();
            largest.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
            largest.truncate(5);
            FileKindBucket {
                kind,
                label: kind.label().to_string(),
                file_count,
                total_bytes: bucket_bytes,
                largest_files: largest,
            }
        })
        .collect();
    buckets.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));

    HomeFilesSummary {
        scanned_roots: roots.to_vec(),
        total_files,
        total_bytes,
        truncated,
        buckets,
    }
}

fn record_file(
    map: &mut HashMap<FileKind, (usize, u64, Vec<FileEntrySummary>)>,
    kind: FileKind,
    path: PathBuf,
    name: String,
    size_bytes: u64,
) {
    let entry = map.entry(kind).or_insert_with(|| (0, 0, Vec::new()));
    entry.0 += 1;
    entry.1 = entry.1.saturating_add(size_bytes);
    // Keep top-5 candidates bounded in memory while scanning.
    if entry.2.len() < 10 || entry.2.last().is_some_and(|l| size_bytes > l.size_bytes) {
        entry.2.push(FileEntrySummary {
            path,
            name,
            size_bytes,
            kind,
        });
        if entry.2.len() > 16 {
            entry.2.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
            entry.2.truncate(5);
        }
    }
}

/// Scans standard macOS user locations for safe-to-review Clean Up candidates.
pub fn scan_cleanup_default(config: &CleanUpScanConfig) -> CleanUpScanReport {
    let Some(home) = home_dir() else {
        return CleanUpScanReport {
            candidates: vec![],
            total_reclaimable_bytes: 0,
            trash_summary: TrashSummary::default(),
            summary: "Home folder unavailable".into(),
        };
    };

    scan_cleanup_roots(
        &home.join("Downloads"),
        &home.join("Library/Caches"),
        &home.join("Library/Developer/Xcode/DerivedData"),
        &home.join(".Trash"),
        config,
    )
}

pub fn scan_cleanup_roots(
    downloads_dir: &Path,
    caches_dir: &Path,
    xcode_derived_data_dir: &Path,
    trash_dir: &Path,
    config: &CleanUpScanConfig,
) -> CleanUpScanReport {
    let mut candidates = Vec::new();
    let now = SystemTime::now();
    let old_cutoff = now
        .checked_sub(Duration::from_secs(
            config.forgotten_download_min_age_days * 86_400,
        ))
        .unwrap_or(SystemTime::UNIX_EPOCH);

    // 1 & 2. Downloads: old installers (.dmg, .pkg) & forgotten large downloads (>30d)
    if let Ok(entries) = fs::read_dir(downloads_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let modified_sys = meta.modified().ok();
            let modified_at = modified_sys.map(DateTime::<Utc>::from);
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();

            if meta.is_file() && matches!(ext.as_str(), "dmg" | "pkg" | "mpkg") {
                candidates.push(CleanUpCandidate {
                    path,
                    name,
                    category: CleanUpCategory::OldInstaller,
                    size_bytes: meta.len(),
                    modified_at,
                    is_directory: false,
                    reason: "Installer image/package in Downloads (usually unneeded after installing)".into(),
                });
            } else if meta.is_file()
                && meta.len() >= config.forgotten_download_min_bytes
                && modified_sys.is_some_and(|m| m < old_cutoff)
            {
                candidates.push(CleanUpCandidate {
                    path,
                    name,
                    category: CleanUpCategory::ForgottenLargeDownload,
                    size_bytes: meta.len(),
                    modified_at,
                    is_directory: false,
                    reason: format!(
                        "Large file in Downloads untouched for over {} days",
                        config.forgotten_download_min_age_days
                    ),
                });
            }
        }
    }

    // 3. Safe user app caches (skip Apple system caches & sensitive dirs)
    if let Ok(entries) = fs::read_dir(caches_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if !is_safe_user_cache_entry(&name) {
                continue;
            }
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            let size_bytes = if meta.is_dir() {
                directory_size_bounded(&path, 4_000)
            } else {
                meta.len()
            };
            if size_bytes >= config.safe_cache_min_bytes {
                candidates.push(CleanUpCandidate {
                    path,
                    name: name.clone(),
                    category: CleanUpCategory::SafeAppCache,
                    size_bytes,
                    modified_at: meta.modified().ok().map(DateTime::<Utc>::from),
                    is_directory: meta.is_dir(),
                    reason: format!("Rebuildable app cache for {name}"),
                });
            }
        }
    }

    // 4. Xcode DerivedData project folders
    if let Ok(entries) = fs::read_dir(xcode_derived_data_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name == "ModuleCache.noindex" {
                continue;
            }
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.is_dir() && !meta.file_type().is_symlink() {
                let size_bytes = directory_size_bounded(&path, 5_000);
                if size_bytes > 0 {
                    candidates.push(CleanUpCandidate {
                        path,
                        name,
                        category: CleanUpCategory::XcodeDerivedData,
                        size_bytes,
                        modified_at: meta.modified().ok().map(DateTime::<Utc>::from),
                        is_directory: true,
                        reason: "Xcode build intermediates (Xcode rebuilds automatically when needed)".into(),
                    });
                }
            }
        }
    }

    // 5. Trash summary (informational; MacHealth never empties Trash silently)
    let trash_summary = summarize_trash(trash_dir);

    candidates.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    let total_reclaimable_bytes: u64 = candidates.iter().map(|c| c.size_bytes).sum();

    let summary = if candidates.is_empty() {
        "No obvious clutter found to clean up".into()
    } else {
        let mb = total_reclaimable_bytes as f64 / 1_000_000.0;
        if mb >= 1000.0 {
            format!(
                "Found {} items ({:.2} GB) safe to review",
                candidates.len(),
                mb / 1000.0
            )
        } else {
            format!("Found {} items ({:.0} MB) safe to review", candidates.len(), mb)
        }
    };

    CleanUpScanReport {
        candidates,
        total_reclaimable_bytes,
        trash_summary,
        summary,
    }
}

fn is_safe_user_cache_entry(name: &str) -> bool {
    if name.starts_with('.') {
        return false;
    }
    let lower = name.to_ascii_lowercase();
    if lower.starts_with("com.apple.")
        || lower == "cloudkit"
        || lower == "familycircle"
        || lower == "geoservices"
        || lower == "passkit"
        || lower == "animoji"
        || lower.contains("machealth")
    {
        return false;
    }
    true
}

fn summarize_trash(trash_dir: &Path) -> TrashSummary {
    let mut item_count = 0usize;
    let mut total_bytes = 0u64;
    if let Ok(entries) = fs::read_dir(trash_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name == ".DS_Store" {
                continue;
            }
            let path = entry.path();
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            item_count += 1;
            if meta.is_dir() && !meta.file_type().is_symlink() {
                total_bytes = total_bytes.saturating_add(directory_size_bounded(&path, 2_000));
            } else {
                total_bytes = total_bytes.saturating_add(meta.len());
            }
        }
    }
    TrashSummary {
        item_count,
        total_bytes,
    }
}

fn directory_size_bounded(root: &Path, max_entries: usize) -> u64 {
    let mut total = 0u64;
    let mut seen = 0usize;
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            seen += 1;
            if seen > max_entries {
                return total;
            }
            let path = entry.path();
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_dir() {
                stack.push(path);
            } else {
                total = total.saturating_add(meta.len());
            }
        }
    }
    total
}

/// Builds a mandatory confirmation prompt before moving any files to Trash.
pub fn build_cleanup_confirmation(items: &[CleanUpCandidate]) -> CleanUpConfirmation {
    let total_bytes: u64 = items.iter().map(|i| i.size_bytes).sum();
    let target_names: Vec<String> = items.iter().map(|i| i.name.clone()).collect();
    let mb = total_bytes as f64 / 1_000_000.0;
    let size_label = if mb >= 1000.0 {
        format!("{:.2} GB", mb / 1000.0)
    } else {
        format!("{:.1} MB", mb)
    };

    CleanUpConfirmation {
        title: format!(
            "Move {} {} to Trash?",
            items.len(),
            if items.len() == 1 { "item" } else { "items" }
        ),
        item_count: items.len(),
        total_bytes,
        target_names,
        message: format!(
            "Selected items ({size_label}) will be moved to macOS Trash and can be recovered if needed."
        ),
    }
}

/// Validates that a path is safe to move to Trash (never system directories or home root).
pub fn validate_trash_target(path: &Path) -> Result<(), String> {
    if !path.is_absolute() {
        return Err("Refused relative path".to_string());
    }
    let path_str = path.to_string_lossy();
    let protected_exact = ["/", "/System", "/Library", "/Applications", "/Users", "/usr", "/bin", "/sbin", "/etc", "/var"];
    if protected_exact.contains(&path_str.as_ref()) {
        return Err(format!("Refused protected macOS path: {path_str}"));
    }
    if path_str.starts_with("/System/")
        || path_str.starts_with("/usr/")
        || path_str.starts_with("/bin/")
        || path_str.starts_with("/sbin/")
    {
        return Err(format!("Refused protected macOS system path: {path_str}"));
    }
    if let Some(home) = home_dir() {
        if path == home || path == home.join(".Trash") || path == home.join("Library") {
            return Err(format!("Refused protected home folder root: {path_str}"));
        }
    }
    if !path.exists() {
        return Err(format!("File no longer exists: {path_str}"));
    }
    Ok(())
}

/// Moves confirmed user-selected paths to macOS Trash (recoverable — never permanent `rm`).
pub fn move_items_to_trash(paths: &[PathBuf]) -> TrashMoveOutcome {
    let mut moved_to_trash = Vec::new();
    let mut reclaimed_bytes = 0u64;
    let mut refused_or_failed = Vec::new();

    for path in paths {
        if let Err(reason) = validate_trash_target(path) {
            refused_or_failed.push((path.clone(), reason));
            continue;
        }
        let size = fs::symlink_metadata(path)
            .map(|m| {
                if m.is_dir() {
                    directory_size_bounded(path, 5_000)
                } else {
                    m.len()
                }
            })
            .unwrap_or(0);

        match move_single_to_macos_trash(path) {
            Ok(()) => {
                moved_to_trash.push(path.clone());
                reclaimed_bytes = reclaimed_bytes.saturating_add(size);
            }
            Err(err) => refused_or_failed.push((path.clone(), err)),
        }
    }

    TrashMoveOutcome {
        moved_to_trash,
        reclaimed_bytes,
        refused_or_failed,
    }
}

fn move_single_to_macos_trash(path: &Path) -> Result<(), String> {
    if ns_file_manager_trash_item(path).unwrap_or(false) {
        return Ok(());
    }
    // Safe fallback: rename into ~/.Trash with collision-safe name (still recoverable, never deletes).
    let trash_dir = home_dir()
        .ok_or_else(|| "HOME directory unavailable".to_string())?
        .join(".Trash");
    move_to_trash_dir_fallback(path, &trash_dir)
}

fn move_to_trash_dir_fallback(path: &Path, trash_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(trash_dir).map_err(|e| format!("failed to access Trash: {e}"))?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "invalid file name".to_string())?;
    let mut dest = trash_dir.join(file_name);
    if dest.exists() {
        let ts = Utc::now().timestamp_millis();
        dest = trash_dir.join(format!("{file_name}-{ts}"));
    }
    fs::rename(path, &dest).map_err(|e| format!("failed to move item to Trash: {e}"))
}

fn ns_file_manager_trash_item(path: &Path) -> Option<bool> {
    let cpath = CString::new(path.to_str()?).ok()?;
    let ns_string_cls = CString::new("NSString").ok()?;
    let ns_url_cls = CString::new("NSURL").ok()?;
    let ns_fm_cls = CString::new("NSFileManager").ok()?;
    let sel_utf8 = CString::new("stringWithUTF8String:").ok()?;
    let sel_file_url = CString::new("fileURLWithPath:").ok()?;
    let sel_default_mgr = CString::new("defaultManager").ok()?;
    let sel_trash = CString::new("trashItemAtURL:resultingItemURL:error:").ok()?;

    unsafe {
        let str_cls = objc_getClass(ns_string_cls.as_ptr());
        let url_cls = objc_getClass(ns_url_cls.as_ptr());
        let fm_cls = objc_getClass(ns_fm_cls.as_ptr());
        if str_cls.is_null() || url_cls.is_null() || fm_cls.is_null() {
            return None;
        }

        let msg_str: unsafe extern "C" fn(*mut c_void, *mut c_void, *const libc::c_char) -> *mut c_void =
            mem::transmute(objc_msgSend as *const ());
        let ns_path = msg_str(str_cls, sel_registerName(sel_utf8.as_ptr()), cpath.as_ptr());
        if ns_path.is_null() {
            return None;
        }

        let msg_url: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
            mem::transmute(objc_msgSend as *const ());
        let url = msg_url(url_cls, sel_registerName(sel_file_url.as_ptr()), ns_path);
        if url.is_null() {
            return None;
        }

        let msg_mgr: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
            mem::transmute(objc_msgSend as *const ());
        let fm = msg_mgr(fm_cls, sel_registerName(sel_default_mgr.as_ptr()));
        if fm.is_null() {
            return None;
        }

        let msg_trash: unsafe extern "C" fn(
            *mut c_void,
            *mut c_void,
            *mut c_void,
            *mut *mut c_void,
            *mut *mut c_void,
        ) -> bool = mem::transmute(objc_msgSend as *const ());

        let mut resulting: *mut c_void = ptr::null_mut();
        let mut err: *mut c_void = ptr::null_mut();
        Some(msg_trash(
            fm,
            sel_registerName(sel_trash.as_ptr()),
            url,
            &mut resulting,
            &mut err,
        ))
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_file_kinds_by_extension() {
        assert_eq!(FileKind::from_path(Path::new("report.pdf")), FileKind::Documents);
        assert_eq!(FileKind::from_path(Path::new("photo.HEIC")), FileKind::Images);
        assert_eq!(FileKind::from_path(Path::new("clip.mov")), FileKind::Videos);
        assert_eq!(FileKind::from_path(Path::new("song.flac")), FileKind::Audio);
        assert_eq!(FileKind::from_path(Path::new("backup.tar.gz")), FileKind::Archives);
        assert_eq!(FileKind::from_path(Path::new("Chrome.dmg")), FileKind::AppsAndInstallers);
        assert_eq!(FileKind::from_path(Path::new("main.rs")), FileKind::DeveloperAndCode);
        assert_eq!(FileKind::from_path(Path::new("blob.bin")), FileKind::Other);
    }

    #[test]
    fn scans_files_by_kind_and_cleanup_candidates_in_temp_sandbox() {
        let base = std::env::temp_dir().join(format!("machealth_cleanup_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let downloads = base.join("Downloads");
        let caches = base.join("Caches");
        let xcode = base.join("DerivedData");
        let trash = base.join("Trash");
        fs::create_dir_all(&downloads).unwrap();
        fs::create_dir_all(caches.join("com.apple.protected")).unwrap();
        fs::create_dir_all(caches.join("SomeThirdPartyApp")).unwrap();
        fs::create_dir_all(xcode.join("MyApp-abcdef")).unwrap();
        fs::create_dir_all(&trash).unwrap();

        fs::write(downloads.join("Installer.dmg"), vec![1u8; 4096]).unwrap();
        fs::write(downloads.join("notes.pdf"), vec![2u8; 1024]).unwrap();
        fs::write(caches.join("com.apple.protected/data.bin"), vec![3u8; 5000]).unwrap();
        fs::write(caches.join("SomeThirdPartyApp/cache.bin"), vec![4u8; 5000]).unwrap();
        fs::write(xcode.join("MyApp-abcdef/build.o"), vec![5u8; 2048]).unwrap();
        fs::write(trash.join("deleted.txt"), vec![6u8; 512]).unwrap();

        let home_summary = scan_files_by_kind(&[downloads.clone()], 1000, 4);
        assert_eq!(home_summary.total_files, 2);
        assert_eq!(home_summary.total_bytes, 5120);

        let cfg = CleanUpScanConfig {
            forgotten_download_min_age_days: 30,
            forgotten_download_min_bytes: 1000,
            safe_cache_min_bytes: 1000,
        };
        let report = scan_cleanup_roots(&downloads, &caches, &xcode, &trash, &cfg);
        assert_eq!(report.candidates.len(), 3); // Installer.dmg, SomeThirdPartyApp cache, MyApp-abcdef (com.apple.protected skipped!)
        assert!(!report.candidates.iter().any(|c| c.name == "com.apple.protected"));
        assert_eq!(report.trash_summary.item_count, 1);
        assert_eq!(report.trash_summary.total_bytes, 512);

        // Verify confirmation builder & trash fallback (without polluting user's real ~/.Trash)
        let conf = build_cleanup_confirmation(&report.candidates);
        assert_eq!(conf.item_count, 3);
        let victim = downloads.join("Installer.dmg");
        move_to_trash_dir_fallback(&victim, &trash).expect("move to sandbox trash");
        assert!(!victim.exists());
        assert!(trash.join("Installer.dmg").exists());

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn guardrails_refuse_protected_system_paths_for_trash() {
        assert!(validate_trash_target(Path::new("/")).is_err());
        assert!(validate_trash_target(Path::new("/System/Library")).is_err());
        assert!(validate_trash_target(Path::new("/Applications")).is_err());
        assert!(validate_trash_target(Path::new("relative/file.txt")).is_err());
    }
}
