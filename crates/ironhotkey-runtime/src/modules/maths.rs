use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use super::ModuleMethod;

fn format_number(value: f64) -> String {
    if !value.is_finite() {
        return String::new();
    }
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn parse_numeric_text(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (sign, digits) = match trimmed.as_bytes().first().copied() {
        Some(b'+') => (1.0, &trimmed[1..]),
        Some(b'-') => (-1.0, &trimmed[1..]),
        _ => (1.0, trimmed),
    };

    if digits.starts_with("0x") || digits.starts_with("0X") {
        if digits.len() <= 2 {
            return None;
        }
        return i64::from_str_radix(&digits[2..], 16)
            .ok()
            .map(|number| sign * number as f64);
    }

    let is_scientific = digits.contains('e') || digits.contains('E');
    if is_scientific && !digits.contains('.') {
        return None;
    }

    trimmed.parse::<f64>().ok()
}

fn parse_number(value: Option<&str>) -> f64 {
    value.and_then(parse_numeric_text).unwrap_or(0.0)
}

fn parse_optional_i32(value: Option<&str>) -> Option<i32> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<i32>().ok())
}

fn apply_width(value: String, width: Option<usize>, align_left: bool, zero_pad: bool) -> String {
    let Some(width) = width else {
        return value;
    };
    if value.len() >= width {
        return value;
    }

    let pad_len = width - value.len();
    let pad_char = if zero_pad { '0' } else { ' ' };
    let padding = std::iter::repeat_n(pad_char, pad_len).collect::<String>();

    if align_left {
        format!("{value}{padding}")
    } else if zero_pad && (value.starts_with('-') || value.starts_with('+')) {
        let sign = &value[0..1];
        let rest = &value[1..];
        format!("{sign}{padding}{rest}")
    } else {
        format!("{padding}{value}")
    }
}

fn apply_format_spec(value: &str, spec: &str) -> String {
    let normalized = spec.trim();
    if normalized.is_empty() {
        return value.to_string();
    }

    if normalized.eq_ignore_ascii_case("u") && normalized.len() == 1 {
        return value.to_uppercase();
    }
    if normalized.eq_ignore_ascii_case("l") && normalized.len() == 1 {
        return value.to_lowercase();
    }
    if normalized.eq_ignore_ascii_case("t") && normalized.len() == 1 {
        let mut chars = value.chars();
        if let Some(first) = chars.next() {
            return first
                .to_uppercase()
                .chain(chars.flat_map(|ch| ch.to_lowercase()))
                .collect();
        }
        return String::new();
    }

    let mut chars = normalized.chars();
    let maybe_ty = chars.next_back();
    let (fmt_part, ty) = match maybe_ty {
        Some(kind) if "diuoxXfFeEc".contains(kind) => (
            &normalized[..normalized.len() - kind.len_utf8()],
            Some(kind),
        ),
        _ => (normalized, None),
    };

    let align_left = fmt_part.contains('-');
    let zero_pad = fmt_part.starts_with('0') && !align_left;

    let (width_part, precision) = if let Some((lhs, rhs)) = fmt_part.split_once('.') {
        let precision = rhs.parse::<usize>().ok();
        (lhs, precision)
    } else {
        (fmt_part, None)
    };

    let width_digits = width_part
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>();
    let width = width_digits.parse::<usize>().ok();

    let rendered = match ty {
        Some('d') | Some('i') => parse_numeric_text(value)
            .map(|number| (number.trunc() as i64).to_string())
            .unwrap_or_else(|| value.to_string()),
        Some('u') => parse_numeric_text(value)
            .map(|number| (number.trunc() as u64).to_string())
            .unwrap_or_else(|| value.to_string()),
        Some('x') => parse_numeric_text(value)
            .map(|number| format!("{:x}", number.trunc() as i64 as u64))
            .unwrap_or_else(|| value.to_string()),
        Some('X') => parse_numeric_text(value)
            .map(|number| format!("{:X}", number.trunc() as i64 as u64))
            .unwrap_or_else(|| value.to_string()),
        Some('o') => parse_numeric_text(value)
            .map(|number| format!("{:o}", number.trunc() as i64 as u64))
            .unwrap_or_else(|| value.to_string()),
        Some('c') => parse_numeric_text(value)
            .and_then(|number| char::from_u32(number.trunc() as u32))
            .map(|ch| ch.to_string())
            .unwrap_or_else(|| value.to_string()),
        Some('f') | Some('F') => parse_numeric_text(value)
            .map(|number| format!("{:.*}", precision.unwrap_or(6), number))
            .unwrap_or_else(|| value.to_string()),
        Some('e') => parse_numeric_text(value)
            .map(|number| format!("{:.*e}", precision.unwrap_or(6), number))
            .unwrap_or_else(|| value.to_string()),
        Some('E') => parse_numeric_text(value)
            .map(|number| format!("{:.*E}", precision.unwrap_or(6), number))
            .unwrap_or_else(|| value.to_string()),
        _ => value.to_string(),
    };

    apply_width(rendered, width, align_left, zero_pad)
}

