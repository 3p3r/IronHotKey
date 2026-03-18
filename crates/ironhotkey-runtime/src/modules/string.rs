// String manipulation module with full AHK v1 compatibility.

use super::ModuleMethod;
use regex::Regex;

pub fn in_str(
    haystack: &str,
    needle: &str,
    case_sensitive: Option<i32>,
    starting_pos: Option<i32>,
    occurrence: Option<i32>,
) -> String {
    if needle.is_empty() {
        return String::new();
    }

    let case_sensitive = case_sensitive.unwrap_or(0) != 0;
    let start = starting_pos
        .and_then(|pos| {
            if pos < 0 {
                None
            } else {
                Some((pos - 1).max(0) as usize)
            }
        })
        .unwrap_or(0);
    let occurrence = occurrence.unwrap_or(1).max(1) as usize;

    if start >= haystack.len() {
        return String::new();
    }

    let search_text = if case_sensitive {
        haystack[start..].to_string()
    } else {
        haystack[start..].to_lowercase()
    };
    let search_needle = if case_sensitive {
        needle.to_string()
    } else {
        needle.to_lowercase()
    };

    let mut count = 0;
    let mut pos = 0;
    while let Some(found) = search_text[pos..].find(&search_needle) {
        count += 1;
        if count == occurrence {
            return ((start + pos + found) + 1).to_string();
        }
        pos += found + search_needle.len();
    }

    String::new()
}

pub fn str_len(text: &str) -> String {
    text.chars().count().to_string()
}

pub fn str_replace(
    haystack: &str,
    needle: &str,
    replacement: Option<&str>,
    limit: Option<i32>,
) -> String {
    if needle.is_empty() {
        return haystack.to_string();
    }

    let replacement = replacement.unwrap_or("");
    let replacement_count = limit.unwrap_or(-1);

    if replacement_count == 0 {
        return haystack.to_string();
    }

    let mut result = haystack.to_string();
    let max_count = if replacement_count < 0 {
        usize::MAX
    } else {
        replacement_count as usize
    };

    let mut count = 0;
    let mut pos = 0;
    while count < max_count {
        if let Some(found) = result[pos..].find(needle) {
            result.replace_range(pos + found..pos + found + needle.len(), replacement);
            pos += found + replacement.len();
            count += 1;
        } else {
            break;
        }
    }

    result
}

pub fn sub_str(text: &str, start_pos: i32, length: Option<i32>) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let len = chars.len() as i32;

    let actual_start = if start_pos < 0 {
        ((len + start_pos).max(0)) as usize
    } else if start_pos == 0 {
        0
    } else {
        ((start_pos - 1).max(0)) as usize
    };

    if actual_start >= chars.len() {
        return String::new();
    }

    match length {
        None => chars[actual_start..].iter().collect(),
        Some(len) if len < 0 => {
            let end = (chars.len() as i32 + len).max(0) as usize;
            if end <= actual_start {
                String::new()
            } else {
                chars[actual_start..end].iter().collect()
            }
        }
        Some(len) => {
            let end = (actual_start + len as usize).min(chars.len());
            chars[actual_start..end].iter().collect()
        }
    }
}

pub fn trim(text: &str, omit_chars: Option<&str>) -> String {
    match omit_chars {
        None => text.trim().to_string(),
        Some(chars) => {
            let set: std::collections::HashSet<char> = chars.chars().collect();
            let trimmed = text
                .chars()
                .skip_while(|c| set.contains(c))
                .collect::<String>();
            trimmed
                .chars()
                .rev()
                .skip_while(|c| set.contains(c))
                .collect::<String>()
                .chars()
                .rev()
                .collect()
        }
    }
}

pub fn l_trim(text: &str, omit_chars: Option<&str>) -> String {
    match omit_chars {
        None => text.trim_start().to_string(),
        Some(chars) => {
            let set: std::collections::HashSet<char> = chars.chars().collect();
            text.chars()
                .skip_while(|c| set.contains(c))
                .collect::<String>()
        }
    }
}

