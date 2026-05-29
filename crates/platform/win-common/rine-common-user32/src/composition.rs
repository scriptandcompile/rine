use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use fontdb::{Database, Family, Query, Stretch, Style, Weight};
use fontdue::{Font, FontSettings};
use rine_common_gdi32::get_logical_font_info;
use rine_types::windows::window_message;
use rine_types::windows::{HWND, WINDOW_MANAGER, WindowState};
use tiny_skia::{Color, Paint, Pixmap, PremultipliedColorU8, Rect};

pub(crate) struct ComposedFrame {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels_argb: Vec<u32>,
}

pub(crate) fn compose_top_level_frame(hwnd: HWND) -> Option<ComposedFrame> {
    let top_level = WINDOW_MANAGER.get_window(hwnd)?;
    let width = u32::try_from(top_level.rect.right.saturating_sub(top_level.rect.left)).ok()?;
    let height = u32::try_from(top_level.rect.bottom.saturating_sub(top_level.rect.top)).ok()?;

    if width == 0 || height == 0 {
        return None;
    }

    let mut pixmap = Pixmap::new(width, height)?;
    pixmap.fill(Color::WHITE);

    for child in WINDOW_MANAGER.child_windows(hwnd) {
        if !child.visible {
            continue;
        }

        draw_control(&mut pixmap, &child);
    }

    Some(ComposedFrame {
        width,
        height,
        pixels_argb: pixmap_to_argb32(&pixmap),
    })
}

fn draw_control(pixmap: &mut Pixmap, child: &WindowState) {
    let control_width = child.rect.right.saturating_sub(child.rect.left);
    let control_height = child.rect.bottom.saturating_sub(child.rect.top);

    if control_width <= 0 || control_height <= 0 {
        return;
    }

    let x = child.rect.left.max(0);
    let y = child.rect.top.max(0);
    let w = u32::try_from(control_width).unwrap_or(0);
    let h = u32::try_from(control_height).unwrap_or(0);

    if w == 0 || h == 0 {
        return;
    }

    let class_name = child.class_name.as_str();

    if class_name.eq_ignore_ascii_case("BUTTON") {
        fill_rect(pixmap, x, y, w, h, Color::from_rgba8(220, 220, 220, 255));
        draw_rect_border(pixmap, x, y, w, h, Color::BLACK);
    } else if class_name.eq_ignore_ascii_case("EDIT") {
        fill_rect(pixmap, x, y, w, h, Color::WHITE);
        draw_rect_border(pixmap, x, y, w, h, Color::from_rgba8(96, 96, 96, 255));
    }

    if !child.title.is_empty() {
        let text_x = x.saturating_add(6);
        let text_y = y.saturating_add(4);
        let text_width = w.saturating_sub(12);
        let text_height = h.saturating_sub(8);
        let text_style = resolve_text_style(child.hwnd);
        if !draw_text_with_font(
            pixmap,
            text_x,
            text_y,
            text_width,
            text_height,
            &child.title,
            Color::BLACK,
            &text_style,
        ) {
            draw_text_5x7(pixmap, text_x, text_y, &child.title, Color::BLACK, 2);
        }
    }
}

#[derive(Clone)]
struct TextStyle {
    size: f32,
    weight: Weight,
    style: Style,
    face_name: Option<String>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: 13.0,
            weight: Weight::NORMAL,
            style: Style::Normal,
            face_name: None,
        }
    }
}

fn resolve_text_style(hwnd: HWND) -> TextStyle {
    let mut style = TextStyle::default();
    let font_handle = crate::message_queue::def_window_proc(hwnd, window_message::WM_GETFONT, 0, 0);
    let Ok(font_handle) = usize::try_from(font_handle) else {
        return style;
    };
    if font_handle == 0 {
        return style;
    }

    let Some(info) = get_logical_font_info(font_handle) else {
        return style;
    };

    let abs_height = info.height.unsigned_abs();
    if abs_height > 0 {
        style.size = (abs_height as f32).clamp(8.0, 72.0);
    }

    style.weight = if info.weight >= 700 {
        Weight::BOLD
    } else {
        Weight::NORMAL
    };
    style.style = if info.italic {
        Style::Italic
    } else {
        Style::Normal
    };

    // Preserve explicit face names from CreateFont* while ignoring empty strings.
    style.face_name = info.face_name.and_then(|name| {
        let trimmed = name.trim().to_owned();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });

    style
}

