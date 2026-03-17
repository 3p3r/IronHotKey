#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub fn platform_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
}

pub fn init_log() {
    #[cfg(target_os = "windows")]
    windows::init();
    #[cfg(target_os = "macos")]
    macos::init();
    #[cfg(target_os = "linux")]
    linux::init();
}
