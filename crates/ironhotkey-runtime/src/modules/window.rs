use super::{stub_log, ModuleMethod};

pub fn control(args: &[&str]) -> String {
    stub_log("window", "Control", args)
}
pub fn control_click(args: &[&str]) -> String {
    stub_log("window", "ControlClick", args)
}
pub fn control_focus(args: &[&str]) -> String {
    stub_log("window", "ControlFocus", args)
}
pub fn control_get(args: &[&str]) -> String {
    stub_log("window", "ControlGet", args)
}
pub fn control_get_focus(args: &[&str]) -> String {
    stub_log("window", "ControlGetFocus", args)
}
pub fn control_get_pos(args: &[&str]) -> String {
    stub_log("window", "ControlGetPos", args)
}
pub fn control_get_text(args: &[&str]) -> String {
    stub_log("window", "ControlGetText", args)
}
pub fn control_move(args: &[&str]) -> String {
    stub_log("window", "ControlMove", args)
}
pub fn control_send(args: &[&str]) -> String {
    stub_log("window", "ControlSend", args)
}
pub fn control_send_raw(args: &[&str]) -> String {
    stub_log("window", "ControlSendRaw", args)
}
pub fn control_set_text(args: &[&str]) -> String {
    stub_log("window", "ControlSetText", args)
}
pub fn detect_hidden_text(args: &[&str]) -> String {
    stub_log("window", "DetectHiddenText", args)
}
pub fn detect_hidden_windows(args: &[&str]) -> String {
    stub_log("window", "DetectHiddenWindows", args)
}
pub fn group_activate(args: &[&str]) -> String {
    stub_log("window", "GroupActivate", args)
}
pub fn group_add(args: &[&str]) -> String {
    stub_log("window", "GroupAdd", args)
}
pub fn group_close(args: &[&str]) -> String {
    stub_log("window", "GroupClose", args)
}
pub fn group_deactivate(args: &[&str]) -> String {
    stub_log("window", "GroupDeactivate", args)
}
pub fn post_message(args: &[&str]) -> String {
    stub_log("window", "PostMessage", args)
}
pub fn send_message(args: &[&str]) -> String {
    stub_log("window", "SendMessage", args)
}
pub fn win_activate(args: &[&str]) -> String {
    stub_log("window", "WinActivate", args)
}
pub fn win_activate_bottom(args: &[&str]) -> String {
    stub_log("window", "WinActivateBottom", args)
}
pub fn win_active(args: &[&str]) -> String {
    stub_log("window", "WinActive", args)
}
pub fn win_close(args: &[&str]) -> String {
    stub_log("window", "WinClose", args)
}
pub fn win_exist(args: &[&str]) -> String {
    stub_log("window", "WinExist", args)
}
pub fn win_get(args: &[&str]) -> String {
    stub_log("window", "WinGet", args)
}
pub fn win_get_active_stats(args: &[&str]) -> String {
    stub_log("window", "WinGetActiveStats", args)
}
pub fn win_get_active_title(args: &[&str]) -> String {
    stub_log("window", "WinGetActiveTitle", args)
}
pub fn win_get_class(args: &[&str]) -> String {
    stub_log("window", "WinGetClass", args)
}
pub fn win_get_pos(args: &[&str]) -> String {
    stub_log("window", "WinGetPos", args)
}
pub fn win_get_text(args: &[&str]) -> String {
    stub_log("window", "WinGetText", args)
}
pub fn win_get_title(args: &[&str]) -> String {
    stub_log("window", "WinGetTitle", args)
}
pub fn win_hide(args: &[&str]) -> String {
    stub_log("window", "WinHide", args)
}
pub fn win_kill(args: &[&str]) -> String {
    stub_log("window", "WinKill", args)
}
pub fn win_maximize(args: &[&str]) -> String {
    stub_log("window", "WinMaximize", args)
}
pub fn win_menu_select_item(args: &[&str]) -> String {
    stub_log("window", "WinMenuSelectItem", args)
}
pub fn win_minimize(args: &[&str]) -> String {
    stub_log("window", "WinMinimize", args)
}
pub fn win_minimize_all(args: &[&str]) -> String {
    stub_log("window", "WinMinimizeAll", args)
}
pub fn win_minimize_all_undo(args: &[&str]) -> String {
    stub_log("window", "WinMinimizeAllUndo", args)
}
pub fn win_move(args: &[&str]) -> String {
    stub_log("window", "WinMove", args)
}
pub fn win_restore(args: &[&str]) -> String {
    stub_log("window", "WinRestore", args)
}
pub fn win_set(args: &[&str]) -> String {
    stub_log("window", "WinSet", args)
}
pub fn win_set_title(args: &[&str]) -> String {
    stub_log("window", "WinSetTitle", args)
}
pub fn win_show(args: &[&str]) -> String {
    stub_log("window", "WinShow", args)
}
pub fn win_wait(args: &[&str]) -> String {
    stub_log("window", "WinWait", args)
}
pub fn win_wait_active(args: &[&str]) -> String {
    stub_log("window", "WinWaitActive", args)
}
pub fn win_wait_close(args: &[&str]) -> String {
    stub_log("window", "WinWaitClose", args)
}
pub fn win_wait_not_active(args: &[&str]) -> String {
    stub_log("window", "WinWaitNotActive", args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("Control", control),
    ("ControlClick", control_click),
    ("ControlFocus", control_focus),
    ("ControlGet", control_get),
    ("ControlGetFocus", control_get_focus),
    ("ControlGetPos", control_get_pos),
    ("ControlGetText", control_get_text),
    ("ControlMove", control_move),
    ("ControlSend", control_send),
    ("ControlSendRaw", control_send_raw),
    ("ControlSetText", control_set_text),
    ("DetectHiddenText", detect_hidden_text),
    ("DetectHiddenWindows", detect_hidden_windows),
    ("GroupActivate", group_activate),
    ("GroupAdd", group_add),
    ("GroupClose", group_close),
    ("GroupDeactivate", group_deactivate),
    ("PostMessage", post_message),
    ("SendMessage", send_message),
    ("WinActivate", win_activate),
    ("WinActivateBottom", win_activate_bottom),
    ("WinActive", win_active),
    ("WinClose", win_close),
    ("WinExist", win_exist),
    ("WinGet", win_get),
    ("WinGetActiveStats", win_get_active_stats),
    ("WinGetActiveTitle", win_get_active_title),
    ("WinGetClass", win_get_class),
    ("WinGetPos", win_get_pos),
    ("WinGetText", win_get_text),
    ("WinGetTitle", win_get_title),
    ("WinHide", win_hide),
    ("WinKill", win_kill),
    ("WinMaximize", win_maximize),
    ("WinMenuSelectItem", win_menu_select_item),
    ("WinMinimize", win_minimize),
    ("WinMinimizeAll", win_minimize_all),
    ("WinMinimizeAllUndo", win_minimize_all_undo),
    ("WinMove", win_move),
    ("WinRestore", win_restore),
    ("WinSet", win_set),
    ("WinSetTitle", win_set_title),
    ("WinShow", win_show),
    ("WinWait", win_wait),
    ("WinWaitActive", win_wait_active),
    ("WinWaitClose", win_wait_close),
    ("WinWaitNotActive", win_wait_not_active),
];