fn fill_rect(pixmap: &mut Pixmap, x: i32, y: i32, w: u32, h: u32, color: Color) {
    let Some(rect) = Rect::from_xywh(x as f32, y as f32, w as f32, h as f32) else {
        return;
    };

    let mut paint = Paint::default();
    paint.set_color(color);
    pixmap.fill_rect(rect, &paint, tiny_skia::Transform::identity(), None);
}

fn draw_rect_border(pixmap: &mut Pixmap, x: i32, y: i32, w: u32, h: u32, color: Color) {
    if w < 2 || h < 2 {
        return;
    }

    fill_rect(pixmap, x, y, w, 1, color);
    fill_rect(pixmap, x, y + (h as i32) - 1, w, 1, color);
    fill_rect(pixmap, x, y + 1, 1, h - 2, color);
    fill_rect(pixmap, x + (w as i32) - 1, y + 1, 1, h - 2, color);
}

fn default_ui_font() -> Option<&'static Font> {
    static FONT: OnceLock<Option<Font>> = OnceLock::new();
    FONT.get_or_init(load_default_ui_font).as_ref()
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct TextStyleCacheKey {
    face_name: Option<String>,
    bold: bool,
    italic: bool,
}

impl TextStyleCacheKey {
    fn from_style(style: &TextStyle) -> Self {
        Self {
            face_name: style.face_name.clone(),
            bold: style.weight == Weight::BOLD,
            italic: style.style == Style::Italic,
        }
    }
}

fn cached_font_for_text_style(style: &TextStyle) -> Option<Font> {
    static CACHE: OnceLock<Mutex<HashMap<TextStyleCacheKey, Option<Font>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    let key = TextStyleCacheKey::from_style(style);
    {
        let cache_guard = cache.lock().unwrap();
        if let Some(cached_font) = cache_guard.get(&key) {
            return cached_font.clone();
        }
    }

    let loaded = load_font_for_text_style(style);

    let mut cache_guard = cache.lock().unwrap();
    cache_guard.insert(key, loaded.clone());
    loaded
}

fn load_default_ui_font() -> Option<Font> {
    load_font_for_text_style(&TextStyle::default())
}

fn load_font_for_text_style(style: &TextStyle) -> Option<Font> {
    let mut db = Database::new();
    db.load_system_fonts();

    let mut families = Vec::new();
    if let Some(face_name) = style.face_name.as_deref() {
        families.push(Family::Name(face_name));
    }
    families.push(Family::Name("Segoe UI"));
    families.push(Family::Name("Noto Sans"));
    families.push(Family::SansSerif);

    let query = Query {
        families: &families,
        weight: style.weight,
        stretch: Stretch::Normal,
        style: style.style,
    };

    let id = db
        .query(&query)
        .or_else(|| db.faces().next().map(|face| face.id))?;
    db.with_face_data(id, |data, face_index| {
        Font::from_bytes(
            data,
            FontSettings {
                collection_index: face_index,
                ..FontSettings::default()
            },
        )
        .ok()
    })?
}

#[allow(clippy::too_many_arguments)]
fn draw_text_with_font(
    pixmap: &mut Pixmap,
    x: i32,
    y: i32,
    max_width: u32,
    max_height: u32,
    text: &str,
    color: Color,
    style: &TextStyle,
) -> bool {
    let loaded_font = cached_font_for_text_style(style);
    let fallback_font = default_ui_font().cloned();
    let Some(font) = loaded_font.or(fallback_font) else {
        return false;
    };
    let font_size = style.size;

    if max_width == 0 || max_height == 0 {
        return true;
    }

    let min_x = x;
    let min_y = y;
    let max_x = x.saturating_add(max_width as i32);
    let max_y = y.saturating_add(max_height as i32);

    struct PositionedGlyph {
        x: i32,
        top_offset: i32,
        width: usize,
        height: usize,
        bitmap: Vec<u8>,
    }

    let mut glyphs = Vec::new();
    let mut pen_x = x as f32;
    let mut prev = None;
    let mut run_top = i32::MAX;
    let mut run_bottom = i32::MIN;

    for ch in text.chars() {
        if ch == '\n' || pen_x >= max_x as f32 {
            break;
        }

        if let Some(prev_ch) = prev {
            pen_x += font.horizontal_kern(prev_ch, ch, font_size).unwrap_or(0.0);
        }

        let (metrics, bitmap) = font.rasterize(ch, font_size);
        let glyph_x = (pen_x + metrics.xmin as f32).round() as i32;
        let top_offset = -(metrics.height as i32) - metrics.ymin;
        let bottom_offset = top_offset.saturating_add(metrics.height as i32);

        if metrics.width > 0 && metrics.height > 0 {
            run_top = run_top.min(top_offset);
            run_bottom = run_bottom.max(bottom_offset);
        }

        glyphs.push(PositionedGlyph {
            x: glyph_x,
            top_offset,
            width: metrics.width,
            height: metrics.height,
            bitmap,
        });

        pen_x += metrics.advance_width;
        prev = Some(ch);
    }

    if glyphs.is_empty() {
        return true;
    }

    let text_pixel_height = if run_top <= run_bottom {
        (run_bottom - run_top).max(1)
    } else {
        font_size.round() as i32
    };

    let vertical_padding = ((max_height as i32).saturating_sub(text_pixel_height) / 2).max(0);
    let baseline = y
        .saturating_add(vertical_padding)
        .saturating_sub(run_top.min(0));

    for glyph in glyphs {
        let glyph_y = baseline.saturating_add(glyph.top_offset);
        draw_alpha_bitmap(
            pixmap,
            glyph.x,
            glyph_y,
            glyph.width,
            glyph.height,
            &glyph.bitmap,
            color,
            min_x,
            min_y,
            max_x,
            max_y,
        );
    }

    true
}

