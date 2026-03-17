use super::{stub_log, ModuleMethod};

pub fn block_input(args: &[&str]) -> String {
    stub_log("mnk", "BlockInput", args)
}
pub fn click(args: &[&str]) -> String {
    stub_log("mnk", "Click", args)
}
pub fn clip_wait(args: &[&str]) -> String {
    stub_log("mnk", "ClipWait", args)
}
pub fn get_key(args: &[&str]) -> String {
    stub_log("mnk", "GetKey", args)
}
pub fn get_key_state(args: &[&str]) -> String {
    stub_log("mnk", "GetKeyState", args)
}
pub fn hotkey(args: &[&str]) -> String {
    stub_log("mnk", "Hotkey", args)
}
pub fn hotstring(args: &[&str]) -> String {
    stub_log("mnk", "Hotstring", args)
}
pub fn key_history(args: &[&str]) -> String {
    stub_log("mnk", "KeyHistory", args)
}
pub fn key_wait(args: &[&str]) -> String {
    stub_log("mnk", "KeyWait", args)
}
pub fn mouse_click(args: &[&str]) -> String {
    stub_log("mnk", "MouseClick", args)
}
pub fn mouse_click_drag(args: &[&str]) -> String {
    stub_log("mnk", "MouseClickDrag", args)
}
pub fn mouse_get_pos(args: &[&str]) -> String {
    stub_log("mnk", "MouseGetPos", args)
}
pub fn mouse_move(args: &[&str]) -> String {
    stub_log("mnk", "MouseMove", args)
}
pub fn send(args: &[&str]) -> String {
    stub_log("mnk", "Send", args)
}
pub fn send_level(args: &[&str]) -> String {
    stub_log("mnk", "SendLevel", args)
}
pub fn send_mode(args: &[&str]) -> String {
    stub_log("mnk", "SendMode", args)
}
pub fn send_raw(args: &[&str]) -> String {
    stub_log("mnk", "SendRaw", args)
}
pub fn send_input(args: &[&str]) -> String {
    stub_log("mnk", "SendInput", args)
}
pub fn send_play(args: &[&str]) -> String {
    stub_log("mnk", "SendPlay", args)
}
pub fn send_event(args: &[&str]) -> String {
    stub_log("mnk", "SendEvent", args)
}
pub fn get_key_name(args: &[&str]) -> String {
    stub_log("mnk", "GetKeyName", args)
}
pub fn get_key_vk(args: &[&str]) -> String {
    stub_log("mnk", "GetKeyVK", args)
}
pub fn get_key_sc(args: &[&str]) -> String {
    stub_log("mnk", "GetKeySC", args)
}
pub fn register_hotkey(args: &[&str]) -> String {
    stub_log("mnk", "registerHotkey", args)
}
pub fn register_hotstring(args: &[&str]) -> String {
    stub_log("mnk", "registerHotstring", args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("BlockInput", block_input),
    ("Click", click),
    ("ClipWait", clip_wait),
    ("GetKey", get_key),
    ("GetKeyName", get_key_name),
    ("GetKeySC", get_key_sc),
    ("GetKeyState", get_key_state),
    ("GetKeyVK", get_key_vk),
    ("Hotkey", hotkey),
    ("Hotstring", hotstring),
    ("KeyHistory", key_history),
    ("KeyWait", key_wait),
    ("MouseClick", mouse_click),
    ("MouseClickDrag", mouse_click_drag),
    ("MouseGetPos", mouse_get_pos),
    ("MouseMove", mouse_move),
    ("Send", send),
    ("SendEvent", send_event),
    ("SendInput", send_input),
    ("SendLevel", send_level),
    ("SendMode", send_mode),
    ("SendPlay", send_play),
    ("SendRaw", send_raw),
    ("registerHotkey", register_hotkey),
    ("registerHotstring", register_hotstring),
];
