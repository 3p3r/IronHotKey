use super::{stub_log, ModuleMethod};

pub fn drive(args: &[&str]) -> String {
    stub_log("disk", "Drive", args)
}
pub fn drive_get(args: &[&str]) -> String {
    stub_log("disk", "DriveGet", args)
}
pub fn drive_space_free(args: &[&str]) -> String {
    stub_log("disk", "DriveSpaceFree", args)
}
pub fn file(args: &[&str]) -> String {
    stub_log("disk", "File", args)
}
pub fn file_append(args: &[&str]) -> String {
    stub_log("disk", "FileAppend", args)
}
pub fn file_copy(args: &[&str]) -> String {
    stub_log("disk", "FileCopy", args)
}
pub fn file_copy_dir(args: &[&str]) -> String {
    stub_log("disk", "FileCopyDir", args)
}
pub fn file_create_dir(args: &[&str]) -> String {
    stub_log("disk", "FileCreateDir", args)
}
pub fn file_create_shortcut(args: &[&str]) -> String {
    stub_log("disk", "FileCreateShortcut", args)
}
pub fn file_delete(args: &[&str]) -> String {
    stub_log("disk", "FileDelete", args)
}
pub fn file_exist(args: &[&str]) -> String {
    stub_log("disk", "FileExist", args)
}
pub fn file_get_attrib(args: &[&str]) -> String {
    stub_log("disk", "FileGetAttrib", args)
}
pub fn file_get_shortcut(args: &[&str]) -> String {
    stub_log("disk", "FileGetShortcut", args)
}
pub fn file_get_size(args: &[&str]) -> String {
    stub_log("disk", "FileGetSize", args)
}
pub fn file_get_time(args: &[&str]) -> String {
    stub_log("disk", "FileGetTime", args)
}
pub fn file_get_version(args: &[&str]) -> String {
    stub_log("disk", "FileGetVersion", args)
}
pub fn file_install(args: &[&str]) -> String {
    stub_log("disk", "FileInstall", args)
}
pub fn file_move(args: &[&str]) -> String {
    stub_log("disk", "FileMove", args)
}
pub fn file_move_dir(args: &[&str]) -> String {
    stub_log("disk", "FileMoveDir", args)
}
pub fn file_open(args: &[&str]) -> String {
    stub_log("disk", "FileOpen", args)
}
pub fn file_read(args: &[&str]) -> String {
    stub_log("disk", "FileRead", args)
}
pub fn file_read_line(args: &[&str]) -> String {
    stub_log("disk", "FileReadLine", args)
}
pub fn file_recycle(args: &[&str]) -> String {
    stub_log("disk", "FileRecycle", args)
}
pub fn file_recycle_empty(args: &[&str]) -> String {
    stub_log("disk", "FileRecycleEmpty", args)
}
pub fn file_remove_dir(args: &[&str]) -> String {
    stub_log("disk", "FileRemoveDir", args)
}
pub fn file_select_file(args: &[&str]) -> String {
    stub_log("disk", "FileSelectFile", args)
}
pub fn file_select_folder(args: &[&str]) -> String {
    stub_log("disk", "FileSelectFolder", args)
}
pub fn file_set_attrib(args: &[&str]) -> String {
    stub_log("disk", "FileSetAttrib", args)
}
pub fn file_set_time(args: &[&str]) -> String {
    stub_log("disk", "FileSetTime", args)
}
pub fn loop_file(args: &[&str]) -> String {
    stub_log("disk", "LoopFile", args)
}
pub fn loop_read_file(args: &[&str]) -> String {
    stub_log("disk", "LoopReadFile", args)
}
pub fn split_path(args: &[&str]) -> String {
    stub_log("disk", "SplitPath", args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("Drive", drive),
    ("DriveGet", drive_get),
    ("DriveSpaceFree", drive_space_free),
    ("File", file),
    ("FileAppend", file_append),
    ("FileCopy", file_copy),
    ("FileCopyDir", file_copy_dir),
    ("FileCreateDir", file_create_dir),
    ("FileCreateShortcut", file_create_shortcut),
    ("FileDelete", file_delete),
    ("FileExist", file_exist),
    ("FileGetAttrib", file_get_attrib),
    ("FileGetShortcut", file_get_shortcut),
    ("FileGetSize", file_get_size),
    ("FileGetTime", file_get_time),
    ("FileGetVersion", file_get_version),
    ("FileInstall", file_install),
    ("FileMove", file_move),
    ("FileMoveDir", file_move_dir),
    ("FileOpen", file_open),
    ("FileRead", file_read),
    ("FileReadLine", file_read_line),
    ("FileRecycle", file_recycle),
    ("FileRecycleEmpty", file_recycle_empty),
    ("FileRemoveDir", file_remove_dir),
    ("FileSelectFile", file_select_file),
    ("FileSelectFolder", file_select_folder),
    ("FileSetAttrib", file_set_attrib),
    ("FileSetTime", file_set_time),
    ("LoopFile", loop_file),
    ("LoopReadFile", loop_read_file),
    ("SplitPath", split_path),
];
