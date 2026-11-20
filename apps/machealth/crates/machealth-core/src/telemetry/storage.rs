use serde::{Deserialize, Serialize};
use std::ffi::{CString, c_void};
use std::mem;
use std::process::Command;
use std::ptr;

#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {
    fn objc_getClass(name: *const libc::c_char) -> *mut c_void;
    fn sel_registerName(name: *const libc::c_char) -> *mut c_void;
    fn objc_msgSend();
    static NSURLVolumeAvailableCapacityForImportantUsageKey: *mut c_void;
    static NSURLVolumeTotalCapacityKey: *mut c_void;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SmartStatus {
    Verified,
    Failing,
    NotSupported,
    Unknown,
}

impl SmartStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Verified => "Verified",
            Self::Failing => "Failing",
            Self::NotSupported => "Not Supported",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageTelemetry {
    pub mount_point: String,
    pub total_bytes: u64,
    /// Available bytes as Finder reports them (includes purgeable APFS space).
    pub available_bytes: u64,
    /// Raw free blocks from `statfs` (excludes purgeable space).
    pub raw_free_bytes: u64,
    pub purgeable_bytes: u64,
    pub used_bytes: u64,
    pub used_percent: f32,
    pub free_percent: f32,
    pub is_solid_state: Option<bool>,
    pub smart_status: SmartStatus,
    pub ssd_wear_percent: Option<u8>,
    pub summary: String,
}

pub fn collect_storage() -> Result<StorageTelemetry, String> {
    let (statfs_total, statfs_free) = read_statfs("/")?;
    let (ns_total, ns_available) = read_nsurl_volume_capacity("/").unwrap_or((None, None));
    let diskutil = read_diskutil_info("/").unwrap_or_default();

    let total_bytes = ns_total
        .or(diskutil.container_total_bytes)
        .unwrap_or(statfs_total);
    let raw_free_bytes = diskutil.container_free_bytes.unwrap_or(statfs_free);
    let available_bytes = ns_available.unwrap_or(raw_free_bytes).min(total_bytes);
    let purgeable_bytes = available_bytes.saturating_sub(raw_free_bytes);
    let used_bytes = total_bytes.saturating_sub(available_bytes);

    let (used_percent, free_percent) = if total_bytes > 0 {
        let used = (used_bytes as f64 / total_bytes as f64 * 100.0) as f32;
        let free = (available_bytes as f64 / total_bytes as f64 * 100.0) as f32;
        (used, free)
    } else {
        (0.0, 0.0)
    };

    let available_gb = available_bytes as f64 / 1_000_000_000.0;
    let summary = if diskutil.smart_status == SmartStatus::Failing {
        "Drive health warning — back up immediately".to_string()
    } else if available_gb < 15.0 || free_percent < 8.0 {
        format!("Storage is critically low ({available_gb:.1} GB free)")
    } else if available_gb < 40.0 || free_percent < 15.0 {
        format!("Storage is getting low ({available_gb:.1} GB free)")
    } else {
        format!("Plenty of space free ({available_gb:.1} GB available)")
    };

    Ok(StorageTelemetry {
        mount_point: "/".to_string(),
        total_bytes,
        available_bytes,
        raw_free_bytes,
        purgeable_bytes,
        used_bytes,
        used_percent,
        free_percent,
        is_solid_state: diskutil.is_solid_state,
        smart_status: diskutil.smart_status,
        ssd_wear_percent: None,
        summary,
    })
}

fn read_statfs(path: &str) -> Result<(u64, u64), String> {
    let cpath = CString::new(path).map_err(|e| e.to_string())?;
    let mut stat: libc::statfs = unsafe { mem::zeroed() };
    let ret = unsafe { libc::statfs(cpath.as_ptr(), &mut stat) };
    if ret != 0 {
        return Err(format!("statfs({path}) failed with code {ret}"));
    }
    let block_size = stat.f_bsize as u64;
    let total = stat.f_blocks.saturating_mul(block_size);
    let free = stat.f_bavail.saturating_mul(block_size);
    Ok((total, free))
}

