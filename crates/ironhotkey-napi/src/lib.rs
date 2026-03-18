use napi::bindgen_prelude::*;
use napi_derive::napi;

fn call_runtime(func: fn(&[&str]) -> String, args: Vec<String>) -> String {
    let refs = args.iter().map(|arg| arg.as_str()).collect::<Vec<_>>();
    func(&refs)
}

#[napi]
pub fn parse(source: String) -> Result<String> {
    let script = ironhotkey_parser::parse(&source)
        .map_err(|error| Error::from_reason(format!("parse failed: {error}")))?;
    serde_json::to_string_pretty(&script)
        .map_err(|error| Error::from_reason(format!("serialize failed: {error}")))
}

#[napi]
pub fn codegen(source: String) -> Result<String> {
    let script = ironhotkey_parser::parse(&source)
        .map_err(|error| Error::from_reason(format!("parse failed: {error}")))?;
    ironhotkey_codegen::codegen(&script)
        .map_err(|error| Error::from_reason(format!("codegen failed: {error}")))
}

#[napi]
pub fn run(source: String) -> Result<()> {
    let script = ironhotkey_parser::parse(&source)
        .map_err(|error| Error::from_reason(format!("parse failed: {error}")))?;
    let ts = ironhotkey_codegen::codegen(&script)
        .map_err(|error| Error::from_reason(format!("codegen failed: {error}")))?;
    let js = ironhotkey_codegen::transpile(&ts)
        .map_err(|error| Error::from_reason(format!("transpile failed: {error}")))?;
    ironhotkey_runtime::run(&js)
        .map_err(|error| Error::from_reason(format!("runtime failed: {error}")))?;
    Ok(())
}

macro_rules! napi_module {
    ($class:ident, $mod_path:path, [$(($js_name:expr, $fn_name:ident)),* $(,)?]) => {
        #[napi]
        pub struct $class;

        #[napi]
        impl $class {
            #[napi(constructor)]
            pub fn new() -> Self {
                Self
            }

            $(
                #[napi(js_name = $js_name)]
                pub fn $fn_name(&self, args: Vec<String>) -> String {
                    use $mod_path as module;
                    call_runtime(module::$fn_name, args)
                }
            )*
        }
    };
}

napi_module!(
    AhkEnv,
    ironhotkey_runtime::modules::env,
    [
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
    ]
);

napi_module!(
    AhkDisk,
    ironhotkey_runtime::modules::disk,
    [
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
    ]
);

napi_module!(
    AhkWindow,
    ironhotkey_runtime::modules::window,
    [
        ("Control", control),
        ("ControlClick", control_click),
        ("ControlFocus", control_focus),
        ("ControlGet", control_get),
        ("ControlGetFocus", control_get_focus),
        ("ControlGetPos", control_get_pos),
        ("ControlGetText", control_get_text),
        ("ControlMove", control_move),
        ("ControlSend", control_send),
        ("ControlSendRaw", control_send_raw),
        ("ControlSetText", control_set_text),
        ("DetectHiddenText", detect_hidden_text),
        ("DetectHiddenWindows", detect_hidden_windows),
        ("GroupActivate", group_activate),
        ("GroupAdd", group_add),
        ("GroupClose", group_close),
        ("GroupDeactivate", group_deactivate),
        ("PostMessage", post_message),
        ("SendMessage", send_message),
        ("WinActivate", win_activate),
        ("WinActivateBottom", win_activate_bottom),
        ("WinActive", win_active),
        ("WinClose", win_close),
        ("WinExist", win_exist),
        ("WinGet", win_get),
        ("WinGetActiveStats", win_get_active_stats),
        ("WinGetActiveTitle", win_get_active_title),
        ("WinGetClass", win_get_class),
        ("WinGetPos", win_get_pos),
        ("WinGetText", win_get_text),
        ("WinGetTitle", win_get_title),
        ("WinHide", win_hide),
        ("WinKill", win_kill),
        ("WinMaximize", win_maximize),
        ("WinMenuSelectItem", win_menu_select_item),
        ("WinMinimize", win_minimize),
        ("WinMinimizeAll", win_minimize_all),
        ("WinMinimizeAllUndo", win_minimize_all_undo),
        ("WinMove", win_move),
        ("WinRestore", win_restore),
        ("WinSet", win_set),
        ("WinSetTitle", win_set_title),
        ("WinShow", win_show),
        ("WinWait", win_wait),
        ("WinWaitActive", win_wait_active),
        ("WinWaitClose", win_wait_close),
        ("WinWaitNotActive", win_wait_not_active),
    ]
);