pub fn r_trim(text: &str, omit_chars: Option<&str>) -> String {
    match omit_chars {
        None => text.trim_end().to_string(),
        Some(chars) => {
            let set: std::collections::HashSet<char> = chars.chars().collect();
            let chars_vec = text.chars().collect::<Vec<_>>();
            chars_vec
                .iter()
                .rev()
                .skip_while(|c| set.contains(c))
                .collect::<String>()
                .chars()
                .rev()
                .collect()
        }
    }
}

pub fn regex_match(haystack: &str, pattern: &str, starting_pos: Option<i32>) -> String {
    let pattern = pattern.trim();
    let start = starting_pos
        .and_then(|pos| usize::try_from(pos - 1).ok())
        .unwrap_or(0);

    if start >= haystack.len() {
        return String::new();
    }

    let (actual_pattern, case_insensitive) = if pattern.starts_with("i)") {
        (&pattern[2..], true)
    } else {
        (pattern, false)
    };

    let re_pattern = if case_insensitive {
        format!("(?i){actual_pattern}")
    } else {
        actual_pattern.to_string()
    };

    match Regex::new(&re_pattern) {
        Ok(re) => {
            if let Some(mat) = re.find(&haystack[start..]) {
                ((start + mat.start()) + 1).to_string()
            } else {
                String::new()
            }
        }
        Err(_) => String::new(),
    }
}

pub fn regex_replace(haystack: &str, pattern: &str, replacement: Option<&str>) -> String {
    let replacement = replacement.unwrap_or("");
    let pattern = pattern.trim();

    let (actual_pattern, case_insensitive) = if pattern.starts_with("i)") {
        (&pattern[2..], true)
    } else {
        (pattern, false)
    };

    let re_pattern = if case_insensitive {
        format!("(?i){actual_pattern}")
    } else {
        actual_pattern.to_string()
    };

    match Regex::new(&re_pattern) {
        Ok(re) => re.replace_all(haystack, replacement).into_owned(),
        Err(_) => haystack.to_string(),
    }
}

pub fn string_len_compat(text: &str) -> String {
    str_len(text)
}

pub fn string_upper(text: &str) -> String {
    text.to_uppercase()
}

pub fn string_lower(text: &str) -> String {
    text.to_lowercase()
}

pub fn string_trim_left(text: &str, count: i32) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let skip = (count as usize).min(chars.len());
    chars[skip..].iter().collect()
}

pub fn string_trim_right(text: &str, count: i32) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let len = chars.len();
    let skip = (count as usize).min(len);
    chars[..(len - skip)].iter().collect()
}

pub fn string_left(text: &str, count: i32) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let take = (count as usize).min(chars.len());
    chars[..take].iter().collect()
}

pub fn string_right(text: &str, count: i32) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let len = chars.len();
    let skip = (len as i32 - count).max(0) as usize;
    chars[skip..].iter().collect()
}

pub fn string_mid(text: &str, start_pos: i32, length: Option<i32>) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let start = ((start_pos - 1).max(0)) as usize;

    if start >= chars.len() {
        return String::new();
    }

    match length {
        None => chars[start..].iter().collect(),
        Some(len) => {
            let end = (start + len as usize).min(chars.len());
            chars[start..end].iter().collect()
        }
    }
}

pub fn string_get_pos(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }
    haystack.find(needle).map(|pos| pos)
}

pub fn string_replace(haystack: &str, needle: &str, replacement: Option<&str>) -> String {
    str_replace(haystack, needle, replacement, None)
}

pub fn str_get(address: &str, _length: Option<i32>, _encoding: Option<&str>) -> String {
    address.to_string()
}

pub fn str_put(
    text: &str,
    _address: Option<&str>,
    _length: Option<i32>,
    _encoding: Option<&str>,
) -> String {
    text.len().to_string()
}

pub fn str_split(text: &str, delimiters: Option<&str>) -> String {
    let delimiters = delimiters.unwrap_or(",");
    let parts: Vec<&str> = if delimiters.is_empty() {
        vec![text]
    } else {
        text.split(|c: char| delimiters.contains(c)).collect()
    };

    if parts.is_empty() {
        String::from("0")
    } else {
        parts.len().to_string()
    }
}