#[allow(clippy::too_many_arguments)]
fn draw_alpha_bitmap(
    pixmap: &mut Pixmap,
    x: i32,
    y: i32,
    width: usize,
    height: usize,
    alpha_bitmap: &[u8],
    color: Color,
    clip_min_x: i32,
    clip_min_y: i32,
    clip_max_x: i32,
    clip_max_y: i32,
) {
    if width == 0 || height == 0 {
        return;
    }

    let r = (color.red() * 255.0).round() as u8;
    let g = (color.green() * 255.0).round() as u8;
    let b = (color.blue() * 255.0).round() as u8;
    let base_alpha = (color.alpha() * 255.0).round() as u8;

    for row in 0..height {
        for col in 0..width {
            let coverage = alpha_bitmap[row * width + col];
            if coverage == 0 {
                continue;
            }

            let px = x.saturating_add(col as i32);
            let py = y.saturating_add(row as i32);
            if px < clip_min_x || py < clip_min_y || px >= clip_max_x || py >= clip_max_y {
                continue;
            }

            let alpha = ((u16::from(base_alpha) * u16::from(coverage)) / 255) as u8;
            fill_rect(pixmap, px, py, 1, 1, Color::from_rgba8(r, g, b, alpha));
        }
    }
}

fn draw_text_5x7(pixmap: &mut Pixmap, x: i32, y: i32, text: &str, color: Color, scale: u32) {
    let mut pen_x = x;
    for c in text.chars() {
        if c == ' ' {
            pen_x += (6 * scale) as i32;
            continue;
        }

        draw_glyph_5x7(pixmap, pen_x, y, c, color, scale);
        pen_x += (6 * scale) as i32;
    }
}

fn draw_glyph_5x7(pixmap: &mut Pixmap, x: i32, y: i32, ch: char, color: Color, scale: u32) {
    let rows = glyph_5x7(ch);

    for (row_idx, row) in rows.iter().enumerate() {
        for col_idx in 0usize..5usize {
            let bit = (row >> (4 - col_idx)) & 0x01;
            if bit == 0 {
                continue;
            }

            let px = x + (col_idx as i32 * scale as i32);
            let py = y + (row_idx as i32 * scale as i32);
            fill_rect(pixmap, px, py, scale, scale, color);
        }
    }
}

fn glyph_5x7(ch: char) -> [u8; 7] {
    let upper = ch.to_ascii_uppercase();
    match upper {
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        '6' => [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b11100,
        ],
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'G' => [
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10001, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100,
        ],
        ':' => [
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        _ => [
            0b11111, 0b10001, 0b00110, 0b00100, 0b00110, 0b10001, 0b11111,
        ],
    }
}

fn pixmap_to_argb32(pixmap: &Pixmap) -> Vec<u32> {
    let mut out = Vec::with_capacity((pixmap.width() * pixmap.height()) as usize);

    for px in pixmap.pixels() {
        out.push(premul_to_argb32(*px));
    }

    out
}

fn premul_to_argb32(px: PremultipliedColorU8) -> u32 {
    let r = u32::from(px.red());
    let g = u32::from(px.green());
    let b = u32::from(px.blue());
    let a = u32::from(px.alpha());
    (a << 24) | (r << 16) | (g << 8) | b
}