fn format_template(format_str: &str, values: &[&str]) -> String {
    let mut output = String::new();
    let chars = format_str.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut auto_index = 0usize;

    while index < chars.len() {
        let ch = chars[index];

        if ch == '{' {
            if index + 1 < chars.len() && chars[index + 1] == '{' {
                output.push('{');
                index += 2;
                continue;
            }

            let mut end = index + 1;
            while end < chars.len() && chars[end] != '}' {
                end += 1;
            }
            if end >= chars.len() {
                output.push('{');
                index += 1;
                continue;
            }

            let inner = chars[index + 1..end].iter().collect::<String>();
            let (raw_slot, raw_spec) = inner
                .split_once(':')
                .map(|(a, b)| (a.trim(), b.trim()))
                .unwrap_or((inner.trim(), ""));

            let slot = if raw_slot.is_empty() {
                auto_index += 1;
                auto_index
            } else {
                raw_slot.parse::<usize>().ok().unwrap_or(1)
            };

            let replacement = values.get(slot.saturating_sub(1)).copied().unwrap_or("");
            output.push_str(&apply_format_spec(replacement, raw_spec));
            index = end + 1;
            continue;
        }

        if ch == '}' {
            if index + 1 < chars.len() && chars[index + 1] == '}' {
                output.push('}');
                index += 2;
            } else {
                output.push('}');
                index += 1;
            }
            continue;
        }

        output.push(ch);
        index += 1;
    }

    output
}

#[derive(Clone, Copy)]
struct DateTimeParts {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };

    (year as i32, m as u32, d as u32)
}

fn current_utc_parts() -> DateTimeParts {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = (now / 86_400) as i64;
    let seconds_of_day = (now % 86_400) as u32;
    let (year, month, day) = civil_from_days(days);

    DateTimeParts {
        year,
        month,
        day,
        hour: seconds_of_day / 3600,
        minute: (seconds_of_day % 3600) / 60,
        second: seconds_of_day % 60,
    }
}

fn parse_timestamp(timestamp: Option<&str>) -> Option<DateTimeParts> {
    let Some(raw) = timestamp.map(str::trim).filter(|value| !value.is_empty()) else {
        return Some(current_utc_parts());
    };

    let digits = raw
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.len() < 4 {
        return None;
    }

    let read = |start: usize, len: usize| -> Option<u32> {
        digits
            .get(start..start + len)
            .and_then(|part| part.parse::<u32>().ok())
    };

    let year = read(0, 4)? as i32;
    let month = if digits.len() >= 6 { read(4, 2)? } else { 1 };
    let day = if digits.len() >= 8 { read(6, 2)? } else { 1 };
    let hour = if digits.len() >= 10 { read(8, 2)? } else { 0 };
    let minute = if digits.len() >= 12 { read(10, 2)? } else { 0 };
    let second = if digits.len() >= 14 { read(12, 2)? } else { 0 };

    if !(1..=12).contains(&month)
        || day < 1
        || day > days_in_month(year, month)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }

    Some(DateTimeParts {
        year,
        month,
        day,
        hour,
        minute,
        second,
    })
}