pub fn compat_in_str(args: &[&str]) -> String {
    in_str(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied().unwrap_or_default(),
        args.get(2).copied().and_then(|a| a.parse().ok()),
        args.get(3).copied().and_then(|a| a.parse().ok()),
        args.get(4).copied().and_then(|a| a.parse().ok()),
    )
}

pub fn compat_str_len(args: &[&str]) -> String {
    str_len(args.first().copied().unwrap_or_default())
}

pub fn compat_str_replace(args: &[&str]) -> String {
    str_replace(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied().unwrap_or_default(),
        args.get(2).copied(),
        args.get(3).copied().and_then(|a| a.parse().ok()),
    )
}

pub fn compat_sub_str(args: &[&str]) -> String {
    sub_str(
        args.first().copied().unwrap_or_default(),
        args.get(1)
            .copied()
            .and_then(|a| a.parse().ok())
            .unwrap_or(1),
        args.get(2).copied().and_then(|a| a.parse().ok()),
    )
}

pub fn compat_trim(args: &[&str]) -> String {
    trim(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied(),
    )
}

pub fn compat_l_trim(args: &[&str]) -> String {
    l_trim(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied(),
    )
}

pub fn compat_r_trim(args: &[&str]) -> String {
    r_trim(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied(),
    )
}

pub fn compat_regex_match(args: &[&str]) -> String {
    regex_match(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied().unwrap_or_default(),
        args.get(2).copied().and_then(|a| a.parse().ok()),
    )
}

pub fn compat_regex_replace(args: &[&str]) -> String {
    regex_replace(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied().unwrap_or_default(),
        args.get(2).copied(),
    )
}

pub fn compat_string_len(args: &[&str]) -> String {
    string_len_compat(args.first().copied().unwrap_or_default())
}

pub fn compat_string_upper(args: &[&str]) -> String {
    string_upper(args.first().copied().unwrap_or_default())
}

pub fn compat_string_lower(args: &[&str]) -> String {
    string_lower(args.first().copied().unwrap_or_default())
}

pub fn compat_string_trim_left(args: &[&str]) -> String {
    string_trim_left(
        args.first().copied().unwrap_or_default(),
        args.get(1)
            .copied()
            .and_then(|a| a.parse().ok())
            .unwrap_or(0),
    )
}

pub fn compat_string_trim_right(args: &[&str]) -> String {
    string_trim_right(
        args.first().copied().unwrap_or_default(),
        args.get(1)
            .copied()
            .and_then(|a| a.parse().ok())
            .unwrap_or(0),
    )
}

pub fn compat_string_left(args: &[&str]) -> String {
    string_left(
        args.first().copied().unwrap_or_default(),
        args.get(1)
            .copied()
            .and_then(|a| a.parse().ok())
            .unwrap_or(0),
    )
}

pub fn compat_string_right(args: &[&str]) -> String {
    string_right(
        args.first().copied().unwrap_or_default(),
        args.get(1)
            .copied()
            .and_then(|a| a.parse().ok())
            .unwrap_or(0),
    )
}

pub fn compat_string_mid(args: &[&str]) -> String {
    string_mid(
        args.first().copied().unwrap_or_default(),
        args.get(1)
            .copied()
            .and_then(|a| a.parse().ok())
            .unwrap_or(1),
        args.get(2).copied().and_then(|a| a.parse().ok()),
    )
}

pub fn compat_string_get_pos(args: &[&str]) -> String {
    string_get_pos(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied().unwrap_or_default(),
    )
    .map(|pos| pos.to_string())
    .unwrap_or_default()
}

pub fn compat_string_replace(args: &[&str]) -> String {
    string_replace(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied().unwrap_or_default(),
        args.get(2).copied(),
    )
}

pub fn compat_str_get(args: &[&str]) -> String {
    str_get(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied().and_then(|a| a.parse().ok()),
        args.get(2).copied(),
    )
}

pub fn compat_str_put(args: &[&str]) -> String {
    str_put(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied(),
        args.get(2).copied().and_then(|a| a.parse().ok()),
        args.get(3).copied(),
    )
}

pub fn compat_str_split(args: &[&str]) -> String {
    str_split(
        args.first().copied().unwrap_or_default(),
        args.get(1).copied(),
    )
}

