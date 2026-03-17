use super::{stub_log, ModuleMethod};

pub fn edit(args: &[&str]) -> String {
    stub_log("gui", "Edit", args)
}
pub fn gui(args: &[&str]) -> String {
    stub_log("gui", "Gui", args)
}
pub fn gui_control(args: &[&str]) -> String {
    stub_log("gui", "GuiControl", args)
}
pub fn gui_control_get(args: &[&str]) -> String {
    stub_log("gui", "GuiControlGet", args)
}
pub fn gui_controls(args: &[&str]) -> String {
    stub_log("gui", "GuiControls", args)
}
pub fn input(args: &[&str]) -> String {
    stub_log("gui", "Input", args)
}
pub fn input_box(args: &[&str]) -> String {
    stub_log("gui", "InputBox", args)
}
pub fn input_hook(args: &[&str]) -> String {
    stub_log("gui", "InputHook", args)
}
pub fn list_hotkeys(args: &[&str]) -> String {
    stub_log("gui", "ListHotkeys", args)
}
pub fn list_view(args: &[&str]) -> String {
    stub_log("gui", "ListView", args)
}
pub fn load_picture(args: &[&str]) -> String {
    stub_log("gui", "LoadPicture", args)
}
pub fn menu(args: &[&str]) -> String {
    stub_log("gui", "Menu", args)
}
pub fn menu_get_handle(args: &[&str]) -> String {
    stub_log("gui", "MenuGetHandle", args)
}
pub fn menu_get_name(args: &[&str]) -> String {
    stub_log("gui", "MenuGetName", args)
}
pub fn msg_box(args: &[&str]) -> String {
    stub_log("gui", "MsgBox", args)
}
pub fn progress(args: &[&str]) -> String {
    stub_log("gui", "Progress", args)
}
pub fn splash_text_on(args: &[&str]) -> String {
    stub_log("gui", "SplashTextOn", args)
}
pub fn status_bar_get_text(args: &[&str]) -> String {
    stub_log("gui", "StatusBarGetText", args)
}
pub fn status_bar_wait(args: &[&str]) -> String {
    stub_log("gui", "StatusBarWait", args)
}
pub fn tool_tip(args: &[&str]) -> String {
    stub_log("gui", "ToolTip", args)
}
pub fn tree_view(args: &[&str]) -> String {
    stub_log("gui", "TreeView", args)
}
pub fn tray_tip(args: &[&str]) -> String {
    stub_log("gui", "TrayTip", args)
}
pub fn splash_image(args: &[&str]) -> String {
    stub_log("gui", "SplashImage", args)
}
pub fn splash_text_off(args: &[&str]) -> String {
    stub_log("gui", "SplashTextOff", args)
}
pub fn lv_add(args: &[&str]) -> String {
    stub_log("gui", "LV_Add", args)
}
pub fn lv_insert(args: &[&str]) -> String {
    stub_log("gui", "LV_Insert", args)
}
pub fn lv_modify(args: &[&str]) -> String {
    stub_log("gui", "LV_Modify", args)
}
pub fn lv_delete(args: &[&str]) -> String {
    stub_log("gui", "LV_Delete", args)
}
pub fn lv_modify_col(args: &[&str]) -> String {
    stub_log("gui", "LV_ModifyCol", args)
}
pub fn lv_insert_col(args: &[&str]) -> String {
    stub_log("gui", "LV_InsertCol", args)
}
pub fn lv_delete_col(args: &[&str]) -> String {
    stub_log("gui", "LV_DeleteCol", args)
}
pub fn lv_get_count(args: &[&str]) -> String {
    stub_log("gui", "LV_GetCount", args)
}
pub fn lv_get_next(args: &[&str]) -> String {
    stub_log("gui", "LV_GetNext", args)
}
pub fn lv_get_text(args: &[&str]) -> String {
    stub_log("gui", "LV_GetText", args)
}
pub fn lv_set_image_list(args: &[&str]) -> String {
    stub_log("gui", "LV_SetImageList", args)
}
pub fn il_create(args: &[&str]) -> String {
    stub_log("gui", "IL_Create", args)
}
pub fn il_add(args: &[&str]) -> String {
    stub_log("gui", "IL_Add", args)
}
pub fn il_destroy(args: &[&str]) -> String {
    stub_log("gui", "IL_Destroy", args)
}
pub fn tv_add(args: &[&str]) -> String {
    stub_log("gui", "TV_Add", args)
}
pub fn tv_modify(args: &[&str]) -> String {
    stub_log("gui", "TV_Modify", args)
}
pub fn tv_delete(args: &[&str]) -> String {
    stub_log("gui", "TV_Delete", args)
}
pub fn tv_get_selection(args: &[&str]) -> String {
    stub_log("gui", "TV_GetSelection", args)
}
pub fn tv_get_count(args: &[&str]) -> String {
    stub_log("gui", "TV_GetCount", args)
}
pub fn tv_get_parent(args: &[&str]) -> String {
    stub_log("gui", "TV_GetParent", args)
}
pub fn tv_get_child(args: &[&str]) -> String {
    stub_log("gui", "TV_GetChild", args)
}
pub fn tv_get_prev(args: &[&str]) -> String {
    stub_log("gui", "TV_GetPrev", args)
}
pub fn tv_get_next(args: &[&str]) -> String {
    stub_log("gui", "TV_GetNext", args)
}
pub fn tv_get_text(args: &[&str]) -> String {
    stub_log("gui", "TV_GetText", args)
}
pub fn tv_get(args: &[&str]) -> String {
    stub_log("gui", "TV_Get", args)
}
pub fn tv_set_image_list(args: &[&str]) -> String {
    stub_log("gui", "TV_SetImageList", args)
}
pub fn sb_set_text(args: &[&str]) -> String {
    stub_log("gui", "SB_SetText", args)
}
pub fn sb_set_parts(args: &[&str]) -> String {
    stub_log("gui", "SB_SetParts", args)
}
pub fn sb_set_icon(args: &[&str]) -> String {
    stub_log("gui", "SB_SetIcon", args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("Edit", edit),
    ("Gui", gui),
    ("GuiControl", gui_control),
    ("GuiControlGet", gui_control_get),
    ("GuiControls", gui_controls),
    ("IL_Add", il_add),
    ("IL_Create", il_create),
    ("IL_Destroy", il_destroy),
    ("Input", input),
    ("InputBox", input_box),
    ("InputHook", input_hook),
    ("LV_Add", lv_add),
    ("LV_Delete", lv_delete),
    ("LV_DeleteCol", lv_delete_col),
    ("LV_GetCount", lv_get_count),
    ("LV_GetNext", lv_get_next),
    ("LV_GetText", lv_get_text),
    ("LV_Insert", lv_insert),
    ("LV_InsertCol", lv_insert_col),
    ("LV_Modify", lv_modify),
    ("LV_ModifyCol", lv_modify_col),
    ("LV_SetImageList", lv_set_image_list),
    ("ListHotkeys", list_hotkeys),
    ("ListView", list_view),
    ("LoadPicture", load_picture),
    ("Menu", menu),
    ("MenuGetHandle", menu_get_handle),
    ("MenuGetName", menu_get_name),
    ("MsgBox", msg_box),
    ("Progress", progress),
    ("SB_SetIcon", sb_set_icon),
    ("SB_SetParts", sb_set_parts),
    ("SB_SetText", sb_set_text),
    ("SplashImage", splash_image),
    ("SplashTextOff", splash_text_off),
    ("SplashTextOn", splash_text_on),
    ("StatusBarGetText", status_bar_get_text),
    ("StatusBarWait", status_bar_wait),
    ("TV_Add", tv_add),
    ("TV_Delete", tv_delete),
    ("TV_Get", tv_get),
    ("TV_GetChild", tv_get_child),
    ("TV_GetCount", tv_get_count),
    ("TV_GetNext", tv_get_next),
    ("TV_GetParent", tv_get_parent),
    ("TV_GetPrev", tv_get_prev),
    ("TV_GetSelection", tv_get_selection),
    ("TV_GetText", tv_get_text),
    ("TV_Modify", tv_modify),
    ("TV_SetImageList", tv_set_image_list),
    ("ToolTip", tool_tip),
    ("TrayTip", tray_tip),
    ("TreeView", tree_view),
];