fn format_datetime(parts: DateTimeParts, pattern: Option<&str>) -> String {
    let template = pattern
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("HH:mm:ss yyyy-MM-dd");

    let hour_12 = {
        let h = parts.hour % 12;
        if h == 0 {
            12
        } else {
            h
        }
    };
    let am_pm = if parts.hour < 12 { "AM" } else { "PM" };

    let tokens = [
        ("yyyy", format!("{:04}", parts.year)),
        ("yy", format!("{:02}", (parts.year % 100).abs())),
        ("MM", format!("{:02}", parts.month)),
        ("M", parts.month.to_string()),
        ("dd", format!("{:02}", parts.day)),
        ("d", parts.day.to_string()),
        ("HH", format!("{:02}", parts.hour)),
        ("H", parts.hour.to_string()),
        ("hh", format!("{:02}", hour_12)),
        ("h", hour_12.to_string()),
        ("mm", format!("{:02}", parts.minute)),
        ("m", parts.minute.to_string()),
        ("ss", format!("{:02}", parts.second)),
        ("s", parts.second.to_string()),
        ("tt", am_pm.to_string()),
    ];

    let mut output = String::new();
    let mut index = 0usize;
    while index < template.len() {
        let slice = &template[index..];
        if let Some((token, replacement)) =
            tokens.iter().find(|(token, _)| slice.starts_with(token))
        {
            output.push_str(replacement);
            index += token.len();
        } else if let Some(ch) = slice.chars().next() {
            output.push(ch);
            index += ch.len_utf8();
        } else {
            break;
        }
    }

    output
}

#[derive(Clone, Copy)]
enum NumKind {
    Int,
    UInt,
    Int64,
    Short,
    UShort,
    Char,
    UChar,
    Float,
    Double,
    Ptr,
    UPtr,
}

impl NumKind {
    fn parse(value: Option<&str>) -> Option<Self> {
        let kind = value.unwrap_or("UPtr").trim().to_ascii_lowercase();
        match kind.as_str() {
            "int" => Some(Self::Int),
            "uint" => Some(Self::UInt),
            "int64" => Some(Self::Int64),
            "short" => Some(Self::Short),
            "ushort" => Some(Self::UShort),
            "char" => Some(Self::Char),
            "uchar" => Some(Self::UChar),
            "float" => Some(Self::Float),
            "double" => Some(Self::Double),
            "ptr" => Some(Self::Ptr),
            "uptr" => Some(Self::UPtr),
            _ => None,
        }
    }

    fn size(self) -> usize {
        match self {
            Self::Int | Self::UInt | Self::Float => 4,
            Self::Short | Self::UShort => 2,
            Self::Char | Self::UChar => 1,
            Self::Int64 | Self::Double | Self::Ptr | Self::UPtr => 8,
        }
    }
}

#[derive(Default)]
struct NumMemory {
    slots: HashMap<String, Vec<u8>>,
}

fn num_memory() -> &'static Mutex<NumMemory> {
    static MEMORY: OnceLock<Mutex<NumMemory>> = OnceLock::new();
    MEMORY.get_or_init(|| Mutex::new(NumMemory::default()))
}

fn memory_key(var_or_address: &str) -> String {
    let trimmed = var_or_address.trim();
    if let Some(value) = parse_numeric_text(trimmed) {
        if value.is_finite() && value >= 0.0 {
            return format!("addr:{}", value as usize);
        }
    }
    format!("var:{trimmed}")
}

fn write_kind_bytes(kind: NumKind, number: f64) -> Vec<u8> {
    match kind {
        NumKind::Int => (number.trunc() as i32).to_le_bytes().to_vec(),
        NumKind::UInt => (number.trunc() as u32).to_le_bytes().to_vec(),
        NumKind::Int64 => (number.trunc() as i64).to_le_bytes().to_vec(),
        NumKind::Short => (number.trunc() as i16).to_le_bytes().to_vec(),
        NumKind::UShort => (number.trunc() as u16).to_le_bytes().to_vec(),
        NumKind::Char => (number.trunc() as i8).to_le_bytes().to_vec(),
        NumKind::UChar => (number.trunc() as u8).to_le_bytes().to_vec(),
        NumKind::Float => (number as f32).to_le_bytes().to_vec(),
        NumKind::Double => number.to_le_bytes().to_vec(),
        NumKind::Ptr | NumKind::UPtr => (number.trunc() as u64).to_le_bytes().to_vec(),
    }
}

