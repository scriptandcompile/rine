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
        let text_y_offset = i32::try_from((h.saturating_sub(14)) / 2).unwrap_or(0);
        let text_y = y.saturating_add(text_y_offset);
        draw_text_5x7(pixmap, text_x, text_y, &child.title, Color::BLACK, 2);
    }
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
