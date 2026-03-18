use std::collections::HashMap;
use std::fs::{self, File as StdFile, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(windows)]
use std::os::windows::io::AsRawHandle;

#[cfg(unix)]
use cross_path::platform::unix::UnixPathExt;
#[cfg(unix)]
use cross_path::platform::PathExt;
use cross_path::CrossPath;
#[cfg(any(target_os = "linux", target_os = "windows"))]
use eject::{device::Device, discovery::cd_drives};
use filetime::{set_file_times, FileTime};
use glob::glob;
use rfd::FileDialog;
use serde_json::{json, Value};
use sys_info::disk_info;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{FILETIME, HANDLE};
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::SetFileTime;

use super::ModuleMethod;

fn format_timestamp_from_epoch(epoch_secs: u64) -> String {
    let days_since_epoch = (epoch_secs / 86_400) as i64;
    let seconds_of_day = (epoch_secs % 86_400) as u32;

    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };

    let hour = seconds_of_day / 3600;
    let minute = (seconds_of_day % 3600) / 60;
    let second = seconds_of_day % 60;

    format!(
        "{:04}{:02}{:02}{:02}{:02}{:02}",
        year as i32, m as u32, d as u32, hour, minute, second
    )
}

fn system_time_to_ahk(time: SystemTime) -> String {
    let secs = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format_timestamp_from_epoch(secs)
}

fn parse_ahk_timestamp(timestamp: Option<&str>) -> Option<SystemTime> {
    let raw = timestamp.unwrap_or("").trim();
    if raw.is_empty() {
        return Some(SystemTime::now());
    }

    let digits = raw
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.len() < 4 {
        return None;
    }

    let year = digits.get(0..4)?.parse::<i32>().ok()?;
    let month = digits
        .get(4..6)
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1)
        .clamp(1, 12);
    let day = digits
        .get(6..8)
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1)
        .clamp(1, 31);
    let hour = digits
        .get(8..10)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
        .min(23);
    let minute = digits
        .get(10..12)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
        .min(59);
    let second = digits
        .get(12..14)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
        .min(59);

    let mut days = 0i64;
    for y in 1970..year {
        let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
        days += if leap { 366 } else { 365 };
    }

    let month_lengths = [31i64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        if m == 2 {
            let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            days += if leap { 29 } else { 28 };
        } else {
            days += month_lengths[(m - 1) as usize];
        }
    }

    days += (day as i64) - 1;

    let total_secs = (days as u64) * 86_400 + hour * 3600 + minute * 60 + second;
    Some(UNIX_EPOCH + std::time::Duration::from_secs(total_secs))
}

fn parse_i32(value: Option<&str>) -> Option<i32> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .and_then(|v| v.parse::<i32>().ok())
}

fn parse_bool_flag(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim),
        Some("1") | Some("true") | Some("True") | Some("TRUE")
    )
}

fn has_glob_pattern(value: &str) -> bool {
    value.contains('*') || value.contains('?') || value.contains('[')
}

fn expand_paths(pattern: &str) -> Vec<PathBuf> {
    if pattern.trim().is_empty() {
        return Vec::new();
    }

    if has_glob_pattern(pattern) {
        return glob(pattern)
            .ok()
            .into_iter()
            .flat_map(|iter| iter.filter_map(Result::ok))
            .collect::<Vec<_>>();
    }

    let path = PathBuf::from(pattern);
    if path.exists() {
        vec![path]
    } else {
        Vec::new()
    }
}

fn attrs_from_std(path: &Path) -> String {
    let Ok(metadata) = fs::metadata(path) else {
        return String::new();
    };
    let mut attrs = String::new();
    if metadata.permissions().readonly() {
        attrs.push('R');
    }
    if metadata.is_dir() {
        attrs.push('D');
    }
    if attrs.is_empty() {
        attrs.push('N');
    }
    attrs
}

#[cfg(unix)]
fn attrs_from_cross(path: &str) -> String {
    let ext = UnixPathExt::new(path);
    let Some(attrs) = ext.get_attributes() else {
        return String::new();
    };
    let mut text = String::new();
    if attrs.is_readonly {
        text.push('R');
    }
    if attrs.is_hidden {
        text.push('H');
    }
    if attrs.is_directory {
        text.push('D');
    }
    if text.is_empty() {
        text.push('N');
    }
    text
}

#[cfg(not(unix))]
fn attrs_from_cross(path: &str) -> String {
    attrs_from_std(Path::new(path))
}

enum OpenHandle {
    Reader(StdFile),
    Writer(StdFile),
    ReadWrite(StdFile),
    Append(StdFile),
}

struct HandleStore {
    handles: HashMap<u64, OpenHandle>,
}

fn handle_store() -> &'static Mutex<HandleStore> {
    static STORE: OnceLock<Mutex<HandleStore>> = OnceLock::new();
    STORE.get_or_init(|| {
        Mutex::new(HandleStore {
            handles: HashMap::new(),
        })
    })
}

fn next_handle_id() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

fn file_time_by_kind(path: &str, which_time: Option<&str>) -> String {
    let which = which_time.unwrap_or("M").trim().to_ascii_uppercase();
    let Ok(metadata) = fs::metadata(path) else {
        return String::new();
    };
    let value = match which.as_str() {
        "C" => metadata.created().ok(),
        "A" => metadata.accessed().ok(),
        _ => metadata.modified().ok(),
    };
    value.map(system_time_to_ahk).unwrap_or_default()
}

fn walk_files(root: &Path, recurse: bool, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        out.push(path.clone());
        if recurse && path.is_dir() {
            walk_files(&path, recurse, out);
        }
    }
}

fn run_capture(command: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(command)
        .args(args)
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
            } else {
                None
            }
        })
}

fn trash_files_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".local/share/Trash/files")
}

