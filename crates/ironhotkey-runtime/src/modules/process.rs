use super::{stub_log, ModuleMethod};

pub fn process(args: &[&str]) -> String {
    stub_log("process", "Process", args)
}
pub fn run_cmd(args: &[&str]) -> String {
    stub_log("process", "Run", args)
}
pub fn run_as(args: &[&str]) -> String {
    stub_log("process", "RunAs", args)
}
pub fn run_wait(args: &[&str]) -> String {
    stub_log("process", "RunWait", args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("Process", process),
    ("Run", run_cmd),
    ("RunAs", run_as),
    ("RunWait", run_wait),
];
