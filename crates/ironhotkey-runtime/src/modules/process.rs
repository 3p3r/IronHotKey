use std::cell::RefCell;
use std::process::Command;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::io;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(unix)]
use std::process::Stdio;

use sysinfo::{Pid, ProcessesToUpdate, System};

use super::ModuleMethod;

#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Clone)]
struct RunAsCredentials {
    user: String,
    password: String,
    domain: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct LaunchOptions {
    hide: bool,
    min: bool,
    max: bool,
}

thread_local! {
    static RUN_AS: RefCell<Option<RunAsCredentials>> = const { RefCell::new(None) };
}
pub fn process_cmd(sub_command: &str, pid_or_name: Option<&str>, value: Option<&str>) -> String {
    match sub_command.trim().to_uppercase().as_str() {
        "EXIST" => process_exist(pid_or_name),
        "CLOSE" => process_close(pid_or_name),
        "PRIORITY" => process_priority(pid_or_name, value),
        "WAIT" => process_wait(pid_or_name, value),
        "WAITCLOSE" => process_wait_close(pid_or_name, value),
        _ => String::new(),
    }
}

fn process_exist(pid_or_name: Option<&str>) -> String {
    let target = match pid_or_name {
        None | Some("") => return std::process::id().to_string(),
        Some(v) => v.trim(),
    };
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    find_pid(&sys, target)
        .map(|p| p.to_string())
        .unwrap_or_else(|| "0".to_string())
}

fn process_close(pid_or_name: Option<&str>) -> String {
    let target = match pid_or_name {
        None | Some("") => return "0".to_string(),
        Some(v) => v.trim(),
    };
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let Some(pid) = find_pid(&sys, target) else {
        return "0".to_string();
    };
    let pid_str = pid.to_string();
    if let Some(proc) = sys.process(Pid::from(pid as usize)) {
        proc.kill();
    }
    pid_str
}

fn process_priority(pid_or_name: Option<&str>, value: Option<&str>) -> String {
    let target = match pid_or_name {
        None | Some("") => return String::new(),
        Some(v) => v.trim(),
    };
    let level = value.unwrap_or("N").trim().to_uppercase();
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let Some(pid) = find_pid(&sys, target) else {
        return String::new();
    };

    #[cfg(unix)]
    {
        let nice: i32 = match level.as_str() {
            "L" | "LOW" => 19,
            "B" | "BELOWNORMAL" => 10,
            "N" | "NORMAL" => 0,
            "A" | "ABOVENORMAL" => -5,
            "H" | "HIGH" => -10,
            "R" | "REALTIME" => -20,
            _ => 0,
        };
        let _ = unsafe { libc_setpriority(pid as u32, nice) };
    }

    let _ = (pid, level);
    String::new()
}

#[cfg(unix)]
unsafe fn libc_setpriority(pid: u32, nice: i32) -> i32 {
    extern "C" {
        fn setpriority(which: i32, who: u32, prio: i32) -> i32;
    }
    setpriority(0, pid, nice)
}

fn process_wait(pid_or_name: Option<&str>, value: Option<&str>) -> String {
    let target = match pid_or_name {
        None | Some("") => return "0".to_string(),
        Some(v) => v.trim(),
    };
    let timeout_secs: f64 = value
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(f64::MAX);
    let deadline = Instant::now() + Duration::from_secs_f64(timeout_secs.min(86400.0));

    loop {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        if let Some(pid) = find_pid(&sys, target) {
            return pid.to_string();
        }
        if Instant::now() >= deadline {
            return "0".to_string();
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn process_wait_close(pid_or_name: Option<&str>, value: Option<&str>) -> String {
    let target = match pid_or_name {
        None | Some("") => return String::new(),
        Some(v) => v.trim(),
    };
    let timeout_secs: f64 = value
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(f64::MAX);
    let deadline = Instant::now() + Duration::from_secs_f64(timeout_secs.min(86400.0));

    loop {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        if find_pid(&sys, target).is_none() {
            return String::new();
        }
        if Instant::now() >= deadline {
            if let Some(pid) = find_pid(&sys, target) {
                return pid.to_string();
            }
            return String::new();
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

pub fn run_cmd(target: &str, working_dir: Option<&str>, options: Option<&str>) -> String {
    let (program, args) = parse_target(target);
    let launch_options = parse_launch_options(options);

    #[cfg(windows)]
    if let Some(result) = windows_spawn(&program, &args, working_dir, launch_options, false) {
        return result;
    }

    let mut cmd = build_command(&program, &args, working_dir, launch_options);
    apply_unix_run_as_hint(&mut cmd);

    match cmd.spawn() {
        Ok(child) => {
            #[cfg(unix)]
            apply_unix_window_state(child.id(), launch_options);
            child.id().to_string()
        }
        Err(_) => String::new(),
    }
}

pub fn run_as(user: Option<&str>, password: Option<&str>, domain: Option<&str>) -> String {
    RUN_AS.with(|ra| {
        let u = user.unwrap_or("").trim();
        let p = password.unwrap_or("").trim();
        let d = domain.unwrap_or("").trim();
        if u.is_empty() && p.is_empty() && d.is_empty() {
            *ra.borrow_mut() = None;
        } else {
            *ra.borrow_mut() = Some(RunAsCredentials {
                user: u.to_string(),
                password: p.to_string(),
                domain: d.to_string(),
            });
        }
    });
    String::new()
}

pub fn run_wait(target: &str, working_dir: Option<&str>, options: Option<&str>) -> String {
    let (program, args) = parse_target(target);
    let launch_options = parse_launch_options(options);

    #[cfg(windows)]
    if let Some(result) = windows_spawn(&program, &args, working_dir, launch_options, true) {
        return result;
    }

    let mut cmd = build_command(&program, &args, working_dir, launch_options);
    apply_unix_run_as_hint(&mut cmd);
    match cmd.spawn() {
        Ok(mut child) => {
            #[cfg(unix)]
            apply_unix_window_state(child.id(), launch_options);
            match child.wait() {
                Ok(status) => status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "0".to_string()),
                Err(_) => String::new(),
            }
        }
        Err(_) => String::new(),
    }
}

fn find_pid(sys: &System, target: &str) -> Option<u32> {
    if let Ok(numeric) = target.parse::<u32>() {
        let pid = Pid::from(numeric as usize);
        if sys.process(pid).is_some() {
            return Some(numeric);
        }
        return None;
    }
    let target_lower = target.to_lowercase();
    sys.processes().iter().find_map(|(pid, proc)| {
        let name = proc.name().to_string_lossy().to_lowercase();
        if name == target_lower
            || name.trim_end_matches(".exe") == target_lower.trim_end_matches(".exe")
        {
            Some(pid.as_u32())
        } else {
            None
        }
    })
}

fn parse_target(target: &str) -> (String, Vec<String>) {
    let parts = shell_split(target);
    if parts.is_empty() {
        return (target.to_string(), vec![]);
    }
    let program = parts[0].clone();
    let args = parts[1..].to_vec();
    (program, args)
}

fn shell_split(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    for ch in s.chars() {
        match ch {
            '"' => in_quote = !in_quote,
            ' ' | '\t' if !in_quote => {
                if !cur.is_empty() {
                    tokens.push(cur.clone());
                    cur.clear();
                }
            }
            _ => cur.push(ch),
        }
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    tokens
}

fn build_command(
    program: &str,
    args: &[String],
    working_dir: Option<&str>,
    _launch_options: LaunchOptions,
) -> Command {
    let mut cmd = Command::new(program);
    cmd.args(args);
    if let Some(dir) = working_dir.filter(|d| !d.trim().is_empty()) {
        cmd.current_dir(dir);
    }

    #[cfg(unix)]
    if _launch_options.hide {
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        unsafe {
            cmd.pre_exec(|| {
                if libc_setsid() == -1 {
                    Err(io::Error::last_os_error())
                } else {
                    Ok(())
                }
            });
        }
    }

    cmd
}

fn parse_launch_options(options: Option<&str>) -> LaunchOptions {
    let mut parsed = LaunchOptions::default();

    for token in options.unwrap_or_default().split_whitespace() {
        match token.to_ascii_lowercase().as_str() {
            "hide" => {
                parsed.hide = true;
                parsed.min = false;
                parsed.max = false;
            }
            "min" => {
                if !parsed.hide {
                    parsed.min = true;
                    parsed.max = false;
                }
            }
            "max" => {
                if !parsed.hide {
                    parsed.max = true;
                    parsed.min = false;
                }
            }
            _ => {}
        }
    }

    parsed
}

fn current_run_as() -> Option<RunAsCredentials> {
    RUN_AS.with(|ra| ra.borrow().clone())
}

fn apply_unix_run_as_hint(_cmd: &mut Command) {
    #[cfg(unix)]
    if let Some(creds) = current_run_as() {
        if !creds.user.is_empty() {
            _cmd.env("SUDO_USER", creds.user);
        }
    }
}

#[cfg(unix)]
fn apply_unix_window_state(pid: u32, launch_options: LaunchOptions) {
    if launch_options.hide || (!launch_options.min && !launch_options.max) {
        return;
    }

    std::thread::spawn(move || {
        for _ in 0..20 {
            #[cfg(target_os = "linux")]
            {
                if linux_apply_window_state(pid, launch_options) {
                    return;
                }
            }

            #[cfg(target_os = "macos")]
            {
                if macos_apply_window_state(pid, launch_options) {
                    return;
                }
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    });
}

#[cfg(target_os = "linux")]
fn linux_apply_window_state(pid: u32, launch_options: LaunchOptions) -> bool {
    let window_id = match linux_window_id_for_pid(pid) {
        Some(id) => id,
        None => return false,
    };

    if launch_options.min {
        return Command::new("wmctrl")
            .args(["-i", "-r", &window_id, "-b", "add,hidden"])
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
    }

    if launch_options.max {
        return Command::new("wmctrl")
            .args([
                "-i",
                "-r",
                &window_id,
                "-b",
                "add,maximized_vert,maximized_horz",
            ])
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
    }

    false
}

#[cfg(target_os = "linux")]
fn linux_window_id_for_pid(pid: u32) -> Option<String> {
    let output = Command::new("wmctrl").args(["-lp"]).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    stdout.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let window_id = parts.next()?;
        let _desktop = parts.next()?;
        let parsed_pid = parts.next()?.parse::<u32>().ok()?;
        if parsed_pid == pid {
            Some(window_id.to_string())
        } else {
            None
        }
    })
}

#[cfg(target_os = "macos")]
fn macos_apply_window_state(pid: u32, launch_options: LaunchOptions) -> bool {
    let script = if launch_options.min {
        format!(
            "tell application \"System Events\" to tell (first application process whose unix id is {pid}) to set miniaturized of every window to true"
        )
    } else if launch_options.max {
        format!(
            "tell application \"System Events\" to tell (first application process whose unix id is {pid}) to set zoomed of every window to true"
        )
    } else {
        return false;
    };

    Command::new("osascript")
        .args(["-e", &script])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[cfg(unix)]
unsafe fn libc_setsid() -> i32 {
    unsafe extern "C" {
        fn setsid() -> i32;
    }

    unsafe { setsid() }
}

#[cfg(windows)]
fn windows_spawn(
    program: &str,
    args: &[String],
    working_dir: Option<&str>,
    launch_options: LaunchOptions,
    wait: bool,
) -> Option<String> {
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};

    type Bool = i32;
    type Dword = u32;
    type Handle = *mut c_void;
    type LpVoid = *mut c_void;
    type Lpcwstr = *const u16;
    type Lpwstr = *mut u16;

    const STARTF_USESHOWWINDOW: Dword = 0x0000_0001;
    const SW_HIDE: u16 = 0;
    const SW_SHOWMINIMIZED: u16 = 2;
    const SW_SHOWMAXIMIZED: u16 = 3;
    const LOGON_WITH_PROFILE: Dword = 0x0000_0001;
    const INFINITE: Dword = 0xFFFF_FFFF;

    #[repr(C)]
    struct StartupInfoW {
        cb: Dword,
        lp_reserved: Lpwstr,
        lp_desktop: Lpwstr,
        lp_title: Lpwstr,
        dw_x: Dword,
        dw_y: Dword,
        dw_x_size: Dword,
        dw_y_size: Dword,
        dw_x_count_chars: Dword,
        dw_y_count_chars: Dword,
        dw_fill_attribute: Dword,
        dw_flags: Dword,
        w_show_window: u16,
        cb_reserved2: u16,
        lp_reserved2: *mut u8,
        h_std_input: Handle,
        h_std_output: Handle,
        h_std_error: Handle,
    }

    #[repr(C)]
    struct ProcessInformation {
        h_process: Handle,
        h_thread: Handle,
        dw_process_id: Dword,
        dw_thread_id: Dword,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CloseHandle(h_object: Handle) -> Bool;
        fn CreateProcessW(
            lp_application_name: Lpcwstr,
            lp_command_line: Lpwstr,
            lp_process_attributes: LpVoid,
            lp_thread_attributes: LpVoid,
            b_inherit_handles: Bool,
            dw_creation_flags: Dword,
            lp_environment: LpVoid,
            lp_current_directory: Lpcwstr,
            lp_startup_info: *mut StartupInfoW,
            lp_process_information: *mut ProcessInformation,
        ) -> Bool;
        fn GetExitCodeProcess(h_process: Handle, lp_exit_code: *mut Dword) -> Bool;
        fn WaitForSingleObject(h_handle: Handle, dw_milliseconds: Dword) -> Dword;
    }

    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn CreateProcessWithLogonW(
            lp_username: Lpcwstr,
            lp_domain: Lpcwstr,
            lp_password: Lpcwstr,
            dw_logon_flags: Dword,
            lp_application_name: Lpcwstr,
            lp_command_line: Lpwstr,
            dw_creation_flags: Dword,
            lp_environment: LpVoid,
            lp_current_directory: Lpcwstr,
            lp_startup_info: *mut StartupInfoW,
            lp_process_information: *mut ProcessInformation,
        ) -> Bool;
    }

    fn to_wide_null(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn quote_windows_arg(value: &str) -> String {
        if !value.is_empty()
            && !value
                .chars()
                .any(|ch| ch.is_ascii_whitespace() || ch == '"')
        {
            return value.to_string();
        }

        let mut quoted = String::from('"');
        let mut backslashes = 0;
        for ch in value.chars() {
            if ch == '\\' {
                backslashes += 1;
                continue;
            }

            if ch == '"' {
                quoted.push_str(&"\\".repeat(backslashes * 2 + 1));
                quoted.push('"');
                backslashes = 0;
                continue;
            }

            quoted.push_str(&"\\".repeat(backslashes));
            backslashes = 0;
            quoted.push(ch);
        }
        quoted.push_str(&"\\".repeat(backslashes * 2));
        quoted.push('"');
        quoted
    }

    fn command_line(program: &str, args: &[String]) -> String {
        std::iter::once(program)
            .chain(args.iter().map(String::as_str))
            .map(quote_windows_arg)
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn startup_info_for(launch_options: LaunchOptions) -> StartupInfoW {
        let mut startup = StartupInfoW {
            cb: std::mem::size_of::<StartupInfoW>() as Dword,
            lp_reserved: null_mut(),
            lp_desktop: null_mut(),
            lp_title: null_mut(),
            dw_x: 0,
            dw_y: 0,
            dw_x_size: 0,
            dw_y_size: 0,
            dw_x_count_chars: 0,
            dw_y_count_chars: 0,
            dw_fill_attribute: 0,
            dw_flags: 0,
            w_show_window: 0,
            cb_reserved2: 0,
            lp_reserved2: null_mut(),
            h_std_input: null_mut(),
            h_std_output: null_mut(),
            h_std_error: null_mut(),
        };

        let show_window = if launch_options.hide {
            Some(SW_HIDE)
        } else if launch_options.min {
            Some(SW_SHOWMINIMIZED)
        } else if launch_options.max {
            Some(SW_SHOWMAXIMIZED)
        } else {
            None
        };

        if let Some(value) = show_window {
            startup.dw_flags |= STARTF_USESHOWWINDOW;
            startup.w_show_window = value;
        }

        startup
    }

    let run_as = current_run_as();
    let app_name = to_wide_null(program);
    let mut command_line = to_wide_null(&command_line(program, args));
    let current_dir = working_dir
        .filter(|value| !value.trim().is_empty())
        .map(to_wide_null);
    let mut startup = startup_info_for(launch_options);
    let mut process_info = ProcessInformation {
        h_process: null_mut(),
        h_thread: null_mut(),
        dw_process_id: 0,
        dw_thread_id: 0,
    };

    let created = unsafe {
        if let Some(creds) = run_as.filter(|creds| !creds.user.is_empty()) {
            let username = to_wide_null(&creds.user);
            let domain = (!creds.domain.is_empty()).then(|| to_wide_null(&creds.domain));
            let password = to_wide_null(&creds.password);
            CreateProcessWithLogonW(
                username.as_ptr(),
                domain.as_ref().map_or(null(), |value| value.as_ptr()),
                password.as_ptr(),
                LOGON_WITH_PROFILE,
                app_name.as_ptr(),
                command_line.as_mut_ptr(),
                0,
                null_mut(),
                current_dir.as_ref().map_or(null(), |value| value.as_ptr()),
                &mut startup,
                &mut process_info,
            )
        } else {
            CreateProcessW(
                app_name.as_ptr(),
                command_line.as_mut_ptr(),
                null_mut(),
                null_mut(),
                0,
                0,
                null_mut(),
                current_dir.as_ref().map_or(null(), |value| value.as_ptr()),
                &mut startup,
                &mut process_info,
            )
        }
    };

    if created == 0 {
        return None;
    }

    unsafe {
        let _ = CloseHandle(process_info.h_thread);
    }

    if wait {
        let mut exit_code = 0u32;
        unsafe {
            let _ = WaitForSingleObject(process_info.h_process, INFINITE);
            let _ = GetExitCodeProcess(process_info.h_process, &mut exit_code);
            let _ = CloseHandle(process_info.h_process);
        }
        Some((exit_code as i32).to_string())
    } else {
        let pid = process_info.dw_process_id.to_string();
        unsafe {
            let _ = CloseHandle(process_info.h_process);
        }
        Some(pid)
    }
}

fn process_cmd_compat(args: &[&str]) -> String {
    let sub = args.first().copied().unwrap_or_default();
    let pid_or_name = args.get(1).copied().filter(|v| !v.trim().is_empty());
    let value = args.get(2).copied().filter(|v| !v.trim().is_empty());
    process_cmd(sub, pid_or_name, value)
}

fn run_cmd_compat(args: &[&str]) -> String {
    let target = args.first().copied().unwrap_or_default();
    let working_dir = args.get(1).copied().filter(|v| !v.trim().is_empty());
    let options = args.get(2).copied().filter(|v| !v.trim().is_empty());
    run_cmd(target, working_dir, options)
}

fn run_as_compat(args: &[&str]) -> String {
    let user = args.first().copied().filter(|v| !v.trim().is_empty());
    let password = args.get(1).copied().filter(|v| !v.trim().is_empty());
    let domain = args.get(2).copied().filter(|v| !v.trim().is_empty());
    run_as(user, password, domain)
}

fn run_wait_compat(args: &[&str]) -> String {
    let target = args.first().copied().unwrap_or_default();
    let working_dir = args.get(1).copied().filter(|v| !v.trim().is_empty());
    let options = args.get(2).copied().filter(|v| !v.trim().is_empty());
    run_wait(target, working_dir, options)
}

pub fn compat_process_cmd(args: &[&str]) -> String {
    process_cmd_compat(args)
}
pub fn compat_run_cmd(args: &[&str]) -> String {
    run_cmd_compat(args)
}
pub fn compat_run_as(args: &[&str]) -> String {
    run_as_compat(args)
}
pub fn compat_run_wait(args: &[&str]) -> String {
    run_wait_compat(args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("Process", compat_process_cmd),
    ("Run", compat_run_cmd),
    ("RunAs", compat_run_as),
    ("RunWait", compat_run_wait),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_exist_self() {
        let my_pid = std::process::id().to_string();
        let result = process_exist(Some(&my_pid));
        assert_eq!(result, my_pid, "current process should exist");
    }

    #[test]
    fn test_process_exist_empty_returns_self() {
        let my_pid = std::process::id().to_string();
        let result = process_exist(None);
        assert_eq!(result, my_pid);
    }

    #[test]
    fn test_process_exist_nonexistent_returns_zero() {
        let result = process_exist(Some("999999999"));
        assert_eq!(result, "0");
    }

    #[test]
    fn test_run_echo() {
        #[cfg(unix)]
        let pid_str = run_cmd("echo hello", None, None);
        #[cfg(windows)]
        let pid_str = run_cmd("cmd /c echo hello", None, None);
        assert!(!pid_str.is_empty(), "run_cmd should return a PID");
        assert!(pid_str.parse::<u32>().is_ok(), "PID should be numeric");
    }

    #[test]
    fn test_run_wait_exit_code() {
        #[cfg(unix)]
        let code = run_wait("true", None, None);
        #[cfg(windows)]
        let code = run_wait("cmd /c exit 0", None, None);
        assert_eq!(code, "0");
    }

    #[test]
    fn test_run_wait_nonzero_exit() {
        #[cfg(unix)]
        let code = run_wait("sh -c \"exit 42\"", None, None);
        #[cfg(windows)]
        let code = run_wait("cmd /c exit 42", None, None);
        assert_eq!(code, "42");
    }

    #[test]
    fn test_run_as_clears_with_no_args() {
        run_as(Some("testuser"), None, None);
        run_as(None, None, None);
        RUN_AS.with(|ra| {
            assert!(ra.borrow().is_none(), "RunAs should be cleared");
        });
    }

    #[test]
    fn test_parse_launch_options_prefers_hide() {
        let parsed = parse_launch_options(Some("Min Hide Max"));
        assert_eq!(
            parsed,
            LaunchOptions {
                hide: true,
                min: false,
                max: false,
            }
        );
    }

    #[test]
    fn test_parse_launch_options_last_min_max_wins() {
        let parsed = parse_launch_options(Some("Min Max"));
        assert_eq!(
            parsed,
            LaunchOptions {
                hide: false,
                min: false,
                max: true,
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_run_wait_applies_run_as_hint() {
        run_as(Some("copilot"), None, None);
        let code = run_wait("sh -c \"test x$SUDO_USER = xcopilot\"", None, None);
        run_as(None, None, None);
        assert_eq!(code, "0");
    }

    #[test]
    fn test_process_cmd_dispatch() {
        let my_pid = std::process::id().to_string();
        let result = process_cmd("Exist", Some(&my_pid), None);
        assert_eq!(result, my_pid);
    }

    #[test]
    fn test_compat_run_cmd_parses_args() {
        #[cfg(unix)]
        let result = compat_run_cmd(&["echo hello"]);
        #[cfg(windows)]
        let result = compat_run_cmd(&["cmd /c echo hello"]);
        assert!(!result.is_empty());
        assert!(result.parse::<u32>().is_ok());
    }

    #[test]
    fn test_shell_split_quoted() {
        let parts = shell_split(r#"program "arg with spaces" other"#);
        assert_eq!(parts, vec!["program", "arg with spaces", "other"]);
    }
}
