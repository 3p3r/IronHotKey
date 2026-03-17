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

pub const METHODS: &[ModuleMethod] = &[
    ("ImageSearch", image_search),
    ("PixelGetColor", pixel_get_color),
    ("PixelSearch", pixel_search),
    ("SysGet", sys_get),
];