fn read_kind_value(kind: NumKind, bytes: &[u8]) -> Option<String> {
    match kind {
        NumKind::Int => bytes
            .get(..4)
            .and_then(|slice| <[u8; 4]>::try_from(slice).ok())
            .map(i32::from_le_bytes)
            .map(|number| number.to_string()),
        NumKind::UInt => bytes
            .get(..4)
            .and_then(|slice| <[u8; 4]>::try_from(slice).ok())
            .map(u32::from_le_bytes)
            .map(|number| number.to_string()),
        NumKind::Int64 => bytes
            .get(..8)
            .and_then(|slice| <[u8; 8]>::try_from(slice).ok())
            .map(i64::from_le_bytes)
            .map(|number| number.to_string()),
        NumKind::Short => bytes
            .get(..2)
            .and_then(|slice| <[u8; 2]>::try_from(slice).ok())
            .map(i16::from_le_bytes)
            .map(|number| number.to_string()),
        NumKind::UShort => bytes
            .get(..2)
            .and_then(|slice| <[u8; 2]>::try_from(slice).ok())
            .map(u16::from_le_bytes)
            .map(|number| number.to_string()),
        NumKind::Char => bytes
            .first()
            .map(|number| i8::from_le_bytes([*number]).to_string()),
        NumKind::UChar => bytes.first().map(|number| number.to_string()),
        NumKind::Float => bytes
            .get(..4)
            .and_then(|slice| <[u8; 4]>::try_from(slice).ok())
            .map(f32::from_le_bytes)
            .map(|number| format_number(number as f64)),
        NumKind::Double => bytes
            .get(..8)
            .and_then(|slice| <[u8; 8]>::try_from(slice).ok())
            .map(f64::from_le_bytes)
            .map(format_number),
        NumKind::Ptr | NumKind::UPtr => bytes
            .get(..8)
            .and_then(|slice| <[u8; 8]>::try_from(slice).ok())
            .map(u64::from_le_bytes)
            .map(|number| number.to_string()),
    }
}

fn random_state() -> &'static Mutex<u64> {
    static STATE: OnceLock<Mutex<u64>> = OnceLock::new();
    STATE.get_or_init(|| {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
            ^ 0x9E37_79B9_7F4A_7C15;
        Mutex::new(seed.max(1))
    })
}

fn next_random_u64() -> u64 {
    let mut guard = random_state().lock().expect("random mutex poisoned");
    *guard = guard
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *guard
}

pub fn abs(number: f64) -> String {
    format_number(number.abs())
}

pub fn a_cos(number: f64) -> String {
    if !(-1.0..=1.0).contains(&number) {
        String::new()
    } else {
        format_number(number.acos())
    }
}

pub fn a_sin(number: f64) -> String {
    if !(-1.0..=1.0).contains(&number) {
        String::new()
    } else {
        format_number(number.asin())
    }
}

pub fn asc(text: &str) -> String {
    text.chars()
        .next()
        .map(|value| (value as u32).to_string())
        .unwrap_or_default()
}

pub fn a_tan(number: f64) -> String {
    format_number(number.atan())
}

pub fn ceil(number: f64) -> String {
    format_number(number.ceil())
}

pub fn chr(code: u32) -> String {
    char::from_u32(code)
        .map(|value| value.to_string())
        .unwrap_or_default()
}

pub fn cos(number: f64) -> String {
    format_number(number.cos())
}

pub fn exp(number: f64) -> String {
    format_number(number.exp())
}

pub fn floor(number: f64) -> String {
    format_number(number.floor())
}

pub fn format(format_str: &str, values: &[&str]) -> String {
    format_template(format_str, values)
}

pub fn format_time(timestamp: Option<&str>, pattern: Option<&str>) -> String {
    let Some(parts) = parse_timestamp(timestamp) else {
        return String::new();
    };
    format_datetime(parts, pattern)
}

pub fn ln(number: f64) -> String {
    if number < 0.0 {
        String::new()
    } else {
        format_number(number.ln())
    }
}

pub fn log(number: f64) -> String {
    if number < 0.0 {
        String::new()
    } else {
        format_number(number.log10())
    }
}