napi_module!(
    AhkGui,
    ironhotkey_runtime::modules::gui,
    [
        ("Edit", edit),
        ("Gui", gui),
        ("GuiControl", gui_control),
        ("GuiControlGet", gui_control_get),
        ("GuiControls", gui_controls),
        ("Input", input),
        ("InputBox", input_box),
        ("InputHook", input_hook),
        ("ListHotkeys", list_hotkeys),
        ("ListView", list_view),
        ("LoadPicture", load_picture),
        ("Menu", menu),
        ("MenuGetHandle", menu_get_handle),
        ("MenuGetName", menu_get_name),
        ("MsgBox", msg_box),
        ("Progress", progress),
        ("SplashTextOn", splash_text_on),
        ("StatusBarGetText", status_bar_get_text),
        ("StatusBarWait", status_bar_wait),
        ("ToolTip", tool_tip),
        ("TreeView", tree_view),
        ("TrayTip", tray_tip),
        ("SplashImage", splash_image),
        ("SplashTextOff", splash_text_off),
        ("LV_Add", lv_add),
        ("LV_Insert", lv_insert),
        ("LV_Modify", lv_modify),
        ("LV_Delete", lv_delete),
        ("LV_ModifyCol", lv_modify_col),
        ("LV_InsertCol", lv_insert_col),
        ("LV_DeleteCol", lv_delete_col),
        ("LV_GetCount", lv_get_count),
        ("LV_GetNext", lv_get_next),
        ("LV_GetText", lv_get_text),
        ("LV_SetImageList", lv_set_image_list),
        ("IL_Create", il_create),
        ("IL_Add", il_add),
        ("IL_Destroy", il_destroy),
        ("TV_Add", tv_add),
        ("TV_Modify", tv_modify),
        ("TV_Delete", tv_delete),
        ("TV_GetSelection", tv_get_selection),
        ("TV_GetCount", tv_get_count),
        ("TV_GetParent", tv_get_parent),
        ("TV_GetChild", tv_get_child),
        ("TV_GetPrev", tv_get_prev),
        ("TV_GetNext", tv_get_next),
        ("TV_GetText", tv_get_text),
        ("TV_Get", tv_get),
        ("TV_SetImageList", tv_set_image_list),
        ("SB_SetText", sb_set_text),
        ("SB_SetParts", sb_set_parts),
        ("SB_SetIcon", sb_set_icon),
    ]
);

napi_module!(
    AhkFlow,
    ironhotkey_runtime::modules::flow,
    [
        ("Gosub", gosub),
        ("Goto", goto),
        ("OnClipboardChange", on_clipboard_change),
        ("OnError", on_error),
        ("OnExit", on_exit),
        ("OnMessage", on_message),
        ("SetTimer", set_timer),
        ("registerLabel", register_label),
        ("registerFunction", register_function),
        ("ifLegacy", if_legacy),
        ("loopParse", loop_parse),
        ("loopFile", loop_file),
        ("loopRead", loop_read),
        ("loopReg", loop_reg),
    ]
);

napi_module!(
    AhkMnk,
    ironhotkey_runtime::modules::mnk,
    [
        ("BlockInput", block_input),
        ("Click", click),
        ("ClipWait", clip_wait),
        ("GetKey", get_key),
        ("GetKeyState", get_key_state),
        ("Hotkey", hotkey),
        ("Hotstring", hotstring),
        ("KeyHistory", key_history),
        ("KeyWait", key_wait),
        ("MouseClick", mouse_click),
        ("MouseClickDrag", mouse_click_drag),
        ("MouseGetPos", mouse_get_pos),
        ("MouseMove", mouse_move),
        ("Send", send),
        ("SendRaw", send_raw),
        ("SendInput", send_input),
        ("SendPlay", send_play),
        ("SendEvent", send_event),
        ("SendLevel", send_level),
        ("SendMode", send_mode),
        ("GetKeyName", get_key_name),
        ("GetKeyVK", get_key_vk),
        ("GetKeySC", get_key_sc),
        ("registerHotkey", register_hotkey),
        ("registerHotstring", register_hotstring),
    ]
);

