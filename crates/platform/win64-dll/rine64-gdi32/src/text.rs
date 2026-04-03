use crate::objects::Bitmap;

pub(crate) fn draw_text(bitmap: &mut Bitmap, x: i32, y: i32, text: &str) {
    for (glyph_index, ch) in text.chars().enumerate() {
        let glyph_base_x = x + (glyph_index as i32 * 8);
        let glyph = ch as u32;

        for row in 0..12 {
            for col in 0..8 {
                // Lightweight pseudo-glyph so TextOut mutates the software surface.
                if ((glyph >> ((row + col) & 0x7)) & 1) == 0 {
                    continue;
                }

                let px = glyph_base_x + col;
                let py = y + row;
                if let Some(index) = bitmap.index(px, py) {
                    bitmap.pixels[index] = 0x00FF_FFFF;
                }
            }
        }
    }
}