pub fn math(expression: &str) -> String {
    #[derive(Clone, Copy)]
    enum Token {
        Number(f64),
        Op(char),
        LParen,
        RParen,
    }

    fn precedence(op: char) -> i32 {
        match op {
            '^' => 4,
            '*' | '/' | '%' => 3,
            '+' | '-' => 2,
            _ => 0,
        }
    }

    fn right_associative(op: char) -> bool {
        op == '^'
    }

    fn apply_op(values: &mut Vec<f64>, op: char) -> Option<()> {
        let rhs = values.pop()?;
        let lhs = values.pop()?;
        let result = match op {
            '+' => lhs + rhs,
            '-' => lhs - rhs,
            '*' => lhs * rhs,
            '/' => {
                if rhs == 0.0 {
                    return None;
                }
                lhs / rhs
            }
            '%' => {
                if rhs == 0.0 {
                    return None;
                }
                lhs % rhs
            }
            '^' => lhs.powf(rhs),
            _ => return None,
        };
        values.push(result);
        Some(())
    }

    fn tokenize(input: &str) -> Option<Vec<Token>> {
        let mut tokens = Vec::new();
        let chars = input.chars().collect::<Vec<_>>();
        let mut index = 0usize;
        let mut expect_unary = true;

        while index < chars.len() {
            let ch = chars[index];
            if ch.is_ascii_whitespace() {
                index += 1;
                continue;
            }

            if ch == '(' {
                tokens.push(Token::LParen);
                expect_unary = true;
                index += 1;
                continue;
            }
            if ch == ')' {
                tokens.push(Token::RParen);
                expect_unary = false;
                index += 1;
                continue;
            }

            if "+-*/%^".contains(ch) {
                if (ch == '+' || ch == '-') && expect_unary {
                    let start = index;
                    index += 1;
                    while index < chars.len() && chars[index].is_ascii_whitespace() {
                        index += 1;
                    }

                    if index < chars.len() {
                        if chars[index] == '0'
                            && index + 1 < chars.len()
                            && (chars[index + 1] == 'x' || chars[index + 1] == 'X')
                        {
                            index += 2;
                            while index < chars.len() && chars[index].is_ascii_hexdigit() {
                                index += 1;
                            }
                        } else {
                            while index < chars.len() && chars[index].is_ascii_digit() {
                                index += 1;
                            }
                            if index < chars.len() && chars[index] == '.' {
                                index += 1;
                                while index < chars.len() && chars[index].is_ascii_digit() {
                                    index += 1;
                                }
                            }
                            if index < chars.len() && (chars[index] == 'e' || chars[index] == 'E') {
                                index += 1;
                                if index < chars.len()
                                    && (chars[index] == '+' || chars[index] == '-')
                                {
                                    index += 1;
                                }
                                while index < chars.len() && chars[index].is_ascii_digit() {
                                    index += 1;
                                }
                            }
                        }
                    }

                    let number_text = chars[start..index].iter().collect::<String>();
                    let value = parse_numeric_text(&number_text)?;
                    tokens.push(Token::Number(value));
                    expect_unary = false;
                    continue;
                }

                tokens.push(Token::Op(ch));
                expect_unary = true;
                index += 1;
                continue;
            }

            let start = index;
            if ch == '0'
                && index + 1 < chars.len()
                && (chars[index + 1] == 'x' || chars[index + 1] == 'X')
            {
                index += 2;
                while index < chars.len() && chars[index].is_ascii_hexdigit() {
                    index += 1;
                }
            } else {
                while index < chars.len() && chars[index].is_ascii_digit() {
                    index += 1;
                }
                if index < chars.len() && chars[index] == '.' {
                    index += 1;
                    while index < chars.len() && chars[index].is_ascii_digit() {
                        index += 1;
                    }
                }
                if index < chars.len() && (chars[index] == 'e' || chars[index] == 'E') {
                    index += 1;
                    if index < chars.len() && (chars[index] == '+' || chars[index] == '-') {
                        index += 1;
                    }
                    while index < chars.len() && chars[index].is_ascii_digit() {
                        index += 1;
                    }
                }
            }

            let number_text = chars[start..index].iter().collect::<String>();
            let value = parse_numeric_text(&number_text)?;
            tokens.push(Token::Number(value));
            expect_unary = false;
        }

        Some(tokens)
    }

    let tokens = match tokenize(expression) {
        Some(tokens) => tokens,
        None => return String::new(),
    };

    let mut values = Vec::<f64>::new();
    let mut ops = Vec::<char>::new();

    for token in tokens {
        match token {
            Token::Number(number) => values.push(number),
            Token::LParen => ops.push('('),
            Token::RParen => {
                while let Some(op) = ops.pop() {
                    if op == '(' {
                        break;
                    }
                    if apply_op(&mut values, op).is_none() {
                        return String::new();
                    }
                }
            }
            Token::Op(op) => {
                while let Some(&top) = ops.last() {
                    if top == '(' {
                        break;
                    }
                    let should_apply = precedence(top) > precedence(op)
                        || (precedence(top) == precedence(op) && !right_associative(op));
                    if !should_apply {
                        break;
                    }
                    let top = ops.pop().unwrap_or('+');
                    if apply_op(&mut values, top).is_none() {
                        return String::new();
                    }
                }
                ops.push(op);
            }
        }
    }

    while let Some(op) = ops.pop() {
        if op == '(' {
            return String::new();
        }
        if apply_op(&mut values, op).is_none() {
            return String::new();
        }
    }

    if values.len() == 1 {
        format_number(values[0])
    } else {
        String::new()
    }
}

