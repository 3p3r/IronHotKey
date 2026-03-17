use super::{stub_log, ModuleMethod};

pub fn com_obj_active(args: &[&str]) -> String {
    stub_log("ext", "ComObjActive", args)
}
pub fn com_obj_array(args: &[&str]) -> String {
    stub_log("ext", "ComObjArray", args)
}
pub fn com_obj_connect(args: &[&str]) -> String {
    stub_log("ext", "ComObjConnect", args)
}
pub fn com_obj_create(args: &[&str]) -> String {
    stub_log("ext", "ComObjCreate", args)
}
pub fn com_obj_error(args: &[&str]) -> String {
    stub_log("ext", "ComObjError", args)
}
pub fn com_obj_flags(args: &[&str]) -> String {
    stub_log("ext", "ComObjFlags", args)
}
pub fn com_obj_get(args: &[&str]) -> String {
    stub_log("ext", "ComObjGet", args)
}
pub fn com_obj_query(args: &[&str]) -> String {
    stub_log("ext", "ComObjQuery", args)
}
pub fn com_obj_type(args: &[&str]) -> String {
    stub_log("ext", "ComObjType", args)
}
pub fn com_obj_value(args: &[&str]) -> String {
    stub_log("ext", "ComObjValue", args)
}
pub fn dll_call(args: &[&str]) -> String {
    stub_log("ext", "DllCall", args)
}
pub fn register_callback(args: &[&str]) -> String {
    stub_log("ext", "RegisterCallback", args)
}
pub fn url_download_to_file(args: &[&str]) -> String {
    stub_log("ext", "URLDownloadToFile", args)
}
pub fn com_object(args: &[&str]) -> String {
    stub_log("ext", "ComObject", args)
}
pub fn com_obj_parameter(args: &[&str]) -> String {
    stub_log("ext", "ComObjParameter", args)
}
pub fn com_obj_missing(args: &[&str]) -> String {
    stub_log("ext", "ComObjMissing", args)
}
pub fn com_obj_enwrap(args: &[&str]) -> String {
    stub_log("ext", "ComObjEnwrap", args)
}
pub fn com_obj_unwrap(args: &[&str]) -> String {
    stub_log("ext", "ComObjUnwrap", args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("ComObjActive", com_obj_active),
    ("ComObjArray", com_obj_array),
    ("ComObjConnect", com_obj_connect),
    ("ComObjCreate", com_obj_create),
    ("ComObjEnwrap", com_obj_enwrap),
    ("ComObjError", com_obj_error),
    ("ComObjFlags", com_obj_flags),
    ("ComObjGet", com_obj_get),
    ("ComObjMissing", com_obj_missing),
    ("ComObjParameter", com_obj_parameter),
    ("ComObjQuery", com_obj_query),
    ("ComObjType", com_obj_type),
    ("ComObjUnwrap", com_obj_unwrap),
    ("ComObjValue", com_obj_value),
    ("ComObject", com_object),
    ("DllCall", dll_call),
    ("RegisterCallback", register_callback),
    ("URLDownloadToFile", url_download_to_file),
];
