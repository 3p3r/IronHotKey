use super::{stub_log, ModuleMethod};

pub fn allow_same_line_comments(args: &[&str]) -> String {
    stub_log("directives", "AllowSameLineComments", args)
}
pub fn clipboard_timeout(args: &[&str]) -> String {
    stub_log("directives", "ClipboardTimeout", args)
}
pub fn comment_flag(args: &[&str]) -> String {
    stub_log("directives", "CommentFlag", args)
}
pub fn error_std_out(args: &[&str]) -> String {
    stub_log("directives", "ErrorStdOut", args)
}
pub fn escape_char(args: &[&str]) -> String {
    stub_log("directives", "EscapeChar", args)
}
pub fn hotkey_interval(args: &[&str]) -> String {
    stub_log("directives", "HotkeyInterval", args)
}
pub fn hotkey_modifier_timeout(args: &[&str]) -> String {
    stub_log("directives", "HotkeyModifierTimeout", args)
}
pub fn hotstring(args: &[&str]) -> String {
    stub_log("directives", "Hotstring", args)
}
pub fn if_directive(args: &[&str]) -> String {
    stub_log("directives", "If", args)
}
pub fn if_timeout(args: &[&str]) -> String {
    stub_log("directives", "IfTimeout", args)
}
pub fn if_win_active(args: &[&str]) -> String {
    stub_log("directives", "IfWinActive", args)
}
pub fn include(args: &[&str]) -> String {
    stub_log("directives", "Include", args)
}
pub fn input_level(args: &[&str]) -> String {
    stub_log("directives", "InputLevel", args)
}
pub fn install_keybd_hook(args: &[&str]) -> String {
    stub_log("directives", "InstallKeybdHook", args)
}
pub fn install_mouse_hook(args: &[&str]) -> String {
    stub_log("directives", "InstallMouseHook", args)
}
pub fn key_history(args: &[&str]) -> String {
    stub_log("directives", "KeyHistory", args)
}
pub fn max_hotkeys_per_interval(args: &[&str]) -> String {
    stub_log("directives", "MaxHotkeysPerInterval", args)
}
pub fn max_mem(args: &[&str]) -> String {
    stub_log("directives", "MaxMem", args)
}
pub fn max_threads(args: &[&str]) -> String {
    stub_log("directives", "MaxThreads", args)
}
pub fn max_threads_buffer(args: &[&str]) -> String {
    stub_log("directives", "MaxThreadsBuffer", args)
}
pub fn max_threads_per_hotkey(args: &[&str]) -> String {
    stub_log("directives", "MaxThreadsPerHotkey", args)
}
pub fn menu_mask_key(args: &[&str]) -> String {
    stub_log("directives", "MenuMaskKey", args)
}
pub fn no_env(args: &[&str]) -> String {
    stub_log("directives", "NoEnv", args)
}
pub fn no_tray_icon(args: &[&str]) -> String {
    stub_log("directives", "NoTrayIcon", args)
}
pub fn persistent(args: &[&str]) -> String {
    stub_log("directives", "Persistent", args)
}
pub fn requires(args: &[&str]) -> String {
    stub_log("directives", "Requires", args)
}
pub fn single_instance(args: &[&str]) -> String {
    stub_log("directives", "SingleInstance", args)
}
pub fn use_hook(args: &[&str]) -> String {
    stub_log("directives", "UseHook", args)
}
pub fn warn(args: &[&str]) -> String {
    stub_log("directives", "Warn", args)
}
pub fn win_activate_force(args: &[&str]) -> String {
    stub_log("directives", "WinActivateForce", args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("AllowSameLineComments", allow_same_line_comments),
    ("ClipboardTimeout", clipboard_timeout),
    ("CommentFlag", comment_flag),
    ("ErrorStdOut", error_std_out),
    ("EscapeChar", escape_char),
    ("HotkeyInterval", hotkey_interval),
    ("HotkeyModifierTimeout", hotkey_modifier_timeout),
    ("Hotstring", hotstring),
    ("If", if_directive),
    ("IfTimeout", if_timeout),
    ("IfWinActive", if_win_active),
    ("Include", include),
    ("InputLevel", input_level),
    ("InstallKeybdHook", install_keybd_hook),
    ("InstallMouseHook", install_mouse_hook),
    ("KeyHistory", key_history),
    ("MaxHotkeysPerInterval", max_hotkeys_per_interval),
    ("MaxMem", max_mem),
    ("MaxThreads", max_threads),
    ("MaxThreadsBuffer", max_threads_buffer),
    ("MaxThreadsPerHotkey", max_threads_per_hotkey),
    ("MenuMaskKey", menu_mask_key),
    ("NoEnv", no_env),
    ("NoTrayIcon", no_tray_icon),
    ("Persistent", persistent),
    ("Requires", requires),
    ("SingleInstance", single_instance),
    ("UseHook", use_hook),
    ("Warn", warn),
    ("WinActivateForce", win_activate_force),
];