pub fn max(numbers: &[f64]) -> String {
    numbers
        .iter()
        .copied()
        .reduce(f64::max)
        .map(format_number)
        .unwrap_or_default()
}

pub fn min(numbers: &[f64]) -> String {
    numbers
        .iter()
        .copied()
        .reduce(f64::min)
        .map(format_number)
        .unwrap_or_default()
}

pub fn mod_fn(dividend: f64, divisor: f64) -> String {
    if divisor == 0.0 {
        String::new()
    } else {
        format_number(dividend % divisor)
    }
}

pub fn num_get(var_or_address: &str, offset: Option<i32>, kind: Option<&str>) -> String {
    let kind = match NumKind::parse(kind) {
        Some(kind) => kind,
        None => return String::new(),
    };

    let offset = offset.unwrap_or(0);
    if offset < 0 {
        return String::new();
    }
    let offset = offset as usize;

    let key = memory_key(var_or_address);
    let guard = num_memory().lock().expect("num memory mutex poisoned");
    let Some(buffer) = guard.slots.get(&key) else {
        return String::new();
    };

    let end = offset + kind.size();
    if end > buffer.len() {
        return String::new();
    }

    read_kind_value(kind, &buffer[offset..end]).unwrap_or_default()
}

pub fn num_put(
    number: f64,
    var_or_address: &str,
    offset: Option<i32>,
    kind: Option<&str>,
) -> String {
    let kind = match NumKind::parse(kind) {
        Some(kind) => kind,
        None => return String::new(),
    };

    let offset = offset.unwrap_or(0);
    if offset < 0 {
        return String::new();
    }
    let offset = offset as usize;

    let bytes = write_kind_bytes(kind, number);
    let size = kind.size();
    let key = memory_key(var_or_address);

    let mut guard = num_memory().lock().expect("num memory mutex poisoned");
    let buffer = guard.slots.entry(key.clone()).or_default();
    let end = offset + size;
    if buffer.len() < end {
        buffer.resize(end, 0);
    }
    buffer[offset..end].copy_from_slice(&bytes);

    if let Some(address) = key.strip_prefix("addr:") {
        let base = address.parse::<usize>().ok().unwrap_or_default();
        return (base + end).to_string();
    }

    end.to_string()
}

pub fn ord(text: &str) -> String {
    asc(text)
}

pub fn random(min: Option<f64>, max: Option<f64>) -> String {
    let (lo, hi) = match (min, max) {
        (None, None) => (0.0, 2_147_483_647.0),
        (Some(min), None) => (0.0, min),
        (Some(min), Some(max)) => (min, max),
        (None, Some(max)) => (0.0, max),
    };

    let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };

    if lo.fract() != 0.0 || hi.fract() != 0.0 {
        let unit = (next_random_u64() as f64) / (u64::MAX as f64);
        return format_number(lo + (hi - lo) * unit);
    }

    let lo_i = lo as i64;
    let hi_i = hi as i64;
    if hi_i < lo_i {
        return String::new();
    }

    let span = (hi_i - lo_i + 1) as u64;
    let value = lo_i + (next_random_u64() % span) as i64;
    value.to_string()
}

