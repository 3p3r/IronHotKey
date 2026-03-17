use super::{stub_log, ModuleMethod};

pub fn auto_trim(args: &[&str]) -> String {
    stub_log("env", "AutoTrim", args)
}
pub fn coord_mode(args: &[&str]) -> String {
    stub_log("env", "CoordMode", args)
}
pub fn critical(args: &[&str]) -> String {
    stub_log("env", "Critical", args)
}
pub fn env_add(args: &[&str]) -> String {
    stub_log("env", "EnvAdd", args)
}
pub fn env_div(args: &[&str]) -> String {
    stub_log("env", "EnvDiv", args)
}
pub fn env_get(args: &[&str]) -> String {
    stub_log("env", "EnvGet", args)
}
pub fn env_mult(args: &[&str]) -> String {
    stub_log("env", "EnvMult", args)
}
pub fn env_set(args: &[&str]) -> String {
    stub_log("env", "EnvSet", args)
}
pub fn env_sub(args: &[&str]) -> String {
    stub_log("env", "EnvSub", args)
}
pub fn env_update(args: &[&str]) -> String {
    stub_log("env", "EnvUpdate", args)
}
pub fn exit(args: &[&str]) -> String {
    stub_log("env", "Exit", args)
}
pub fn exit_app(args: &[&str]) -> String {
    stub_log("env", "ExitApp", args)
}
pub fn file_encoding(args: &[&str]) -> String {
    stub_log("env", "FileEncoding", args)
}
pub fn pause(args: &[&str]) -> String {
    stub_log("env", "Pause", args)
}
pub fn reload(args: &[&str]) -> String {
    stub_log("env", "Reload", args)
}
pub fn set_batch_lines(args: &[&str]) -> String {
    stub_log("env", "SetBatchLines", args)
}
pub fn set_control_delay(args: &[&str]) -> String {
    stub_log("env", "SetControlDelay", args)
}
pub fn set_default_mouse_speed(args: &[&str]) -> String {
    stub_log("env", "SetDefaultMouseSpeed", args)
}
pub fn set_env(args: &[&str]) -> String {
    stub_log("env", "SetEnv", args)
}
pub fn set_expression(args: &[&str]) -> String {
    stub_log("env", "SetExpression", args)
}
pub fn set_format(args: &[&str]) -> String {
    stub_log("env", "SetFormat", args)
}
pub fn set_key_delay(args: &[&str]) -> String {
    stub_log("env", "SetKeyDelay", args)
}
pub fn set_mouse_delay(args: &[&str]) -> String {
    stub_log("env", "SetMouseDelay", args)
}
pub fn set_num_scroll_caps_lock_state(args: &[&str]) -> String {
    stub_log("env", "SetNumScrollCapsLockState", args)
}
pub fn set_store_caps_lock_mode(args: &[&str]) -> String {
    stub_log("env", "SetStoreCapsLockMode", args)
}
pub fn set_title_match_mode(args: &[&str]) -> String {
    stub_log("env", "SetTitleMatchMode", args)
}
pub fn set_win_delay(args: &[&str]) -> String {
    stub_log("env", "SetWinDelay", args)
}
pub fn set_working_dir(args: &[&str]) -> String {
    stub_log("env", "SetWorkingDir", args)
}
pub fn shutdown(args: &[&str]) -> String {
    stub_log("env", "Shutdown", args)
}
pub fn string_case_sense(args: &[&str]) -> String {
    stub_log("env", "StringCaseSense", args)
}
pub fn suspend(args: &[&str]) -> String {
    stub_log("env", "Suspend", args)
}
pub fn thread(args: &[&str]) -> String {
    stub_log("env", "Thread", args)
}
pub fn transform(args: &[&str]) -> String {
    stub_log("env", "Transform", args)
}
pub fn var_set_capacity(args: &[&str]) -> String {
    stub_log("env", "VarSetCapacity", args)
}
pub fn set(args: &[&str]) -> String {
    stub_log("env", "set", args)
}
pub fn get(args: &[&str]) -> String {
    stub_log("env", "get", args)
}
pub fn get_built_in(args: &[&str]) -> String {
    stub_log("env", "getBuiltIn", args)
}
pub fn push_scope(args: &[&str]) -> String {
    stub_log("env", "pushScope", args)
}
pub fn pop_scope(args: &[&str]) -> String {
    stub_log("env", "popScope", args)
}
pub fn declare_global(args: &[&str]) -> String {
    stub_log("env", "declareGlobal", args)
}
pub fn declare_local(args: &[&str]) -> String {
    stub_log("env", "declareLocal", args)
}
pub fn declare_static(args: &[&str]) -> String {
    stub_log("env", "declareStatic", args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("AutoTrim", auto_trim),
    ("CoordMode", coord_mode),
    ("Critical", critical),
    ("EnvAdd", env_add),
    ("EnvDiv", env_div),
    ("EnvGet", env_get),
    ("EnvMult", env_mult),
    ("EnvSet", env_set),
    ("EnvSub", env_sub),
    ("EnvUpdate", env_update),
    ("Exit", exit),
    ("ExitApp", exit_app),
    ("FileEncoding", file_encoding),
    ("Pause", pause),
    ("Reload", reload),
    ("SetBatchLines", set_batch_lines),
    ("SetControlDelay", set_control_delay),
    ("SetDefaultMouseSpeed", set_default_mouse_speed),
    ("SetEnv", set_env),
    ("SetExpression", set_expression),
    ("SetFormat", set_format),
    ("SetKeyDelay", set_key_delay),
    ("SetMouseDelay", set_mouse_delay),
    ("SetNumScrollCapsLockState", set_num_scroll_caps_lock_state),
    ("SetStoreCapsLockMode", set_store_caps_lock_mode),
    ("SetTitleMatchMode", set_title_match_mode),
    ("SetWinDelay", set_win_delay),
    ("SetWorkingDir", set_working_dir),
    ("Shutdown", shutdown),
    ("StringCaseSense", string_case_sense),
    ("Suspend", suspend),
    ("Thread", thread),
    ("Transform", transform),
    ("VarSetCapacity", var_set_capacity),
    ("set", set),
    ("get", get),
    ("getBuiltIn", get_built_in),
    ("pushScope", push_scope),
    ("popScope", pop_scope),
    ("declareGlobal", declare_global),
    ("declareLocal", declare_local),
    ("declareStatic", declare_static),
];