napi_module!(
    AhkString,
    ironhotkey_runtime::modules::string,
    [
        ("InStr", in_str),
        ("StrGet", str_get),
        ("StrLen", str_len),
        ("StrPut", str_put),
        ("StrReplace", str_replace),
        ("StrSplit", str_split),
        ("StringGetPos", string_get_pos),
        ("StringLeft", string_left),
        ("StringLen", string_len),
        ("StringLower", string_lower),
        ("StringMid", string_mid),
        ("StringReplace", string_replace),
        ("StringSplit", string_split),
        ("StringTrimLeft", string_trim_left),
        ("SubStr", sub_str),
        ("Trim", trim),
        ("LTrim", l_trim),
        ("RTrim", r_trim),
        ("RegExMatch", regex_match),
        ("RegExReplace", regex_replace),
        ("StringUpper", string_upper),
        ("StringRight", string_right),
        ("StringTrimRight", string_trim_right),
    ]
);

napi_module!(
    AhkExt,
    ironhotkey_runtime::modules::ext,
    [
        ("ComObjActive", com_obj_active),
        ("ComObjArray", com_obj_array),
        ("ComObjConnect", com_obj_connect),
        ("ComObjCreate", com_obj_create),
        ("ComObjError", com_obj_error),
        ("ComObjFlags", com_obj_flags),
        ("ComObjGet", com_obj_get),
        ("ComObjQuery", com_obj_query),
        ("ComObjType", com_obj_type),
        ("ComObjValue", com_obj_value),
        ("ComObject", com_object),
        ("ComObjEnwrap", com_obj_enwrap),
        ("ComObjUnwrap", com_obj_unwrap),
        ("ComObjParameter", com_obj_parameter),
        ("ComObjMissing", com_obj_missing),
        ("DllCall", dll_call),
        ("RegisterCallback", register_callback),
        ("URLDownloadToFile", url_download_to_file),
    ]
);

#[napi]
pub struct AhkMaths;