pub fn round(number: f64, digits: Option<i32>) -> String {
    match digits.unwrap_or(0) {
        0 => format_number(number.round()),
        digits if digits > 0 => {
            let precision = 10f64.powi(digits);
            format!(
                "{:.1$}",
                (number * precision).round() / precision,
                digits as usize
            )
        }
        digits => {
            let precision = 10f64.powi(-digits);
            format_number((number / precision).round() * precision)
        }
    }
}

pub fn sin(number: f64) -> String {
    format_number(number.sin())
}

pub fn sqrt(number: f64) -> String {
    if number < 0.0 {
        String::new()
    } else {
        format_number(number.sqrt())
    }
}

pub fn tan(number: f64) -> String {
    format_number(number.tan())
}

fn abs_compat(args: &[&str]) -> String {
    abs(parse_number(args.first().copied()))
}

fn a_cos_compat(args: &[&str]) -> String {
    a_cos(parse_number(args.first().copied()))
}

fn a_sin_compat(args: &[&str]) -> String {
    a_sin(parse_number(args.first().copied()))
}

fn asc_compat(args: &[&str]) -> String {
    asc(args.first().copied().unwrap_or_default())
}

fn a_tan_compat(args: &[&str]) -> String {
    a_tan(parse_number(args.first().copied()))
}

fn ceil_compat(args: &[&str]) -> String {
    ceil(parse_number(args.first().copied()))
}

fn chr_compat(args: &[&str]) -> String {
    chr(parse_number(args.first().copied()) as u32)
}

fn cos_compat(args: &[&str]) -> String {
    cos(parse_number(args.first().copied()))
}

fn exp_compat(args: &[&str]) -> String {
    exp(parse_number(args.first().copied()))
}

fn floor_compat(args: &[&str]) -> String {
    floor(parse_number(args.first().copied()))
}

fn format_compat(args: &[&str]) -> String {
    if let Some((format_str, values)) = args.split_first() {
        format(format_str, values)
    } else {
        format("", &[])
    }
}

fn format_time_compat(args: &[&str]) -> String {
    format_time(args.first().copied(), args.get(1).copied())
}

fn ln_compat(args: &[&str]) -> String {
    ln(parse_number(args.first().copied()))
}

fn log_compat(args: &[&str]) -> String {
    log(parse_number(args.first().copied()))
}

fn math_compat(args: &[&str]) -> String {
    math(args.first().copied().unwrap_or_default())
}

fn max_compat(args: &[&str]) -> String {
    let values = args
        .iter()
        .map(|value| parse_number(Some(*value)))
        .collect::<Vec<_>>();
    max(&values)
}

fn min_compat(args: &[&str]) -> String {
    let values = args
        .iter()
        .map(|value| parse_number(Some(*value)))
        .collect::<Vec<_>>();
    min(&values)
}

fn mod_compat(args: &[&str]) -> String {
    mod_fn(
        parse_number(args.first().copied()),
        parse_number(args.get(1).copied()),
    )
}

fn num_get_compat(args: &[&str]) -> String {
    num_get(
        args.first().copied().unwrap_or_default(),
        parse_optional_i32(args.get(1).copied()),
        args.get(2).copied(),
    )
}

fn num_put_compat(args: &[&str]) -> String {
    num_put(
        parse_number(args.first().copied()),
        args.get(1).copied().unwrap_or_default(),
        parse_optional_i32(args.get(2).copied()),
        args.get(3).copied(),
    )
}

fn ord_compat(args: &[&str]) -> String {
    ord(args.first().copied().unwrap_or_default())
}

fn random_compat(args: &[&str]) -> String {
    random(
        args.first().copied().and_then(parse_numeric_text),
        args.get(1).copied().and_then(parse_numeric_text),
    )
}

fn round_compat(args: &[&str]) -> String {
    round(
        parse_number(args.first().copied()),
        parse_optional_i32(args.get(1).copied()),
    )
}

fn sin_compat(args: &[&str]) -> String {
    sin(parse_number(args.first().copied()))
}

fn sqrt_compat(args: &[&str]) -> String {
    sqrt(parse_number(args.first().copied()))
}

