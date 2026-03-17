use super::{stub_log, ModuleMethod};

pub fn image_search(args: &[&str]) -> String {
    stub_log("screen", "ImageSearch", args)
}
pub fn pixel_get_color(args: &[&str]) -> String {
    stub_log("screen", "PixelGetColor", args)
}
pub fn pixel_search(args: &[&str]) -> String {
    stub_log("screen", "PixelSearch", args)
}
pub fn sys_get(args: &[&str]) -> String {
    stub_log("screen", "SysGet", args)
}
pub fn monitor_get(args: &[&str]) -> String {
    stub_log("screen", "MonitorGet", args)
}
pub fn monitor_get_count(args: &[&str]) -> String {
    stub_log("screen", "MonitorGetCount", args)
}
pub fn monitor_get_name(args: &[&str]) -> String {
    stub_log("screen", "MonitorGetName", args)
}
pub fn monitor_get_primary(args: &[&str]) -> String {
    stub_log("screen", "MonitorGetPrimary", args)
}
pub fn monitor_get_work_area(args: &[&str]) -> String {
    stub_log("screen", "MonitorGetWorkArea", args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("ImageSearch", image_search),
    ("MonitorGet", monitor_get),
    ("MonitorGetCount", monitor_get_count),
    ("MonitorGetName", monitor_get_name),
    ("MonitorGetPrimary", monitor_get_primary),
    ("MonitorGetWorkArea", monitor_get_work_area),
    ("PixelGetColor", pixel_get_color),
    ("PixelSearch", pixel_search),
    ("SysGet", sys_get),
];