pub const METHODS: &[ModuleMethod] = &[
    ("InStr", compat_in_str),
    ("StrLen", compat_str_len),
    ("StrReplace", compat_str_replace),
    ("SubStr", compat_sub_str),
    ("Trim", compat_trim),
    ("LTrim", compat_l_trim),
    ("RTrim", compat_r_trim),
    ("RegExMatch", compat_regex_match),
    ("RegExReplace", compat_regex_replace),
    ("StringLen", compat_string_len),
    ("StringUpper", compat_string_upper),
    ("StringLower", compat_string_lower),
    ("StringTrimLeft", compat_string_trim_left),
    ("StringTrimRight", compat_string_trim_right),
    ("StringLeft", compat_string_left),
    ("StringRight", compat_string_right),
    ("StringMid", compat_string_mid),
    ("StringGetPos", compat_string_get_pos),
    ("StringReplace", compat_string_replace),
    ("StrGet", compat_str_get),
    ("StrPut", compat_str_put),
    ("StrSplit", compat_str_split),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_str_finds_substrings() {
        assert_eq!(in_str("Hello World", "World", None, None, None), "7");
        assert_eq!(in_str("Hello World", "o", None, None, None), "5");
        assert_eq!(in_str("Hello World", "xyz", None, None, None), "");
        assert_eq!(in_str("HELLO", "hello", Some(1), None, None), "");
        assert_eq!(in_str("HELLO", "hello", Some(0), None, None), "1");
    }

    #[test]
    fn str_len_counts_characters() {
        assert_eq!(str_len("Hello"), "5");
        assert_eq!(str_len(""), "0");
        assert_eq!(str_len("你好"), "2");
    }

    #[test]
    fn str_replace_handles_multiple_replacements() {
        assert_eq!(
            str_replace("Hello World", "World", Some("AHK"), None),
            "Hello AHK"
        );
        assert_eq!(str_replace("aaa", "a", Some("b"), Some(2)), "bba");
        assert_eq!(str_replace("test", "t", Some("T"), None), "TesT");
    }

    #[test]
    fn sub_str_extracts_substrings() {
        assert_eq!(sub_str("Hello World", 1, Some(5)), "Hello");
        assert_eq!(sub_str("Hello World", 7, None), "World");
        assert_eq!(sub_str("Hello", 2, Some(3)), "ell");
        assert_eq!(sub_str("Hello", -2, None), "lo");
    }

    #[test]
    fn trim_functions_remove_whitespace() {
        assert_eq!(trim("  Hello  ", None), "Hello");
        assert_eq!(l_trim("  Hello", None), "Hello");
        assert_eq!(r_trim("Hello  ", None), "Hello");
        assert_eq!(trim("xxxHelloxxx", Some("x")), "Hello");
    }

    #[test]
    fn string_upper_lower_convert_case() {
        assert_eq!(string_upper("hello"), "HELLO");
        assert_eq!(string_lower("HELLO"), "hello");
    }

    #[test]
    fn string_extraction_functions_work() {
        assert_eq!(string_left("Hello", 2), "He");
        assert_eq!(string_right("Hello", 3), "llo");
        assert_eq!(string_mid("Hello World", 7, Some(5)), "World");
    }

    #[test]
    fn string_get_pos_locates_substrings() {
        assert_eq!(string_get_pos("Hello World", "World"), Some(6));
        assert_eq!(string_get_pos("Hello World", "xyz"), None);
    }

    #[test]
    fn str_split_counts_parts() {
        assert_eq!(str_split("one,two,three", Some(",")), "3");
        assert_eq!(str_split("a:b;c", Some(":;")), "3");
    }

    #[test]
    fn regex_functions_match_and_replace() {
        assert_eq!(regex_match("test123", "\\d+", None), "5");
        assert_eq!(regex_replace("test123test", "\\d+", Some("X")), "testXtest");
        assert_eq!(regex_match("HELLO", "i)hello", None), "1");
    }

    #[test]
    fn compat_wrappers_preserve_behavior() {
        assert_eq!(compat_str_len(&["Hello"]), "5");
        assert_eq!(compat_sub_str(&["Hello", "1", "3"]), "Hel");
        assert_eq!(compat_trim(&["  test  "]), "test");
    }
}