fn move_to_trash(path: &Path) -> std::io::Result<()> {
    let trash = trash_files_dir();
    fs::create_dir_all(&trash)?;
    let base = path
        .file_name()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|| "item".to_string());
    let mut target = trash.join(&base);
    if target.exists() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        target = trash.join(format!("{base}.{stamp}"));
    }
    fs::rename(path, target)
}

fn empty_trash() -> std::io::Result<()> {
    let trash = trash_files_dir();
    if !trash.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(&trash)? {
        let path = entry?.path();
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn normalize_cd_device_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.len() == 3 {
        let bytes = trimmed.as_bytes();
        if (bytes[0] as char).is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'\\' {
            return trimmed[0..2].to_string();
        }
    }
    trimmed.to_string()
}

#[cfg(not(target_os = "windows"))]
fn normalize_cd_device_path(path: &str) -> String {
    path.trim().to_string()
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn resolve_cd_device_path(value: Option<&str>) -> Option<String> {
    if let Some(explicit) = value.map(str::trim).filter(|v| !v.is_empty()) {
        return Some(normalize_cd_device_path(explicit));
    }
    cd_drives()
        .next()
        .map(|v| v.to_string_lossy().to_string())
        .map(|v| normalize_cd_device_path(&v))
}

#[cfg(windows)]
fn set_creation_time(path: &Path, ft: FileTime) -> std::io::Result<()> {
    let file = OpenOptions::new().read(true).write(true).open(path)?;
    let secs = ft.unix_seconds() + 11_644_473_600;
    let nanos = ft.nanoseconds();
    let ticks = (secs as u64)
        .saturating_mul(10_000_000)
        .saturating_add((nanos as u64) / 100);
    let creation = FILETIME {
        dwLowDateTime: ticks as u32,
        dwHighDateTime: (ticks >> 32) as u32,
    };
    let ok = unsafe {
        SetFileTime(
            file.as_raw_handle() as HANDLE,
            &creation,
            std::ptr::null(),
            std::ptr::null(),
        )
    };
    if ok == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn set_creation_time(_path: &Path, _ft: FileTime) -> std::io::Result<()> {
    Err(std::io::Error::other(
        "creation time unsupported on this platform",
    ))
}

pub fn drive(sub_command: &str, value1: Option<&str>, _value2: Option<&str>) -> String {
    let cmd = sub_command.trim().to_ascii_lowercase();

    if cmd == "label" {
        return sys_info::os_type()
            .map(|v| {
                if v.is_empty() {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            })
            .unwrap_or_else(|_| "1".to_string());
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        if matches!(cmd.as_str(), "eject" | "lock" | "unlock") {
            let Some(path) = resolve_cd_device_path(value1) else {
                return "1".to_string();
            };
            let Ok(device) = Device::open(path) else {
                return "1".to_string();
            };
            let result = match cmd.as_str() {
                "eject" => device.eject(),
                "lock" => device.lock_ejection().map(|_| ()),
                "unlock" => device.retract(),
                _ => Ok(()),
            };
            return if result.is_ok() {
                "0".to_string()
            } else {
                "1".to_string()
            };
        }
    }

    #[cfg(target_os = "macos")]
    {
        if cmd == "eject" {
            let explicit = value1.map(str::trim).filter(|v| !v.is_empty());
            let success = if let Some(target) = explicit {
                std::process::Command::new("diskutil")
                    .arg("eject")
                    .arg(target)
                    .status()
                    .map(|v| v.success())
                    .unwrap_or(false)
            } else {
                std::process::Command::new("drutil")
                    .arg("tray")
                    .arg("eject")
                    .status()
                    .map(|v| v.success())
                    .unwrap_or(false)
                    || std::process::Command::new("drutil")
                        .arg("tray")
                        .arg("open")
                        .status()
                        .map(|v| v.success())
                        .unwrap_or(false)
            };
            return if success {
                "0".to_string()
            } else {
                "1".to_string()
            };
        }

        if cmd == "unlock" {
            let success = std::process::Command::new("drutil")
                .arg("tray")
                .arg("close")
                .status()
                .map(|v| v.success())
                .unwrap_or(false);
            return if success {
                "0".to_string()
            } else {
                "1".to_string()
            };
        }

        if cmd == "lock" {
            return "1".to_string();
        }
    }

    "".to_string()
}

pub fn drive_get(sub_command: &str, value: Option<&str>) -> String {
    let cmd = sub_command.trim().to_ascii_lowercase();
    match cmd.as_str() {
        "capacity" | "cap" => disk_info()
            .map(|d| (d.total / 1024).to_string())
            .unwrap_or_default(),
        "filesystem" | "fs" => {
            #[cfg(target_os = "linux")]
            {
                let path = value.unwrap_or("/");
                return run_capture("stat", &["-f", "-c", "%T", path]).unwrap_or_default();
            }
            #[cfg(not(target_os = "linux"))]
            {
                return String::new();
            }
        }
        "list" => {
            #[cfg(target_os = "linux")]
            {
                let Ok(text) = fs::read_to_string("/proc/mounts") else {
                    return String::new();
                };
                let mut mounts = Vec::new();
                for line in text.lines() {
                    let fields = line.split_whitespace().collect::<Vec<_>>();
                    if fields.len() >= 2 {
                        mounts.push(fields[1].to_string());
                    }
                }
                return mounts.join("\n");
            }
            #[cfg(target_os = "windows")]
            {
                let mut drives = Vec::new();
                for ch in b'A'..=b'Z' {
                    let drive = format!("{}:\\", ch as char);
                    if Path::new(&drive).exists() {
                        drives.push(format!("{}:", ch as char));
                    }
                }
                return drives.join("");
            }
            #[cfg(not(any(target_os = "linux", target_os = "windows")))]
            {
                return "/".to_string();
            }
        }
        "label" => {
            #[cfg(target_os = "linux")]
            {
                let path = value.unwrap_or("/");
                run_capture("lsblk", &["-no", "LABEL", path]).unwrap_or_default()
            }
            #[cfg(not(target_os = "linux"))]
            {
                String::new()
            }
        }
        "serial" => {
            #[cfg(target_os = "linux")]
            {
                let path = value.unwrap_or("/");
                run_capture("lsblk", &["-no", "SERIAL", path]).unwrap_or_default()
            }
            #[cfg(not(target_os = "linux"))]
            {
                String::new()
            }
        }
        "type" => sys_info::os_type()
            .map(|_| "Fixed".to_string())
            .unwrap_or_else(|_| "Unknown".to_string()),
        "status" => {
            let path = value.unwrap_or("/");
            if Path::new(path).exists() {
                "Ready".to_string()
            } else {
                "NotReady".to_string()
            }
        }
        "statuscd" => "not ready".to_string(),
        _ => String::new(),
    }
}

pub fn drive_space_free(path: Option<&str>) -> String {
    let _ = path;
    disk_info()
        .map(|d| (d.free / 1024).to_string())
        .unwrap_or_default()
}

pub fn file(method: &str, handle: &str, arg1: Option<&str>, _arg2: Option<&str>) -> String {
    let Ok(handle_id) = handle.trim().parse::<u64>() else {
        return String::new();
    };
    let mut store = handle_store().lock().expect("file handle mutex poisoned");
    let Some(open_handle) = store.handles.get_mut(&handle_id) else {
        return String::new();
    };

    match method.trim().to_ascii_lowercase().as_str() {
        "close" => {
            store.handles.remove(&handle_id);
            "1".to_string()
        }
        "read" => {
            let mut text = String::new();
            match open_handle {
                OpenHandle::Reader(file) | OpenHandle::ReadWrite(file) => {
                    if file.read_to_string(&mut text).is_ok() {
                        text
                    } else {
                        String::new()
                    }
                }
                _ => String::new(),
            }
        }
        "write" => {
            let payload = arg1.unwrap_or("");
            match open_handle {
                OpenHandle::Writer(file)
                | OpenHandle::ReadWrite(file)
                | OpenHandle::Append(file) => file
                    .write_all(payload.as_bytes())
                    .map(|_| payload.len().to_string())
                    .unwrap_or_default(),
                _ => String::new(),
            }
        }
        "tell" => match open_handle {
            OpenHandle::Reader(file)
            | OpenHandle::Writer(file)
            | OpenHandle::ReadWrite(file)
            | OpenHandle::Append(file) => file
                .stream_position()
                .map(|v| v.to_string())
                .unwrap_or_default(),
        },
        "seek" => {
            let offset = arg1.and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
            match open_handle {
                OpenHandle::Reader(file)
                | OpenHandle::Writer(file)
                | OpenHandle::ReadWrite(file)
                | OpenHandle::Append(file) => file
                    .seek(SeekFrom::Start(offset.max(0) as u64))
                    .map(|_| "1".to_string())
                    .unwrap_or_default(),
            }
        }
        _ => String::new(),
    }
}

pub fn file_append(text: &str, filename: &str) -> String {
    if filename == "*" {
        return std::io::stdout()
            .write_all(text.as_bytes())
            .map(|_| "0".to_string())
            .unwrap_or_else(|_| "1".to_string());
    }
    if filename == "**" {
        return std::io::stderr()
            .write_all(text.as_bytes())
            .map(|_| "0".to_string())
            .unwrap_or_else(|_| "1".to_string());
    }

    if filename.trim().is_empty() {
        return "1".to_string();
    }

    OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .map(|_| "0".to_string())
        .unwrap_or_else(|_| "1".to_string())
}

pub fn file_copy(source_pattern: &str, dest_pattern: &str, overwrite: bool) -> String {
    let sources = expand_paths(source_pattern);
    if sources.is_empty() {
        return "0".to_string();
    }

    let dest_path = PathBuf::from(dest_pattern);
    let dest_is_dir = dest_path.is_dir() || sources.len() > 1;
    let mut failures = 0usize;

    for source in sources {
        let target = if dest_is_dir {
            match source.file_name() {
                Some(name) => dest_path.join(name),
                None => {
                    failures += 1;
                    continue;
                }
            }
        } else {
            dest_path.clone()
        };

        if target.exists() && !overwrite {
            failures += 1;
            continue;
        }

        if let Some(parent) = target.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if fs::copy(&source, &target).is_err() {
            failures += 1;
        }
    }

    failures.to_string()
}

fn copy_dir_recursive(source: &Path, dest: &Path, overwrite: bool) -> bool {
    if !source.is_dir() {
        return false;
    }
    if fs::create_dir_all(dest).is_err() {
        return false;
    }

    let Ok(entries) = fs::read_dir(source) else {
        return false;
    };

    for entry in entries.flatten() {
        let src_path = entry.path();
        let dst_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            if !copy_dir_recursive(&src_path, &dst_path, overwrite) {
                return false;
            }
        } else {
            if dst_path.exists() && !overwrite {
                continue;
            }
            if let Some(parent) = dst_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if fs::copy(&src_path, &dst_path).is_err() {
                return false;
            }
        }
    }
    true
}

pub fn file_copy_dir(source: &str, dest: &str, overwrite: bool) -> String {
    if source.trim().is_empty() || dest.trim().is_empty() {
        return "1".to_string();
    }
    if copy_dir_recursive(Path::new(source), Path::new(dest), overwrite) {
        "0".to_string()
    } else {
        "1".to_string()
    }
}

pub fn file_create_dir(dir_name: &str) -> String {
    fs::create_dir_all(dir_name)
        .map(|_| "0".to_string())
        .unwrap_or_else(|_| "1".to_string())
}

pub fn file_create_shortcut(_target: &str, _link_file: &str) -> String {
    file_create_shortcut_full(
        _target, _link_file, None, None, None, None, None, None, None,
    )
}

pub fn file_create_shortcut_full(
    target: &str,
    link_file: &str,
    _working_dir: Option<&str>,
    args: Option<&str>,
    description: Option<&str>,
    _icon_file: Option<&str>,
    _shortcut_key: Option<&str>,
    _icon_number: Option<&str>,
    _run_state: Option<&str>,
) -> String {
    if target.trim().is_empty() || link_file.trim().is_empty() {
        return "1".to_string();
    }

    #[cfg(target_os = "linux")]
    {
        if link_file.ends_with(".lnk") {
            let payload = json!({
                "target": target,
                "args": args.unwrap_or(""),
                "description": description.unwrap_or("")
            })
            .to_string();
            if let Some(parent) = Path::new(link_file).parent() {
                let _ = fs::create_dir_all(parent);
            }
            return fs::write(link_file, payload)
                .map(|_| "0".to_string())
                .unwrap_or_else(|_| "1".to_string());
        }

        if let Some(parent) = Path::new(link_file).parent() {
            let _ = fs::create_dir_all(parent);
        }
        return std::os::unix::fs::symlink(target, link_file)
            .map(|_| "0".to_string())
            .unwrap_or_else(|_| "1".to_string());
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = (target, link_file, args, description);
        "1".to_string()
    }
}

pub fn file_delete(file_pattern: &str) -> String {
    let files = expand_paths(file_pattern);
    if files.is_empty() {
        return "0".to_string();
    }
    let mut failures = 0usize;
    for file in files {
        if fs::remove_file(&file).is_err() {
            failures += 1;
        }
    }
    failures.to_string()
}

pub fn file_exist(file_pattern: &str) -> String {
    if file_pattern.trim().is_empty() {
        return String::new();
    }
    if has_glob_pattern(file_pattern) {
        let matches = expand_paths(file_pattern);
        if let Some(first) = matches.first() {
            return attrs_from_std(first);
        }
        return String::new();
    }
    attrs_from_cross(file_pattern)
}

pub fn file_get_attrib(filename: &str) -> String {
    if filename.trim().is_empty() {
        return String::new();
    }
    attrs_from_cross(filename)
}

pub fn file_get_shortcut(link_file: &str) -> String {
    #[cfg(target_os = "linux")]
    {
        if link_file.ends_with(".lnk") {
            let Ok(text) = fs::read_to_string(link_file) else {
                return String::new();
            };
            let Ok(v) = serde_json::from_str::<Value>(&text) else {
                return String::new();
            };
            let target = v
                .get("target")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let dir = Path::new(&target)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let args = v
                .get("args")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let desc = v
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            return format!("{}|{}|{}|{}|||", target, dir, args, desc);
        }

        let Ok(target) = fs::read_link(link_file) else {
            return String::new();
        };
        let target_text = target.to_string_lossy().to_string();
        let dir = target
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        return format!("{}|{}|||||", target_text, dir);
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = link_file;
        String::new()
    }
}

pub fn file_get_size(filename: &str, units: Option<&str>) -> String {
    let Ok(metadata) = fs::metadata(filename) else {
        return String::new();
    };
    let size = metadata.len();
    match units.unwrap_or("B").trim().to_ascii_uppercase().as_str() {
        "K" => (size / 1024).to_string(),
        "M" => (size / 1_048_576).to_string(),
        _ => size.to_string(),
    }
}

pub fn file_get_time(filename: &str, which_time: Option<&str>) -> String {
    file_time_by_kind(filename, which_time)
}

pub fn file_get_version(_filename: &str) -> String {
    String::new()
}

pub fn file_install(source: &str, dest: &str, overwrite: bool) -> String {
    if source.trim().is_empty() || dest.trim().is_empty() {
        return "1".to_string();
    }
    let dest_path = Path::new(dest);
    if dest_path.exists() && !overwrite {
        return "1".to_string();
    }
    if let Some(parent) = dest_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::copy(source, dest)
        .map(|_| "0".to_string())
        .unwrap_or_else(|_| "1".to_string())
}

pub fn file_move(source_pattern: &str, dest_pattern: &str, overwrite: bool) -> String {
    let sources = expand_paths(source_pattern);
    if sources.is_empty() {
        return "0".to_string();
    }

    let dest = PathBuf::from(dest_pattern);
    let dest_is_dir = dest.is_dir() || sources.len() > 1;
    let mut failures = 0usize;

    for source in sources {
        let target = if dest_is_dir {
            match source.file_name() {
                Some(name) => dest.join(name),
                None => {
                    failures += 1;
                    continue;
                }
            }
        } else {
            dest.clone()
        };

        if target.exists() && !overwrite {
            failures += 1;
            continue;
        }
        if let Some(parent) = target.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if fs::rename(&source, &target).is_err() {
            if fs::copy(&source, &target).is_ok() && fs::remove_file(&source).is_ok() {
                continue;
            }
            failures += 1;
        }
    }

    failures.to_string()
}

pub fn file_move_dir(source: &str, dest: &str, overwrite_or_rename: Option<&str>) -> String {
    if source.trim().is_empty() || dest.trim().is_empty() {
        return "1".to_string();
    }
    let mode = overwrite_or_rename
        .unwrap_or("0")
        .trim()
        .to_ascii_uppercase();

    if Path::new(dest).exists() && mode == "0" {
        return "1".to_string();
    }

    if fs::rename(source, dest).is_ok() {
        return "0".to_string();
    }

    if mode == "R" {
        return "1".to_string();
    }

    if copy_dir_recursive(Path::new(source), Path::new(dest), mode != "0")
        && fs::remove_dir_all(source).is_ok()
    {
        "0".to_string()
    } else {
        "1".to_string()
    }
}

pub fn file_open(filename: &str, flags: Option<&str>) -> String {
    let mut options = OpenOptions::new();
    let f = flags.unwrap_or("r").trim().to_ascii_lowercase();

    if f.starts_with("rw") {
        options.read(true).write(true).create(true);
    } else if f.starts_with('w') {
        options.write(true).create(true).truncate(true);
    } else if f.starts_with('a') {
        options.append(true).create(true);
    } else {
        options.read(true);
    }

    let Ok(file) = options.open(filename) else {
        return String::new();
    };

    let open_handle = if f.starts_with("rw") {
        OpenHandle::ReadWrite(file)
    } else if f.starts_with('w') {
        OpenHandle::Writer(file)
    } else if f.starts_with('a') {
        OpenHandle::Append(file)
    } else {
        OpenHandle::Reader(file)
    };

    let id = next_handle_id();
    let mut store = handle_store().lock().expect("file handle mutex poisoned");
    store.handles.insert(id, open_handle);
    id.to_string()
}

pub fn file_read(filename: &str, options_and_filename: Option<&str>) -> String {
    let mut max_bytes: Option<usize> = None;
    let mut normalize_eol = false;

    if let Some(opts) = options_and_filename {
        for part in opts.split_whitespace() {
            let p = part.trim();
            if p.eq_ignore_ascii_case("*t") {
                normalize_eol = true;
            } else if let Some(size) = p.strip_prefix("*m") {
                max_bytes = size.parse::<usize>().ok();
            }
        }
    }

    let mut bytes = Vec::new();
    let Ok(mut file) = StdFile::open(filename) else {
        return String::new();
    };
    if let Some(limit) = max_bytes {
        let mut take = file.take(limit as u64);
        if take.read_to_end(&mut bytes).is_err() {
            return String::new();
        }
    } else if file.read_to_end(&mut bytes).is_err() {
        return String::new();
    }

    let mut text = String::from_utf8(bytes).unwrap_or_default();
    if normalize_eol {
        text = text.replace("\r\n", "\n");
    }
    text
}

pub fn file_read_line(filename: &str, line_num: i32) -> String {
    if line_num <= 0 {
        return String::new();
    }
    let Ok(content) = fs::read_to_string(filename) else {
        return String::new();
    };
    content
        .lines()
        .nth((line_num - 1) as usize)
        .map(str::to_string)
        .unwrap_or_default()
}

pub fn file_recycle(file_pattern: &str) -> String {
    let files = expand_paths(file_pattern);
    if files.is_empty() {
        return "0".to_string();
    }
    let mut has_failure = false;
    for file in files {
        if move_to_trash(&file).is_err() {
            has_failure = true;
        }
    }
    if has_failure {
        "1".to_string()
    } else {
        "0".to_string()
    }
}

pub fn file_recycle_empty(_drive_letter: Option<&str>) -> String {
    if empty_trash().is_ok() {
        "0".to_string()
    } else {
        "1".to_string()
    }
}

pub fn file_remove_dir(dir_name: &str, recurse: bool) -> String {
    let result = if recurse {
        fs::remove_dir_all(dir_name)
    } else {
        fs::remove_dir(dir_name)
    };
    result
        .map(|_| "0".to_string())
        .unwrap_or_else(|_| "1".to_string())
}

pub fn file_select_file(
    options: Option<&str>,
    root: Option<&str>,
    title: Option<&str>,
    _filter: Option<&str>,
) -> String {
    let opts = options.unwrap_or("").to_ascii_uppercase();
    let mut dialog = FileDialog::new();
    if let Some(path) = root.filter(|v| !v.trim().is_empty()) {
        dialog = dialog.set_directory(path);
    }
    if let Some(text) = title.filter(|v| !v.trim().is_empty()) {
        dialog = dialog.set_title(text);
    }
    if opts.contains('M') {
        return dialog
            .pick_files()
            .map(|files| {
                files
                    .into_iter()
                    .map(|path| path.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();
    }
    if opts.contains('S') {
        return dialog
            .save_file()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default();
    }
    dialog
        .pick_file()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn file_select_folder(
    starting_folder: Option<&str>,
    _options: Option<&str>,
    prompt: Option<&str>,
) -> String {
    let mut dialog = FileDialog::new();
    if let Some(path) = starting_folder.filter(|v| !v.trim().is_empty()) {
        dialog = dialog.set_directory(path);
    }
    if let Some(text) = prompt.filter(|v| !v.trim().is_empty()) {
        dialog = dialog.set_title(text);
    }
    dialog
        .pick_folder()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn file_set_attrib(
    attributes: &str,
    file_pattern: Option<&str>,
    operate_on_folders: Option<i32>,
    recurse: bool,
) -> String {
    let pattern = file_pattern.unwrap_or("");
    if pattern.is_empty() {
        return "1".to_string();
    }
    let mut paths = expand_paths(pattern);
    if recurse {
        let mut more = Vec::new();
        for path in &paths {
            if path.is_dir() {
                walk_files(path, true, &mut more);
            }
        }
        paths.extend(more);
    }

    let folders_mode = operate_on_folders.unwrap_or(0);
    let set_readonly = attributes.contains("+R");
    let clear_readonly = attributes.contains("-R");
    let toggle_readonly = attributes.contains("^R");

    let mut failures = 0usize;
    for path in paths {
        if folders_mode == 0 && path.is_dir() {
            continue;
        }
        if folders_mode == 2 && path.is_file() {
            continue;
        }
        let Ok(meta) = fs::metadata(&path) else {
            failures += 1;
            continue;
        };
        let mut perms = meta.permissions();
        let readonly = if toggle_readonly {
            !perms.readonly()
        } else if set_readonly {
            true
        } else if clear_readonly {
            false
        } else {
            perms.readonly()
        };
        perms.set_readonly(readonly);
        if fs::set_permissions(&path, perms).is_err() {
            failures += 1;
        }
    }

    failures.to_string()
}

pub fn file_set_time(
    timestamp: Option<&str>,
    file_pattern: Option<&str>,
    which_time: Option<&str>,
    operate_on_folders: Option<i32>,
    recurse: bool,
) -> String {
    let Some(parsed_time) = parse_ahk_timestamp(timestamp) else {
        return "1".to_string();
    };
    let ft = FileTime::from_system_time(parsed_time);

    let pattern = file_pattern.unwrap_or("");
    if pattern.is_empty() {
        return "1".to_string();
    }

    let mut paths = expand_paths(pattern);
    if recurse {
        let mut more = Vec::new();
        for path in &paths {
            if path.is_dir() {
                walk_files(path, true, &mut more);
            }
        }
        paths.extend(more);
    }

    let mode = operate_on_folders.unwrap_or(0);
    let which = which_time.unwrap_or("M").to_ascii_uppercase();
    let mut failures = 0usize;
    for path in paths {
        if mode == 0 && path.is_dir() {
            continue;
        }
        if mode == 2 && path.is_file() {
            continue;
        }

        if !path.exists() {
            failures += 1;
            continue;
        }

        let Ok(meta) = fs::metadata(&path) else {
            failures += 1;
            continue;
        };
        let current_mtime = FileTime::from_last_modification_time(&meta);
        let current_atime = FileTime::from_last_access_time(&meta);

        let result = match which.as_str() {
            "A" => set_file_times(&path, ft, current_mtime),
            "C" => set_creation_time(&path, ft),
            _ => set_file_times(&path, current_atime, ft),
        };
        if result.is_err() {
            failures += 1;
        }
    }
    failures.to_string()
}

pub fn loop_file(pattern: &str, mode: Option<&str>) -> String {
    let mode_text = mode.unwrap_or("").to_ascii_uppercase();
    let include_dirs = mode_text.contains('D');
    let include_files = mode_text.contains('F') || !include_dirs;
    let recurse = mode_text.contains('R');

    let mut paths = Vec::new();
    if has_glob_pattern(pattern) {
        paths.extend(expand_paths(pattern));
    } else {
        let path = PathBuf::from(pattern);
        if path.is_dir() {
            if recurse {
                walk_files(&path, true, &mut paths);
            } else {
                walk_files(&path, false, &mut paths);
            }
        } else if path.exists() {
            paths.push(path);
        }
    }

    let mut result = Vec::<Value>::new();
    for path in paths {
        if path.is_dir() && !include_dirs {
            continue;
        }
        if path.is_file() && !include_files {
            continue;
        }

        let name = path
            .file_name()
            .map(|v| v.to_string_lossy().to_string())
            .unwrap_or_default();
        let ext = path
            .extension()
            .map(|v| v.to_string_lossy().to_string())
            .unwrap_or_default();
        let full_path = path.to_string_lossy().to_string();
        let dir = path
            .parent()
            .map(|v| v.to_string_lossy().to_string())
            .unwrap_or_default();
        let attrib = attrs_from_std(&path);
        let metadata = fs::metadata(&path).ok();
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .map(system_time_to_ahk)
            .unwrap_or_default();
        let created = metadata
            .as_ref()
            .and_then(|m| m.created().ok())
            .map(system_time_to_ahk)
            .unwrap_or_default();
        let accessed = metadata
            .as_ref()
            .and_then(|m| m.accessed().ok())
            .map(system_time_to_ahk)
            .unwrap_or_default();

        result.push(json!({
            "name": name,
            "ext": ext,
            "fullPath": full_path,
            "longPath": full_path,
            "shortPath": "",
            "shortName": name,
            "dir": dir,
            "timeModified": modified,
            "timeCreated": created,
            "timeAccessed": accessed,
            "attrib": attrib,
            "size": size,
            "sizeKB": size / 1024,
            "sizeMB": size / 1_048_576
        }));
    }

    serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_string())
}

pub fn loop_read_file(input_file: &str) -> String {
    let Ok(content) = fs::read_to_string(input_file) else {
        return "[]".to_string();
    };
    let lines = content.lines().map(str::to_string).collect::<Vec<_>>();
    serde_json::to_string(&lines).unwrap_or_else(|_| "[]".to_string())
}

pub fn split_path(input: &str) -> String {
    let input_trimmed = input.trim();
    if input_trimmed.is_empty() {
        return "||||".to_string();
    }

    let path_obj = Path::new(input_trimmed);
    let file_name = path_obj
        .file_name()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_default();
    let dir = path_obj
        .parent()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = path_obj
        .extension()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_default();
    let name_no_ext = path_obj
        .file_stem()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_default();

    let drive = if let Some((scheme, rest)) = input_trimmed.split_once("://") {
        let host = rest.split('/').next().unwrap_or("");
        if host.is_empty() {
            String::new()
        } else {
            format!("{scheme}://{host}")
        }
    } else {
        let _ = CrossPath::new(input_trimmed);
        let bytes = input_trimmed.as_bytes();
        if bytes.len() >= 2 && bytes[1] == b':' && (bytes[0] as char).is_ascii_alphabetic() {
            input_trimmed[0..2].to_string()
        } else if input_trimmed.starts_with("//") || input_trimmed.starts_with("\\\\") {
            let without = input_trimmed
                .trim_start_matches('/')
                .trim_start_matches('\\');
            let server = without.split(['/', '\\']).next().unwrap_or("").to_string();
            if server.is_empty() {
                String::new()
            } else {
                format!("\\\\{server}")
            }
        } else {
            String::new()
        }
    };

    format!("{}|{}|{}|{}|{}", file_name, dir, ext, name_no_ext, drive)
}

pub fn drive_compat(args: &[&str]) -> String {
    drive(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied(),
        args.get(2).copied(),
    )
}

pub fn drive_get_compat(args: &[&str]) -> String {
    drive_get(
        args.get(1).copied().unwrap_or_default(),
        args.get(2).copied(),
    )
}

pub fn drive_space_free_compat(args: &[&str]) -> String {
    drive_space_free(args.get(1).copied().or_else(|| args.first().copied()))
}

pub fn file_compat(args: &[&str]) -> String {
    file(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied().unwrap_or_default(),
        args.get(2).copied(),
        args.get(3).copied(),
    )
}

pub fn file_append_compat(args: &[&str]) -> String {
    file_append(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied().unwrap_or_default(),
    )
}

pub fn file_copy_compat(args: &[&str]) -> String {
    file_copy(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied().unwrap_or_default(),
        parse_bool_flag(args.get(2).copied()),
    )
}

pub fn file_copy_dir_compat(args: &[&str]) -> String {
    file_copy_dir(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied().unwrap_or_default(),
        parse_bool_flag(args.get(2).copied()),
    )
}

pub fn file_create_dir_compat(args: &[&str]) -> String {
    file_create_dir(args.first().copied().unwrap_or_default())
}

pub fn file_create_shortcut_compat(args: &[&str]) -> String {
    file_create_shortcut_full(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied().unwrap_or_default(),
        args.get(2).copied(),
        args.get(3).copied(),
        args.get(4).copied(),
        args.get(5).copied(),
        args.get(6).copied(),
        args.get(7).copied(),
        args.get(8).copied(),
    )
}

pub fn file_delete_compat(args: &[&str]) -> String {
    file_delete(args.first().copied().unwrap_or_default())
}

pub fn file_exist_compat(args: &[&str]) -> String {
    file_exist(args.first().copied().unwrap_or_default())
}

pub fn file_get_attrib_compat(args: &[&str]) -> String {
    file_get_attrib(args.first().copied().unwrap_or_default())
}

pub fn file_get_shortcut_compat(args: &[&str]) -> String {
    file_get_shortcut(args.first().copied().unwrap_or_default())
}

pub fn file_get_size_compat(args: &[&str]) -> String {
    file_get_size(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied(),
    )
}

pub fn file_get_time_compat(args: &[&str]) -> String {
    file_get_time(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied(),
    )
}

pub fn file_get_version_compat(args: &[&str]) -> String {
    file_get_version(args.first().copied().unwrap_or_default())
}

pub fn file_install_compat(args: &[&str]) -> String {
    file_install(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied().unwrap_or_default(),
        parse_bool_flag(args.get(2).copied()),
    )
}

pub fn file_move_compat(args: &[&str]) -> String {
    file_move(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied().unwrap_or_default(),
        parse_bool_flag(args.get(2).copied()),
    )
}

pub fn file_move_dir_compat(args: &[&str]) -> String {
    file_move_dir(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied().unwrap_or_default(),
        args.get(2).copied(),
    )
}

pub fn file_open_compat(args: &[&str]) -> String {
    file_open(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied(),
    )
}

pub fn file_read_compat(args: &[&str]) -> String {
    file_read(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied(),
    )
}

pub fn file_read_line_compat(args: &[&str]) -> String {
    file_read_line(
        args.first().copied().unwrap_or_default(),
        parse_i32(args.get(1).copied()).unwrap_or(1),
    )
}

pub fn file_recycle_compat(args: &[&str]) -> String {
    file_recycle(args.first().copied().unwrap_or_default())
}

pub fn file_recycle_empty_compat(args: &[&str]) -> String {
    file_recycle_empty(args.first().copied())
}

pub fn file_remove_dir_compat(args: &[&str]) -> String {
    file_remove_dir(
        args.first().copied().unwrap_or_default(),
        parse_bool_flag(args.get(1).copied()),
    )
}

pub fn file_select_file_compat(args: &[&str]) -> String {
    file_select_file(
        args.first().copied(),
        args.get(1).copied(),
        args.get(2).copied(),
        args.get(3).copied(),
    )
}

pub fn file_select_folder_compat(args: &[&str]) -> String {
    file_select_folder(
        args.first().copied(),
        args.get(1).copied(),
        args.get(2).copied(),
    )
}

pub fn file_set_attrib_compat(args: &[&str]) -> String {
    file_set_attrib(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied(),
        parse_i32(args.get(2).copied()),
        parse_bool_flag(args.get(3).copied()),
    )
}

pub fn file_set_time_compat(args: &[&str]) -> String {
    file_set_time(
        args.first().copied(),
        args.get(1).copied(),
        args.get(2).copied(),
        parse_i32(args.get(3).copied()),
        parse_bool_flag(args.get(4).copied()),
    )
}

pub fn loop_file_compat(args: &[&str]) -> String {
    loop_file(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied(),
    )
}

pub fn loop_read_file_compat(args: &[&str]) -> String {
    loop_read_file(args.first().copied().unwrap_or_default())
}

pub fn split_path_compat(args: &[&str]) -> String {
    split_path(args.first().copied().unwrap_or_default())
}

pub fn compat_drive(args: &[&str]) -> String {
    drive_compat(args)
}
pub fn compat_drive_get(args: &[&str]) -> String {
    drive_get_compat(args)
}
pub fn compat_drive_space_free(args: &[&str]) -> String {
    drive_space_free_compat(args)
}
pub fn compat_file(args: &[&str]) -> String {
    file_compat(args)
}
pub fn compat_file_append(args: &[&str]) -> String {
    file_append_compat(args)
}
pub fn compat_file_copy(args: &[&str]) -> String {
    file_copy_compat(args)
}
pub fn compat_file_copy_dir(args: &[&str]) -> String {
    file_copy_dir_compat(args)
}
pub fn compat_file_create_dir(args: &[&str]) -> String {
    file_create_dir_compat(args)
}
pub fn compat_file_create_shortcut(args: &[&str]) -> String {
    file_create_shortcut_compat(args)
}
pub fn compat_file_delete(args: &[&str]) -> String {
    file_delete_compat(args)
}
pub fn compat_file_exist(args: &[&str]) -> String {
    file_exist_compat(args)
}
pub fn compat_file_get_attrib(args: &[&str]) -> String {
    file_get_attrib_compat(args)
}
pub fn compat_file_get_shortcut(args: &[&str]) -> String {
    file_get_shortcut_compat(args)
}
pub fn compat_file_get_size(args: &[&str]) -> String {
    file_get_size_compat(args)
}
pub fn compat_file_get_time(args: &[&str]) -> String {
    file_get_time_compat(args)
}
pub fn compat_file_get_version(args: &[&str]) -> String {
    file_get_version_compat(args)
}
pub fn compat_file_install(args: &[&str]) -> String {
    file_install_compat(args)
}
pub fn compat_file_move(args: &[&str]) -> String {
    file_move_compat(args)
}
pub fn compat_file_move_dir(args: &[&str]) -> String {
    file_move_dir_compat(args)
}
pub fn compat_file_open(args: &[&str]) -> String {
    file_open_compat(args)
}
pub fn compat_file_read(args: &[&str]) -> String {
    file_read_compat(args)
}
pub fn compat_file_read_line(args: &[&str]) -> String {
    file_read_line_compat(args)
}
pub fn compat_file_recycle(args: &[&str]) -> String {
    file_recycle_compat(args)
}
pub fn compat_file_recycle_empty(args: &[&str]) -> String {
    file_recycle_empty_compat(args)
}
pub fn compat_file_remove_dir(args: &[&str]) -> String {
    file_remove_dir_compat(args)
}
pub fn compat_file_select_file(args: &[&str]) -> String {
    file_select_file_compat(args)
}
pub fn compat_file_select_folder(args: &[&str]) -> String {
    file_select_folder_compat(args)
}
pub fn compat_file_set_attrib(args: &[&str]) -> String {
    file_set_attrib_compat(args)
}
pub fn compat_file_set_time(args: &[&str]) -> String {
    file_set_time_compat(args)
}
pub fn compat_loop_file(args: &[&str]) -> String {
    loop_file_compat(args)
}
pub fn compat_loop_read_file(args: &[&str]) -> String {
    loop_read_file_compat(args)
}
pub fn compat_split_path(args: &[&str]) -> String {
    split_path_compat(args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("Drive", compat_drive),
    ("DriveGet", compat_drive_get),
    ("DriveSpaceFree", compat_drive_space_free),
    ("File", compat_file),
    ("FileAppend", compat_file_append),
    ("FileCopy", compat_file_copy),
    ("FileCopyDir", compat_file_copy_dir),
    ("FileCreateDir", compat_file_create_dir),
    ("FileCreateShortcut", compat_file_create_shortcut),
    ("FileDelete", compat_file_delete),
    ("FileExist", compat_file_exist),
    ("FileGetAttrib", compat_file_get_attrib),
    ("FileGetShortcut", compat_file_get_shortcut),
    ("FileGetSize", compat_file_get_size),
    ("FileGetTime", compat_file_get_time),
    ("FileGetVersion", compat_file_get_version),
    ("FileInstall", compat_file_install),
    ("FileMove", compat_file_move),
    ("FileMoveDir", compat_file_move_dir),
    ("FileOpen", compat_file_open),
    ("FileRead", compat_file_read),
    ("FileReadLine", compat_file_read_line),
    ("FileRecycle", compat_file_recycle),
    ("FileRecycleEmpty", compat_file_recycle_empty),
    ("FileRemoveDir", compat_file_remove_dir),
    ("FileSelectFile", compat_file_select_file),
    ("FileSelectFolder", compat_file_select_folder),
    ("FileSetAttrib", compat_file_set_attrib),
    ("FileSetTime", compat_file_set_time),
    ("LoopFile", compat_loop_file),
    ("LoopReadFile", compat_loop_read_file),
    ("SplitPath", compat_split_path),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_path_returns_components() {
        let parts = split_path("/tmp/example.txt");
        assert!(parts.contains("example.txt"));
        assert!(parts.contains("tmp"));
        assert!(parts.contains("txt"));
    }

    #[test]
    fn file_append_and_read_round_trip() {
        let unique = format!(
            "ihk_disk_test_{}.txt",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        let path_str = path.to_string_lossy().to_string();

        assert_eq!(file_append("hello", &path_str), "0");
        assert_eq!(file_read(&path_str, None), "hello");
        assert_eq!(file_read_line(&path_str, 1), "hello");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn file_copy_and_delete_work() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let source = std::env::temp_dir().join(format!("ihk_disk_src_{id}.txt"));
        let dest = std::env::temp_dir().join(format!("ihk_disk_dst_{id}.txt"));
        let source_str = source.to_string_lossy().to_string();
        let dest_str = dest.to_string_lossy().to_string();

        assert_eq!(file_append("abc", &source_str), "0");
        assert_eq!(file_copy(&source_str, &dest_str, true), "0");
        assert_eq!(file_read(&dest_str, None), "abc");
        assert_eq!(file_delete(&source_str), "0");
        assert_eq!(file_delete(&dest_str), "0");
    }

    #[test]
    fn loop_read_file_returns_json_array() {
        let unique = format!(
            "ihk_disk_loop_read_{}.txt",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        let path_str = path.to_string_lossy().to_string();

        assert_eq!(file_append("line1\nline2", &path_str), "0");
        let text = loop_read_file(&path_str);
        let parsed = serde_json::from_str::<Vec<String>>(&text).unwrap_or_default();
        assert_eq!(parsed, vec!["line1".to_string(), "line2".to_string()]);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn shortcut_round_trip_on_linux() {
        let unique = format!(
            "ihk_disk_shortcut_{}.lnk",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let target = "/tmp/target.txt";
        let path = std::env::temp_dir().join(unique);
        let path_str = path.to_string_lossy().to_string();

        let created = file_create_shortcut_full(
            target,
            &path_str,
            None,
            Some("--x"),
            Some("desc"),
            None,
            None,
            None,
            None,
        );
        assert_eq!(created, "0");
        let data = file_get_shortcut(&path_str);
        assert!(data.contains(target));
        assert!(data.contains("--x"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn recycle_moves_file_to_trash() {
        let unique = format!(
            "ihk_disk_recycle_{}.txt",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        let path_str = path.to_string_lossy().to_string();

        assert_eq!(file_append("trash-me", &path_str), "0");
        assert!(path.exists());
        assert_eq!(file_recycle(&path_str), "0");
        assert!(!path.exists());
    }
}