#[napi]
impl AhkMaths {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self
    }

    #[napi(js_name = "Abs")]
    pub fn abs(&self, number: f64) -> String {
        ironhotkey_runtime::modules::maths::abs(number)
    }

    #[napi(js_name = "ACos")]
    pub fn a_cos(&self, number: f64) -> String {
        ironhotkey_runtime::modules::maths::a_cos(number)
    }

    #[napi(js_name = "ASin")]
    pub fn a_sin(&self, number: f64) -> String {
        ironhotkey_runtime::modules::maths::a_sin(number)
    }

    #[napi(js_name = "Asc")]
    pub fn asc(&self, text: String) -> String {
        ironhotkey_runtime::modules::maths::asc(&text)
    }

    #[napi(js_name = "ATan")]
    pub fn a_tan(&self, number: f64) -> String {
        ironhotkey_runtime::modules::maths::a_tan(number)
    }

    #[napi(js_name = "Ceil")]
    pub fn ceil(&self, number: f64) -> String {
        ironhotkey_runtime::modules::maths::ceil(number)
    }

    #[napi(js_name = "Chr")]
    pub fn chr(&self, code: u32) -> String {
        ironhotkey_runtime::modules::maths::chr(code)
    }

    #[napi(js_name = "Cos")]
    pub fn cos(&self, number: f64) -> String {
        ironhotkey_runtime::modules::maths::cos(number)
    }

    #[napi(js_name = "Exp")]
    pub fn exp(&self, number: f64) -> String {
        ironhotkey_runtime::modules::maths::exp(number)
    }

    #[napi(js_name = "Floor")]
    pub fn floor(&self, number: f64) -> String {
        ironhotkey_runtime::modules::maths::floor(number)
    }

    #[napi(js_name = "Format")]
    pub fn format(&self, format_str: String, values: Vec<String>) -> String {
        let refs = values.iter().map(String::as_str).collect::<Vec<_>>();
        ironhotkey_runtime::modules::maths::format(&format_str, &refs)
    }

    #[napi(js_name = "FormatTime")]
    pub fn format_time(&self, timestamp: Option<String>, pattern: Option<String>) -> String {
        ironhotkey_runtime::modules::maths::format_time(timestamp.as_deref(), pattern.as_deref())
    }

    #[napi(js_name = "Ln")]
    pub fn ln(&self, number: f64) -> String {
        ironhotkey_runtime::modules::maths::ln(number)
    }

    #[napi(js_name = "Log")]
    pub fn log(&self, number: f64) -> String {
        ironhotkey_runtime::modules::maths::log(number)
    }

    #[napi(js_name = "Math")]
    pub fn math(&self, expression: String) -> String {
        ironhotkey_runtime::modules::maths::math(&expression)
    }

    #[napi(js_name = "Max")]
    pub fn max(&self, numbers: Vec<f64>) -> String {
        ironhotkey_runtime::modules::maths::max(&numbers)
    }

    #[napi(js_name = "Min")]
    pub fn min(&self, numbers: Vec<f64>) -> String {
        ironhotkey_runtime::modules::maths::min(&numbers)
    }

    #[napi(js_name = "Mod")]
    pub fn mod_fn(&self, dividend: f64, divisor: f64) -> String {
        ironhotkey_runtime::modules::maths::mod_fn(dividend, divisor)
    }

    #[napi(js_name = "NumGet")]
    pub fn num_get(
        &self,
        var_or_address: String,
        offset: Option<i32>,
        kind: Option<String>,
    ) -> String {
        ironhotkey_runtime::modules::maths::num_get(&var_or_address, offset, kind.as_deref())
    }

    #[napi(js_name = "NumPut")]
    pub fn num_put(
        &self,
        number: f64,
        var_or_address: String,
        offset: Option<i32>,
        kind: Option<String>,
    ) -> String {
        ironhotkey_runtime::modules::maths::num_put(
            number,
            &var_or_address,
            offset,
            kind.as_deref(),
        )
    }

    #[napi(js_name = "Ord")]
    pub fn ord(&self, text: String) -> String {
        ironhotkey_runtime::modules::maths::ord(&text)
    }

    #[napi(js_name = "Random")]
    pub fn random(&self, min: Option<f64>, max: Option<f64>) -> String {
        ironhotkey_runtime::modules::maths::random(min, max)
    }

    #[napi(js_name = "Round")]
    pub fn round(&self, number: f64, digits: Option<i32>) -> String {
        ironhotkey_runtime::modules::maths::round(number, digits)
    }

    #[napi(js_name = "Sin")]
    pub fn sin(&self, number: f64) -> String {
        ironhotkey_runtime::modules::maths::sin(number)
    }

    #[napi(js_name = "Sqrt")]
    pub fn sqrt(&self, number: f64) -> String {
        ironhotkey_runtime::modules::maths::sqrt(number)
    }

    #[napi(js_name = "Tan")]
    pub fn tan(&self, number: f64) -> String {
        ironhotkey_runtime::modules::maths::tan(number)
    }
}

napi_module!(
    AhkMisc,
    ironhotkey_runtime::modules::misc,
    [
        ("ListLines", list_lines),
        ("ListVars", list_vars),
        ("OutputDebug", output_debug),
        ("Sleep", sleep),
        ("VerCompare", ver_compare),
        ("Sort", sort),
    ]
);

