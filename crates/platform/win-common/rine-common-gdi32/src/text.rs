use crate::objects::Bitmap;

pub(crate) const PSEUDO_GLYPH_WIDTH: i32 = 8;
pub(crate) const PSEUDO_GLYPH_HEIGHT: i32 = 12;

pub(crate) fn draw_text(bitmap: &mut Bitmap, x: i32, y: i32, text: &str) {
    for (glyph_index, ch) in text.chars().enumerate() {
        let glyph_base_x = x + (glyph_index as i32 * PSEUDO_GLYPH_WIDTH);
        let glyph = ch as u32;

        for row in 0..PSEUDO_GLYPH_HEIGHT {
            for col in 0..PSEUDO_GLYPH_WIDTH {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_empty_string_leaves_bitmap_unchanged() {
        let mut bm = Bitmap::new(10, 12).unwrap();
        draw_text(&mut bm, 0, 0, "");
        assert!(bm.pixels.iter().all(|&p| p == 0));
    }

    #[test]
    fn draw_char_sets_some_pixels() {
        let mut bm = Bitmap::new(100, 100).unwrap();
        draw_text(&mut bm, 10, 20, "A");
        // 'A' (0x41) has bit 0 and bit 6 set.
        // At (row=0, col=0): (0 & 7) = 0, bit 0 = 1 → pixel set.
        assert_eq!(bm.pixels[bm.index(10, 20).unwrap()], 0x00FF_FFFF);
        // At (row=0, col=1): (1 & 7) = 1, bit 1 = 0 → pixel unset.
        assert_eq!(bm.pixels[bm.index(11, 20).unwrap()], 0);
    }

    #[test]
    fn draw_text_off_screen_does_not_panic() {
        let mut bm = Bitmap::new(10, 10).unwrap();
        draw_text(&mut bm, -100, -100, "test");
        assert!(bm.pixels.iter().all(|&p| p == 0));
    }

    #[test]
    fn draw_text_partial_clip_respects_bounds() {
        let mut bm = Bitmap::new(5, 5).unwrap();
        // Drawing at (-3, -3) so most pixels are off-screen,
        // but some of the 8x12 glyph area may overlap (0,0)-(4,4).
        draw_text(&mut bm, -3, -3, "AB");
        // No panic expected; any pixels written should be inside the bitmap.
    }

    #[test]
    fn draw_two_chars_wider_than_one() {
        let mut bm = Bitmap::new(200, 100).unwrap();
        draw_text(&mut bm, 0, 0, "AB");
        // 'B' (0x42) has bit 1 set: at (row=0, col=1): (1 & 7) = 1 → set.
        // Glyph 'B' starts at x offset 8.
        assert_eq!(bm.pixels[bm.index(9, 0).unwrap()], 0x00FF_FFFF);
        // First glyph 'A' next to second glyph 'B'
        assert_eq!(bm.pixels[bm.index(7, 0).unwrap()], 0);
    }
}
