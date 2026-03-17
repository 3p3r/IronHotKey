use super::{stub_log, ModuleMethod};

pub fn ini_delete(args: &[&str]) -> String {
    stub_log("registry", "IniDelete", args)
}
pub fn ini_read(args: &[&str]) -> String {
    stub_log("registry", "IniRead", args)
}
pub fn ini_write(args: &[&str]) -> String {
    stub_log("registry", "IniWrite", args)
}
pub fn loop_reg(args: &[&str]) -> String {
    stub_log("registry", "LoopReg", args)
}
pub fn reg_delete(args: &[&str]) -> String {
    stub_log("registry", "RegDelete", args)
}
pub fn reg_read(args: &[&str]) -> String {
    stub_log("registry", "RegRead", args)
}
pub fn reg_write(args: &[&str]) -> String {
    stub_log("registry", "RegWrite", args)
}
pub fn set_reg_view(args: &[&str]) -> String {
    stub_log("registry", "SetRegView", args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("IniDelete", ini_delete),
    ("IniRead", ini_read),
    ("IniWrite", ini_write),
    ("LoopReg", loop_reg),
    ("RegDelete", reg_delete),
    ("RegRead", reg_read),
    ("RegWrite", reg_write),
    ("SetRegView", set_reg_view),
];