napi_module!(
    AhkTypes,
    ironhotkey_runtime::modules::types,
    [
        ("Array", array),
        ("Enumerator", enumerator),
        ("Func", func),
        ("IsFunc", is_func),
        ("IsLabel", is_label),
        ("IsObject", is_object),
        ("IsSet", is_set),
        ("IsByRef", is_by_ref),
        ("Object", object),
        ("ObjAddRef", obj_add_ref),
        ("ObjBindMethod", obj_bind_method),
        ("ObjClone", obj_clone),
        ("ObjCount", obj_count),
        ("ObjDelete", obj_delete),
        ("ObjGetAddress", obj_get_address),
        ("ObjGetCapacity", obj_get_capacity),
        ("ObjHasKey", obj_has_key),
        ("ObjInsert", obj_insert),
        ("ObjInsertAt", obj_insert_at),
        ("ObjLength", obj_length),
        ("ObjMaxIndex", obj_max_index),
        ("ObjMinIndex", obj_min_index),
        ("ObjNewEnum", obj_new_enum),
        ("ObjPop", obj_pop),
        ("ObjPush", obj_push),
        ("ObjRelease", obj_release),
        ("ObjRawGet", obj_raw_get),
        ("ObjRawSet", obj_raw_set),
        ("ObjRemove", obj_remove),
        ("ObjRemoveAt", obj_remove_at),
        ("ObjGetBase", obj_get_base),
        ("ObjSetBase", obj_set_base),
        ("ObjSetCapacity", obj_set_capacity),
        ("Exception", exception),
        ("addressOf", address_of),
        ("deref", deref),
        ("registerClass", register_class),
    ]
);

napi_module!(
    AhkProcess,
    ironhotkey_runtime::modules::process,
    [
        ("Process", process),
        ("Run", run_cmd),
        ("RunAs", run_as),
        ("RunWait", run_wait),
    ]
);

napi_module!(
    AhkRegistry,
    ironhotkey_runtime::modules::registry,
    [
        ("IniDelete", ini_delete),
        ("IniRead", ini_read),
        ("IniWrite", ini_write),
        ("LoopReg", loop_reg),
        ("RegDelete", reg_delete),
        ("RegRead", reg_read),
        ("RegWrite", reg_write),
        ("SetRegView", set_reg_view),
    ]
);

napi_module!(
    AhkScreen,
    ironhotkey_runtime::modules::screen,
    [
        ("ImageSearch", image_search),
        ("MonitorGet", monitor_get),
        ("MonitorGetCount", monitor_get_count),
        ("MonitorGetName", monitor_get_name),
        ("MonitorGetPrimary", monitor_get_primary),
        ("MonitorGetWorkArea", monitor_get_work_area),
        ("PixelGetColor", pixel_get_color),
        ("PixelSearch", pixel_search),
        ("SysGet", sys_get),
    ]
);

napi_module!(
    AhkSound,
    ironhotkey_runtime::modules::sound,
    [
        ("SoundBeep", sound_beep),
        ("SoundGet", sound_get),
        ("SoundGetWaveVolume", sound_get_wave_volume),
        ("SoundPlay", sound_play),
        ("SoundSet", sound_set),
        ("SoundSetWaveVolume", sound_set_wave_volume),
    ]
);

napi_module!(
    AhkDirectives,
    ironhotkey_runtime::modules::directives,
    [
        ("AllowSameLineComments", allow_same_line_comments),
        ("ClipboardTimeout", clipboard_timeout),
        ("CommentFlag", comment_flag),
        ("ErrorStdOut", error_std_out),
        ("EscapeChar", escape_char),
        ("HotkeyInterval", hotkey_interval),
        ("HotkeyModifierTimeout", hotkey_modifier_timeout),
        ("Hotstring", hotstring),
        ("If", if_directive),
        ("IfTimeout", if_timeout),
        ("IfWinActive", if_win_active),
        ("Include", include),
        ("InputLevel", input_level),
        ("InstallKeybdHook", install_keybd_hook),
        ("InstallMouseHook", install_mouse_hook),
        ("KeyHistory", key_history),
        ("MaxHotkeysPerInterval", max_hotkeys_per_interval),
        ("MaxMem", max_mem),
        ("MaxThreads", max_threads),
        ("MaxThreadsBuffer", max_threads_buffer),
        ("MaxThreadsPerHotkey", max_threads_per_hotkey),
        ("MenuMaskKey", menu_mask_key),
        ("NoEnv", no_env),
        ("NoTrayIcon", no_tray_icon),
        ("Persistent", persistent),
        ("Requires", requires),
        ("SingleInstance", single_instance),
        ("UseHook", use_hook),
        ("Warn", warn),
        ("WinActivateForce", win_activate_force),
    ]
);
