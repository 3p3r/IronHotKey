use super::{stub_log, ModuleMethod};

pub fn gosub(args: &[&str]) -> String {
    stub_log("flow", "Gosub", args)
}
pub fn goto(args: &[&str]) -> String {
    stub_log("flow", "Goto", args)
}
pub fn on_clipboard_change(args: &[&str]) -> String {
    stub_log("flow", "OnClipboardChange", args)
}
pub fn on_error(args: &[&str]) -> String {
    stub_log("flow", "OnError", args)
}
pub fn on_exit(args: &[&str]) -> String {
    stub_log("flow", "OnExit", args)
}
pub fn on_message(args: &[&str]) -> String {
    stub_log("flow", "OnMessage", args)
}
pub fn set_timer(args: &[&str]) -> String {
    stub_log("flow", "SetTimer", args)
}
pub fn register_label(args: &[&str]) -> String {
    stub_log("flow", "registerLabel", args)
}
pub fn register_function(args: &[&str]) -> String {
    stub_log("flow", "registerFunction", args)
}
pub fn if_legacy(args: &[&str]) -> String {
    stub_log("flow", "ifLegacy", args)
}
pub fn loop_parse(args: &[&str]) -> String {
    stub_log("flow", "loopParse", args)
}
pub fn loop_file(args: &[&str]) -> String {
    stub_log("flow", "loopFile", args)
}
pub fn loop_read(args: &[&str]) -> String {
    stub_log("flow", "loopRead", args)
}
pub fn loop_reg(args: &[&str]) -> String {
    stub_log("flow", "loopReg", args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("Gosub", gosub),
    ("Goto", goto),
    ("OnClipboardChange", on_clipboard_change),
    ("OnError", on_error),
    ("OnExit", on_exit),
    ("OnMessage", on_message),
    ("SetTimer", set_timer),
    ("registerLabel", register_label),
    ("registerFunction", register_function),
    ("ifLegacy", if_legacy),
    ("loopParse", loop_parse),
    ("loopFile", loop_file),
    ("loopRead", loop_read),
    ("loopReg", loop_reg),
];
