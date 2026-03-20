use image::{imageops::FilterType, DynamicImage, RgbaImage};
use xcap::Monitor;

use super::ModuleMethod;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ImageSearchOptions<'a> {
    variation: u8,
    trans_color: Option<(u8, u8, u8)>,
    width: Option<i32>,
    height: Option<i32>,
    icon_index: Option<u32>,
    filepath: &'a str,
}
pub fn monitor_get_count() -> String {
    Monitor::all()
        .map(|m| m.len().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

pub fn monitor_get_primary() -> String {
    let Ok(monitors) = Monitor::all() else {
        return "1".to_string();
    };
    monitors
        .iter()
        .enumerate()
        .find_map(|(i, m)| {
            if m.is_primary() {
                Some((i + 1).to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "1".to_string())
}

pub fn monitor_get_name(monitor_num: Option<u32>) -> String {
    let Ok(monitors) = Monitor::all() else {
        return String::new();
    };
    let m = pick_monitor(&monitors, monitor_num);
    m.map(|mon| mon.name().to_string()).unwrap_or_default()
}

pub fn monitor_get(monitor_num: Option<u32>) -> String {
    let Ok(monitors) = Monitor::all() else {
        return String::new();
    };
    let Some(m) = pick_monitor(&monitors, monitor_num) else {
        return String::new();
    };
    format!(
        "{}|{}|{}|{}",
        m.x(),
        m.y(),
        m.x() + m.width() as i32,
        m.y() + m.height() as i32
    )
}

pub fn monitor_get_work_area(monitor_num: Option<u32>) -> String {
    let Ok(monitors) = Monitor::all() else {
        return String::new();
    };
    let Some(m) = pick_monitor(&monitors, monitor_num) else {
        return String::new();
    };

    let mx = m.x();
    let my = m.y();
    let mw = m.width() as i32;
    let mh = m.height() as i32;

    let img = match m.capture_image() {
        Ok(img) => img,
        Err(_) => return format!("{mx}|{my}|{}|{}", mx + mw, my + mh),
    };

    let top = scan_edge_inward(&img, false, false) as i32;
    let bottom = scan_edge_inward(&img, false, true) as i32;
    let left = scan_edge_inward(&img, true, false) as i32;
    let right = scan_edge_inward(&img, true, true) as i32;

    format!(
        "{}|{}|{}|{}",
        mx + left,
        my + top,
        mx + mw - right,
        my + mh - bottom,
    )
}

/// Scans from one edge of the monitor screenshot inward and returns the depth
/// of any taskbar/panel strip found (or 0 if none detected).
///
/// `vertical`: scan columns (left/right edges) rather than rows (top/bottom)
/// `from_far`: scan from the far edge (bottom or right) rather than near (top or left)
fn scan_edge_inward(img: &RgbaImage, vertical: bool, from_far: bool) -> u32 {
    const VARIANCE_THRESHOLD: f32 = 900.0;
    const MIN_DEPTH: u32 = 20;

    let outer = if vertical { img.width() } else { img.height() };
    let max_depth = (outer / 4).min(200);

    let mut depth = 0u32;
    for i in 0..max_depth {
        let idx = if from_far { outer - 1 - i } else { i };
        let v = if vertical {
            strip_variance_col(img, idx)
        } else {
            strip_variance_row(img, idx)
        };
        if v < VARIANCE_THRESHOLD {
            depth = i + 1;
        } else {
            break;
        }
    }

    if depth >= MIN_DEPTH {
        depth
    } else {
        0
    }
}

fn strip_variance_row(img: &RgbaImage, row: u32) -> f32 {
    let w = img.width();
    if w == 0 {
        return 0.0;
    }
    let mut sum = 0u64;
    let mut sum_sq = 0u64;
    for x in 0..w {
        let p = img[(x, row)];
        let luma = (299u64 * p[0] as u64 + 587 * p[1] as u64 + 114 * p[2] as u64) / 1000;
        sum += luma;
        sum_sq += luma * luma;
    }
    let mean = sum as f32 / w as f32;
    sum_sq as f32 / w as f32 - mean * mean
}

fn strip_variance_col(img: &RgbaImage, col: u32) -> f32 {
    let h = img.height();
    if h == 0 {
        return 0.0;
    }
    let mut sum = 0u64;
    let mut sum_sq = 0u64;
    for y in 0..h {
        let p = img[(col, y)];
        let luma = (299u64 * p[0] as u64 + 587 * p[1] as u64 + 114 * p[2] as u64) / 1000;
        sum += luma;
        sum_sq += luma * luma;
    }
    let mean = sum as f32 / h as f32;
    sum_sq as f32 / h as f32 - mean * mean
}

pub fn sys_get(sub_command: &str, value: Option<&str>) -> String {
    let sub = sub_command.trim().to_uppercase();
    let monitor_num = value.and_then(|v| v.trim().parse::<u32>().ok());
    match sub.as_str() {
        "MONITORCOUNT" => monitor_get_count(),
        "MONITORPRIMARY" => monitor_get_primary(),
        "MONITOR" => monitor_get(monitor_num),
        "MONITORWORKAREA" => monitor_get_work_area(monitor_num),
        "MONITORNAME" => monitor_get_name(monitor_num),
        "0" => Monitor::all()
            .ok()
            .and_then(|m| m.into_iter().find(|mon| mon.is_primary()))
            .map(|mon| mon.width().to_string())
            .unwrap_or_default(),
        "1" => Monitor::all()
            .ok()
            .and_then(|m| m.into_iter().find(|mon| mon.is_primary()))
            .map(|mon| mon.height().to_string())
            .unwrap_or_default(),
        "80" => monitor_get_count(),
        _ => String::new(),
    }
}

pub fn pixel_get_color(x: i32, y: i32, mode: Option<&str>) -> String {
    let rgb_mode = mode
        .map(|m| m.to_uppercase().contains("RGB"))
        .unwrap_or(false);
    let Some(img) = capture_point(x, y) else {
        return String::new();
    };
    let (mx, my, img) = img;
    let rel_x = (x - mx) as u32;
    let rel_y = (y - my) as u32;
    if rel_x >= img.width() || rel_y >= img.height() {
        return String::new();
    }
    let pixel = img.get_pixel(rel_x, rel_y);
    let (r, g, b) = (pixel[0], pixel[1], pixel[2]);
    if rgb_mode {
        format!("0x{r:02X}{g:02X}{b:02X}")
    } else {
        format!("0x{b:02X}{g:02X}{r:02X}")
    }
}

pub fn pixel_search(
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    color: &str,
    variation: Option<i32>,
    mode: Option<&str>,
) -> String {
    let rgb_mode = mode
        .map(|m| m.to_uppercase().contains("RGB"))
        .unwrap_or(false);
    let var = variation.unwrap_or(0).clamp(0, 255) as u8;
    let (tr, tg, tb) = parse_color(color, rgb_mode);

    let Some(img) = capture_region_image(x1, y1, x2, y2) else {
        return String::new();
    };
    let (ox, oy, img) = img;
    for py in 0..img.height() {
        for px in 0..img.width() {
            let pixel = img.get_pixel(px, py);
            let (r, g, b) = (pixel[0], pixel[1], pixel[2]);
            if color_matches(r, g, b, tr, tg, tb, var) {
                return format!("{}|{}", ox + px as i32, oy + py as i32);
            }
        }
    }
    String::new()
}

pub fn image_search(x1: i32, y1: i32, x2: i32, y2: i32, image_file: &str) -> String {
    let options = parse_image_options(image_file);

    let Ok(template_dyn) = image::open(options.filepath) else {
        return String::new();
    };
    let template = resize_template(template_dyn.to_rgba8(), options.width, options.height);

    let Some(capture) = capture_region_image(x1, y1, x2, y2) else {
        return String::new();
    };
    let (ox, oy, haystack) = capture;

    let tw = template.width();
    let th = template.height();
    if tw == 0 || th == 0 || tw > haystack.width() || th > haystack.height() {
        return String::new();
    }

    for sy in 0..=(haystack.height() - th) {
        for sx in 0..=(haystack.width() - tw) {
            if template_matches(
                &haystack,
                sx,
                sy,
                &template,
                options.variation,
                options.trans_color,
            ) {
                return format!("{}|{}", ox + sx as i32, oy + sy as i32);
            }
        }
    }
    String::new()
}

fn pick_monitor(monitors: &[Monitor], num: Option<u32>) -> Option<&Monitor> {
    match num {
        Some(n) if n >= 1 => monitors.get((n - 1) as usize),
        _ => monitors
            .iter()
            .find(|m| m.is_primary())
            .or_else(|| monitors.first()),
    }
}

fn capture_point(x: i32, y: i32) -> Option<(i32, i32, RgbaImage)> {
    let monitors = Monitor::all().ok()?;
    let mon = monitors.iter().find(|m| {
        let mx = m.x();
        let my = m.y();
        let mw = m.width() as i32;
        let mh = m.height() as i32;
        x >= mx && x < mx + mw && y >= my && y < my + mh
    })?;
    let img = mon.capture_image().ok()?;
    Some((mon.x(), mon.y(), img))
}

fn capture_region_image(x1: i32, y1: i32, x2: i32, y2: i32) -> Option<(i32, i32, RgbaImage)> {
    let monitors = Monitor::all().ok()?;
    let mon = monitors.iter().find(|m| {
        let mx = m.x();
        let my = m.y();
        let mw = m.width() as i32;
        let mh = m.height() as i32;
        x1 < mx + mw && x2 > mx && y1 < my + mh && y2 > my
    })?;

    let full = mon.capture_image().ok()?;
    let mw = mon.width() as i32;
    let mh = mon.height() as i32;

    let rx1 = (x1 - mon.x()).max(0) as u32;
    let ry1 = (y1 - mon.y()).max(0) as u32;
    let rx2 = ((x2 - mon.x()).min(mw)) as u32;
    let ry2 = ((y2 - mon.y()).min(mh)) as u32;

    if rx2 <= rx1 || ry2 <= ry1 {
        return None;
    }

    let cropped = DynamicImage::ImageRgba8(full)
        .crop_imm(rx1, ry1, rx2 - rx1, ry2 - ry1)
        .to_rgba8();
    Some((mon.x() + rx1 as i32, mon.y() + ry1 as i32, cropped))
}

fn parse_color(color: &str, is_rgb: bool) -> (u8, u8, u8) {
    let val = parse_color_value(color).unwrap_or(0);
    let c1 = ((val >> 16) & 0xFF) as u8;
    let c2 = ((val >> 8) & 0xFF) as u8;
    let c3 = (val & 0xFF) as u8;
    if is_rgb {
        (c1, c2, c3)
    } else {
        (c3, c2, c1)
    }
}

fn color_matches(r: u8, g: u8, b: u8, tr: u8, tg: u8, tb: u8, variation: u8) -> bool {
    let diff = |a: u8, b: u8| (a as i16 - b as i16).unsigned_abs() as u8;
    diff(r, tr) <= variation && diff(g, tg) <= variation && diff(b, tb) <= variation
}

fn template_matches(
    haystack: &RgbaImage,
    sx: u32,
    sy: u32,
    template: &RgbaImage,
    variation: u8,
    trans_color: Option<(u8, u8, u8)>,
) -> bool {
    for ty in 0..template.height() {
        for tx in 0..template.width() {
            let tp = template.get_pixel(tx, ty);
            if tp[3] == 0 {
                continue;
            }
            if let Some((tr, tg, tb)) = trans_color {
                if (tp[0], tp[1], tp[2]) == (tr, tg, tb) {
                    continue;
                }
            }
            let hp = haystack.get_pixel(sx + tx, sy + ty);
            if !color_matches(hp[0], hp[1], hp[2], tp[0], tp[1], tp[2], variation) {
                return false;
            }
        }
    }
    true
}

fn parse_image_options(spec: &str) -> ImageSearchOptions<'_> {
    let trimmed = spec.trim();
    let mut variation = 0u8;
    let mut trans_color = None;
    let mut width = None;
    let mut height = None;
    let mut icon_index = None;
    let mut cursor = trimmed;

    while let Some(rest) = cursor.strip_prefix('*') {
        let token_end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let token = &rest[..token_end];
        let after = rest[token_end..].trim_start();
        let lower = token.to_ascii_lowercase();

        if let Ok(value) = token.parse::<i32>() {
            variation = value.clamp(0, 255) as u8;
            cursor = after;
            continue;
        }
        if let Some(value) = lower.strip_prefix("trans") {
            trans_color = parse_rgb_triplet(value);
            cursor = after;
            continue;
        }
        if let Some(value) = lower.strip_prefix('w') {
            if let Ok(parsed) = value.parse::<i32>() {
                width = Some(parsed);
                cursor = after;
                continue;
            }
        }
        if let Some(value) = lower.strip_prefix('h') {
            if let Ok(parsed) = value.parse::<i32>() {
                height = Some(parsed);
                cursor = after;
                continue;
            }
        }
        if let Some(value) = lower.strip_prefix("icon") {
            if let Ok(parsed) = value.parse::<u32>() {
                icon_index = Some(parsed);
                cursor = after;
                continue;
            }
        }

        break;
    }

    ImageSearchOptions {
        variation,
        trans_color,
        width,
        height,
        icon_index,
        filepath: cursor,
    }
}

fn resize_template(template: RgbaImage, width: Option<i32>, height: Option<i32>) -> RgbaImage {
    let src_width = template.width();
    let src_height = template.height();
    let Some((target_width, target_height)) =
        resolved_template_dimensions(src_width, src_height, width, height)
    else {
        return template;
    };

    if target_width == src_width || target_height == src_height {
        if target_width == src_width && target_height == src_height {
            return template;
        }
    }

    image::imageops::resize(&template, target_width, target_height, FilterType::Triangle)
}

fn resolved_template_dimensions(
    src_width: u32,
    src_height: u32,
    width: Option<i32>,
    height: Option<i32>,
) -> Option<(u32, u32)> {
    if src_width == 0 || src_height == 0 {
        return None;
    }

    let width = width.unwrap_or(src_width as i32);
    let height = height.unwrap_or(src_height as i32);

    let (width, height) = match (width, height) {
        (-1, h) if h > 0 => {
            let scaled = ((src_width as f64 * h as f64) / src_height as f64).round() as i32;
            (scaled.max(1), h)
        }
        (w, -1) if w > 0 => {
            let scaled = ((src_height as f64 * w as f64) / src_width as f64).round() as i32;
            (w, scaled.max(1))
        }
        (0, 0) => (src_width as i32, src_height as i32),
        (0, h) if h > 0 => (src_width as i32, h),
        (w, 0) if w > 0 => (w, src_height as i32),
        (w, h) if w > 0 && h > 0 => (w, h),
        _ => return None,
    };

    Some((width as u32, height as u32))
}

fn parse_rgb_triplet(value: &str) -> Option<(u8, u8, u8)> {
    let parsed = parse_color_value(value)?;
    Some((
        ((parsed >> 16) & 0xFF) as u8,
        ((parsed >> 8) & 0xFF) as u8,
        (parsed & 0xFF) as u8,
    ))
}

fn parse_color_value(color: &str) -> Option<u32> {
    let trimmed = color.trim();
    let lower = trimmed.to_ascii_lowercase();
    let named = match lower.as_str() {
        "black" => Some(0x000000),
        "silver" => Some(0xC0C0C0),
        "gray" | "grey" => Some(0x808080),
        "white" => Some(0xFFFFFF),
        "maroon" => Some(0x800000),
        "red" => Some(0xFF0000),
        "purple" => Some(0x800080),
        "fuchsia" => Some(0xFF00FF),
        "green" => Some(0x008000),
        "lime" => Some(0x00FF00),
        "olive" => Some(0x808000),
        "yellow" => Some(0xFFFF00),
        "navy" => Some(0x000080),
        "blue" => Some(0x0000FF),
        "teal" => Some(0x008080),
        "aqua" => Some(0x00FFFF),
        _ => None,
    };
    if let Some(value) = named {
        return Some(value);
    }

    let hex = trimmed.trim_start_matches("0x").trim_start_matches("0X");
    u32::from_str_radix(hex, 16).ok()
}

fn monitor_get_count_compat(_args: &[&str]) -> String {
    monitor_get_count()
}

fn monitor_get_primary_compat(_args: &[&str]) -> String {
    monitor_get_primary()
}

fn monitor_get_name_compat(args: &[&str]) -> String {
    let num = args.first().and_then(|v| v.trim().parse::<u32>().ok());
    monitor_get_name(num)
}

fn monitor_get_compat(args: &[&str]) -> String {
    let num = args.first().and_then(|v| v.trim().parse::<u32>().ok());
    monitor_get(num)
}

fn monitor_get_work_area_compat(args: &[&str]) -> String {
    let num = args.first().and_then(|v| v.trim().parse::<u32>().ok());
    monitor_get_work_area(num)
}

fn sys_get_compat(args: &[&str]) -> String {
    let sub = args.first().copied().unwrap_or_default();
    let value = args.get(1).copied().filter(|v| !v.trim().is_empty());
    sys_get(sub, value)
}

fn pixel_get_color_compat(args: &[&str]) -> String {
    let x = args
        .first()
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(0);
    let y = args
        .get(1)
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(0);
    let mode = args.get(2).copied().filter(|v| !v.trim().is_empty());
    pixel_get_color(x, y, mode)
}

fn pixel_search_compat(args: &[&str]) -> String {
    let x1 = args
        .first()
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(0);
    let y1 = args
        .get(1)
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(0);
    let x2 = args
        .get(2)
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(0);
    let y2 = args
        .get(3)
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(0);
    let color = args.get(4).copied().unwrap_or_default();
    let variation = args.get(5).and_then(|v| v.trim().parse::<i32>().ok());
    let mode = args.get(6).copied().filter(|v| !v.trim().is_empty());
    pixel_search(x1, y1, x2, y2, color, variation, mode)
}

fn image_search_compat(args: &[&str]) -> String {
    let x1 = args
        .first()
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(0);
    let y1 = args
        .get(1)
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(0);
    let x2 = args
        .get(2)
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(0);
    let y2 = args
        .get(3)
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(0);
    let image_file = args.get(4).copied().unwrap_or_default();
    image_search(x1, y1, x2, y2, image_file)
}

pub fn compat_image_search(args: &[&str]) -> String {
    image_search_compat(args)
}
pub fn compat_monitor_get(args: &[&str]) -> String {
    monitor_get_compat(args)
}
pub fn compat_monitor_get_count(args: &[&str]) -> String {
    monitor_get_count_compat(args)
}
pub fn compat_monitor_get_name(args: &[&str]) -> String {
    monitor_get_name_compat(args)
}
pub fn compat_monitor_get_primary(args: &[&str]) -> String {
    monitor_get_primary_compat(args)
}
pub fn compat_monitor_get_work_area(args: &[&str]) -> String {
    monitor_get_work_area_compat(args)
}
pub fn compat_pixel_get_color(args: &[&str]) -> String {
    pixel_get_color_compat(args)
}
pub fn compat_pixel_search(args: &[&str]) -> String {
    pixel_search_compat(args)
}
pub fn compat_sys_get(args: &[&str]) -> String {
    sys_get_compat(args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("ImageSearch", compat_image_search),
    ("MonitorGet", compat_monitor_get),
    ("MonitorGetCount", compat_monitor_get_count),
    ("MonitorGetName", compat_monitor_get_name),
    ("MonitorGetPrimary", compat_monitor_get_primary),
    ("MonitorGetWorkArea", compat_monitor_get_work_area),
    ("PixelGetColor", compat_pixel_get_color),
    ("PixelSearch", compat_pixel_search),
    ("SysGet", compat_sys_get),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_get_count_parseable() {
        let result = monitor_get_count();
        assert!(
            result.parse::<usize>().is_ok(),
            "count should be numeric: {result}"
        );
    }

    #[test]
    fn test_monitor_get_primary_parseable() {
        let result = monitor_get_primary();
        assert!(
            result.parse::<u32>().is_ok(),
            "primary index should be numeric: {result}"
        );
    }

    #[test]
    fn test_monitor_get_name_non_panicking() {
        let _ = monitor_get_name(None);
        let _ = monitor_get_name(Some(1));
        let _ = monitor_get_name(Some(999));
    }

    #[test]
    fn test_monitor_get_format() {
        let result = monitor_get(None);
        if !result.is_empty() {
            let parts: Vec<_> = result.split('|').collect();
            assert_eq!(
                parts.len(),
                4,
                "MonitorGet should return 4 pipe-separated values"
            );
            for p in &parts {
                assert!(
                    p.parse::<i32>().is_ok(),
                    "each bound should be numeric: {p}"
                );
            }
        }
    }

    #[test]
    fn test_sys_get_monitorcount_matches() {
        let a = monitor_get_count();
        let b = sys_get("MonitorCount", None);
        assert_eq!(a, b);
    }

    #[test]
    fn test_sys_get_monitorprimary_matches() {
        let a = monitor_get_primary();
        let b = sys_get("MonitorPrimary", None);
        assert_eq!(a, b);
    }

    #[test]
    fn test_pixel_color_format() {
        let result = pixel_get_color(0, 0, Some("RGB"));
        if !result.is_empty() {
            assert!(
                result.starts_with("0x"),
                "color should start with 0x: {result}"
            );
            assert_eq!(result.len(), 8, "color should be 0xRRGGBB: {result}");
        }
    }

    #[test]
    fn test_parse_color_rgb() {
        let (r, g, b) = parse_color("0xFF8800", true);
        assert_eq!((r, g, b), (0xFF, 0x88, 0x00));
    }

    #[test]
    fn test_parse_color_bgr() {
        let (r, g, b) = parse_color("0xFF8800", false);
        assert_eq!((r, g, b), (0x00, 0x88, 0xFF));
    }

    #[test]
    fn test_parse_image_options_with_variation() {
        let options = parse_image_options("*5 /tmp/template.png");
        assert_eq!(options.variation, 5);
        assert_eq!(options.filepath, "/tmp/template.png");
    }

    #[test]
    fn test_parse_image_options_no_prefix() {
        let options = parse_image_options("/tmp/template.png");
        assert_eq!(options.variation, 0);
        assert_eq!(options.filepath, "/tmp/template.png");
    }

    #[test]
    fn test_parse_image_options_with_trans_and_scale() {
        let options = parse_image_options("*TransWhite *w200 *h-1 *Icon2 /tmp/template.png");
        assert_eq!(options.trans_color, Some((0xFF, 0xFF, 0xFF)));
        assert_eq!(options.width, Some(200));
        assert_eq!(options.height, Some(-1));
        assert_eq!(options.icon_index, Some(2));
        assert_eq!(options.filepath, "/tmp/template.png");
    }

    #[test]
    fn test_resolved_template_dimensions_preserves_aspect_ratio() {
        let dims = resolved_template_dimensions(100, 50, Some(200), Some(-1));
        assert_eq!(dims, Some((200, 100)));
    }

    #[test]
    fn test_template_matches_respects_trans_color() {
        let haystack = RgbaImage::from_pixel(2, 2, image::Rgba([10, 20, 30, 255]));
        let template = RgbaImage::from_fn(1, 1, |_x, _y| image::Rgba([255, 255, 255, 255]));
        assert!(template_matches(
            &haystack,
            0,
            0,
            &template,
            0,
            Some((255, 255, 255)),
        ));
    }

    #[test]
    fn test_parse_color_value_named_color() {
        assert_eq!(parse_color_value("Aqua"), Some(0x00FFFF));
    }

    #[test]
    fn test_image_search_missing_file_returns_empty() {
        let result = image_search(0, 0, 100, 100, "/nonexistent/template.png");
        assert_eq!(result, "");
    }

    #[test]
    fn test_pixel_search_no_match_returns_empty() {
        let result = pixel_search(0, 0, 0, 0, "0x000000", None, None);
        assert_eq!(result, "");
    }
}