fn read_nsurl_volume_capacity(path: &str) -> Option<(Option<u64>, Option<u64>)> {
    let cpath = CString::new(path).ok()?;
    let ns_string_cls_name = CString::new("NSString").ok()?;
    let ns_url_cls_name = CString::new("NSURL").ok()?;
    let sel_str_utf8 = CString::new("stringWithUTF8String:").ok()?;
    let sel_file_url = CString::new("fileURLWithPath:").ok()?;
    let sel_get_res = CString::new("getResourceValue:forKey:error:").ok()?;
    let sel_u64 = CString::new("unsignedLongLongValue").ok()?;

    unsafe {
        let ns_string_cls = objc_getClass(ns_string_cls_name.as_ptr());
        let ns_url_cls = objc_getClass(ns_url_cls_name.as_ptr());
        if ns_string_cls.is_null() || ns_url_cls.is_null() {
            return None;
        }

        let msg_str: unsafe extern "C" fn(*mut c_void, *mut c_void, *const libc::c_char) -> *mut c_void =
            mem::transmute(objc_msgSend as *const ());
        let ns_path = msg_str(ns_string_cls, sel_registerName(sel_str_utf8.as_ptr()), cpath.as_ptr());
        if ns_path.is_null() {
            return None;
        }

        let msg_url: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
            mem::transmute(objc_msgSend as *const ());
        let url = msg_url(ns_url_cls, sel_registerName(sel_file_url.as_ptr()), ns_path);
        if url.is_null() {
            return None;
        }

        let get_res_sel = sel_registerName(sel_get_res.as_ptr());
        let u64_sel = sel_registerName(sel_u64.as_ptr());
        let msg_get_res: unsafe extern "C" fn(
            *mut c_void,
            *mut c_void,
            *mut *mut c_void,
            *mut c_void,
            *mut *mut c_void,
        ) -> bool = mem::transmute(objc_msgSend as *const ());
        let msg_u64: unsafe extern "C" fn(*mut c_void, *mut c_void) -> u64 =
            mem::transmute(objc_msgSend as *const ());

        let mut total_obj: *mut c_void = ptr::null_mut();
        let mut err_obj: *mut c_void = ptr::null_mut();
        let total = if msg_get_res(
            url,
            get_res_sel,
            &mut total_obj,
            NSURLVolumeTotalCapacityKey,
            &mut err_obj,
        ) && !total_obj.is_null()
        {
            Some(msg_u64(total_obj, u64_sel))
        } else {
            None
        };

        let mut avail_obj: *mut c_void = ptr::null_mut();
        err_obj = ptr::null_mut();
        let avail = if msg_get_res(
            url,
            get_res_sel,
            &mut avail_obj,
            NSURLVolumeAvailableCapacityForImportantUsageKey,
            &mut err_obj,
        ) && !avail_obj.is_null()
        {
            Some(msg_u64(avail_obj, u64_sel))
        } else {
            None
        };

        Some((total, avail))
    }
}

#[derive(Debug, Default)]
struct DiskutilInfo {
    container_total_bytes: Option<u64>,
    container_free_bytes: Option<u64>,
    is_solid_state: Option<bool>,
    smart_status: SmartStatus,
}

impl Default for SmartStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

fn read_diskutil_info(mount: &str) -> Option<DiskutilInfo> {
    let output = Command::new("/usr/sbin/diskutil")
        .args(["info", mount])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Some(parse_diskutil_info(&text))
}

fn parse_diskutil_info(text: &str) -> DiskutilInfo {
    let mut info = DiskutilInfo::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "Container Total Space" | "Disk Size" => {
                if info.container_total_bytes.is_none() {
                    info.container_total_bytes = extract_bytes_in_parens(value);
                }
            }
            "Container Free Space" => {
                info.container_free_bytes = extract_bytes_in_parens(value);
            }
            "Solid State" => {
                info.is_solid_state = Some(value.eq_ignore_ascii_case("yes"));
            }
            "SMART Status" => {
                info.smart_status = match value {
                    v if v.eq_ignore_ascii_case("Verified") => SmartStatus::Verified,
                    v if v.eq_ignore_ascii_case("Failing") => SmartStatus::Failing,
                    v if v.eq_ignore_ascii_case("Not Supported") => SmartStatus::NotSupported,
                    _ => SmartStatus::Unknown,
                };
            }
            _ => {}
        }
    }
    info
}

fn extract_bytes_in_parens(value: &str) -> Option<u64> {
    // e.g. "494.4 GB (494384795648 Bytes) (exactly ...)"
    let start = value.find('(')?;
    let rest = &value[start + 1..];
    let end = rest.find(" Bytes)")?;
    rest[..end].trim().parse::<u64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_diskutil_apfs_output() {
        let sample = "\
   Device Node:               /dev/disk3s1s1
   SMART Status:              Verified
   Container Total Space:     494.4 GB (494384795648 Bytes) (exactly 965595304 512-Byte-Units)
   Container Free Space:      113.4 GB (113442078720 Bytes) (exactly 221566560 512-Byte-Units)
   Solid State:               Yes
";
        let info = parse_diskutil_info(sample);
        assert_eq!(info.container_total_bytes, Some(494_384_795_648));
        assert_eq!(info.container_free_bytes, Some(113_442_078_720));
        assert_eq!(info.is_solid_state, Some(true));
        assert_eq!(info.smart_status, SmartStatus::Verified);
    }

    #[test]
    fn reads_live_macos_storage() {
        let storage = collect_storage().expect("live storage");
        assert!(storage.total_bytes > 10 * 1_000_000_000);
        assert!(storage.available_bytes <= storage.total_bytes);
        assert!(storage.available_bytes >= storage.raw_free_bytes);
    }
}