fn tan_compat(args: &[&str]) -> String {
    tan(parse_number(args.first().copied()))
}

pub const METHODS: &[ModuleMethod] = &[
    ("Abs", abs_compat),
    ("ACos", a_cos_compat),
    ("ASin", a_sin_compat),
    ("Asc", asc_compat),
    ("ATan", a_tan_compat),
    ("Ceil", ceil_compat),
    ("Chr", chr_compat),
    ("Cos", cos_compat),
    ("Exp", exp_compat),
    ("Floor", floor_compat),
    ("Format", format_compat),
    ("FormatTime", format_time_compat),
    ("Ln", ln_compat),
    ("Log", log_compat),
    ("Math", math_compat),
    ("Max", max_compat),
    ("Min", min_compat),
    ("Mod", mod_compat),
    ("NumGet", num_get_compat),
    ("NumPut", num_put_compat),
    ("Ord", ord_compat),
    ("Random", random_compat),
    ("Round", round_compat),
    ("Sin", sin_compat),
    ("Sqrt", sqrt_compat),
    ("Tan", tan_compat),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_value_is_numeric() {
        assert_eq!(abs(-1.2), "1.2");
        assert_eq!(abs(-4.0), "4");
    }

    #[test]
    fn rounding_matches_reference_examples() {
        assert_eq!(round(3.14, None), "3");
        assert_eq!(round(3.14, Some(1)), "3.1");
        assert_eq!(round(345.0, Some(-1)), "350");
        assert_eq!(round(345.0, Some(-2)), "300");
    }

    #[test]
    fn domain_errors_yield_blank_strings() {
        assert_eq!(sqrt(-1.0), "");
        assert_eq!(log(-1.0), "");
        assert_eq!(ln(-1.0), "");
        assert_eq!(a_sin(2.0), "");
        assert_eq!(a_cos(-2.0), "");
        assert_eq!(mod_fn(1.0, 0.0), "");
    }

    #[test]
    fn variadic_math_functions_work() {
        assert_eq!(max(&[2.11, -2.0, 0.0]), "2.11");
        assert_eq!(min(&[2.11, -2.0, 0.0]), "-2");
        assert_eq!(max(&[]), "");
    }

    #[test]
    fn character_helpers_round_trip() {
        assert_eq!(asc("A"), "65");
        assert_eq!(ord("A"), "65");
        assert_eq!(chr(65), "A");
        assert_eq!(chr(u32::MAX), "");
    }

    #[test]
    fn trig_and_exponents_work() {
        assert_eq!(sin(0.0), "0");
        assert_eq!(cos(0.0), "1");
        assert_eq!(tan(0.0), "0");
        assert_eq!(exp(0.0), "1");
    }

    #[test]
    fn format_and_format_time_are_implemented() {
        assert_eq!(format("{:010}", &["123"]), "0000000123");
        assert_eq!(format("{2}-{1}", &["a", "b"]), "b-a");
        assert_eq!(
            format_time(Some("20240317010203"), Some("yyyy-MM-dd HH:mm:ss")),
            "2024-03-17 01:02:03"
        );
    }

    #[test]
    fn math_and_random_are_implemented() {
        assert_eq!(math("1+2*3"), "7");
        assert_eq!(math("1.0e4+-2.1E-4"), "9999.99979");
        let value = random(Some(1.0), Some(3.0));
        let parsed = value.parse::<i64>().ok().unwrap_or_default();
        assert!((1..=3).contains(&parsed));
    }

    #[test]
    fn num_get_and_num_put_are_implemented() {
        assert_eq!(num_put(65.0, "0x1000", Some(0), Some("UChar")), "4097");
        assert_eq!(num_get("0x1000", Some(0), Some("UChar")), "65");

        assert_eq!(num_put(1234.0, "buffer", Some(4), Some("Int")), "8");
        assert_eq!(num_get("buffer", Some(4), Some("Int")), "1234");
    }

    #[test]
    fn compatibility_bridge_still_works() {
        assert_eq!(round_compat(&["3.14", "1"]), "3.1");
        assert_eq!(max_compat(&["1", "4", "2"]), "4");
        assert_eq!(chr_compat(&["65"]), "A");
        assert_eq!(format_compat(&["{:03}", "7"]), "007");
    }
}
