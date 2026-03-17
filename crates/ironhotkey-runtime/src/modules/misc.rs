use super::{stub_log, ModuleMethod};

pub fn list_lines(args: &[&str]) -> String {
    stub_log("misc", "ListLines", args)
}
pub fn list_vars(args: &[&str]) -> String {
    stub_log("misc", "ListVars", args)
}
pub fn output_debug(args: &[&str]) -> String {
    stub_log("misc", "OutputDebug", args)
}
pub fn sleep(args: &[&str]) -> String {
    stub_log("misc", "Sleep", args)
}
pub fn ver_compare(args: &[&str]) -> String {
    stub_log("misc", "VerCompare", args)
}
pub fn sort(args: &[&str]) -> String {
    stub_log("misc", "Sort", args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("ListLines", list_lines),
    ("ListVars", list_vars),
    ("OutputDebug", output_debug),
    ("Sleep", sleep),
    ("VerCompare", ver_compare),
    ("Sort", sort),
];
