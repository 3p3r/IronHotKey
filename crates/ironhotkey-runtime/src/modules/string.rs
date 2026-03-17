use super::{stub_log, ModuleMethod};

pub fn in_str(args: &[&str]) -> String {
    stub_log("string", "InStr", args)
}
pub fn str_get(args: &[&str]) -> String {
    stub_log("string", "StrGet", args)
}
pub fn str_len(args: &[&str]) -> String {
    stub_log("string", "StrLen", args)
}
pub fn str_put(args: &[&str]) -> String {
    stub_log("string", "StrPut", args)
}
pub fn str_replace(args: &[&str]) -> String {
    stub_log("string", "StrReplace", args)
}
pub fn str_split(args: &[&str]) -> String {
    stub_log("string", "StrSplit", args)
}
pub fn string_get_pos(args: &[&str]) -> String {
    stub_log("string", "StringGetPos", args)
}
pub fn string_left(args: &[&str]) -> String {
    stub_log("string", "StringLeft", args)
}
pub fn string_len(args: &[&str]) -> String {
    stub_log("string", "StringLen", args)
}
pub fn string_lower(args: &[&str]) -> String {
    stub_log("string", "StringLower", args)
}
pub fn string_mid(args: &[&str]) -> String {
    stub_log("string", "StringMid", args)
}
pub fn string_replace(args: &[&str]) -> String {
    stub_log("string", "StringReplace", args)
}
pub fn string_split(args: &[&str]) -> String {
    stub_log("string", "StringSplit", args)
}
pub fn string_trim_left(args: &[&str]) -> String {
    stub_log("string", "StringTrimLeft", args)
}
pub fn sub_str(args: &[&str]) -> String {
    stub_log("string", "SubStr", args)
}
pub fn trim(args: &[&str]) -> String {
    stub_log("string", "Trim", args)
}
pub fn l_trim(args: &[&str]) -> String {
    stub_log("string", "LTrim", args)
}
pub fn r_trim(args: &[&str]) -> String {
    stub_log("string", "RTrim", args)
}
pub fn regex_match(args: &[&str]) -> String {
    stub_log("string", "RegExMatch", args)
}
pub fn regex_replace(args: &[&str]) -> String {
    stub_log("string", "RegExReplace", args)
}
pub fn string_upper(args: &[&str]) -> String {
    stub_log("string", "StringUpper", args)
}
pub fn string_right(args: &[&str]) -> String {
    stub_log("string", "StringRight", args)
}
pub fn string_trim_right(args: &[&str]) -> String {
    stub_log("string", "StringTrimRight", args)
}

pub const METHODS: &[ModuleMethod] = &[
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
];
