use tree_sitter::{ffi::TSLanguage, Language};

unsafe extern "C" {
    fn tree_sitter_autohotkey() -> *const TSLanguage;
}

pub fn language() -> Language {
    unsafe { Language::from_raw(tree_sitter_autohotkey()) }
}
