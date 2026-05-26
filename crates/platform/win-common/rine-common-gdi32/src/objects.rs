use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct Bitmap {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) pixels: Vec<u32>,
}

impl Bitmap {
    pub(crate) fn new(width: i32, height: i32) -> Option<Self> {
        if width <= 0 || height <= 0 {
            return None;
        }

        let width = width as usize;
        let height = height as usize;
        Some(Self {
            width,
            height,
            pixels: vec![0; width.saturating_mul(height)],
        })
    }

    pub(crate) fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 {
            return None;
        }

        let x = x as usize;
        let y = y as usize;
        if x >= self.width || y >= self.height {
            return None;
        }

        Some(y * self.width + x)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Brush {
    pub(crate) color: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct Pen {
    pub(crate) color: u32,
}

pub(crate) enum GdiObject {
    Bitmap(Bitmap),
    Brush(Brush),
    Pen(Pen),
}

pub(crate) struct DeviceContext {
    pub(crate) selected_bitmap: Option<usize>,
    pub(crate) selected_brush: Option<usize>,
    pub(crate) selected_pen: Option<usize>,
    pub(crate) bk_mode: i32,
    pub(crate) map_mode: i32,
    pub(crate) owned_objects: Vec<usize>,
}

impl Default for DeviceContext {
    fn default() -> Self {
        Self {
            selected_bitmap: None,
            selected_brush: None,
            selected_pen: None,
            // Windows defaults for a newly created memory DC.
            bk_mode: crate::ops::OPAQUE,
            map_mode: crate::ops::MM_TEXT,
            owned_objects: Vec::new(),
        }
    }
}

#[derive(Default)]
pub(crate) struct GdiState {
    pub(crate) dcs: HashMap<usize, DeviceContext>,
    pub(crate) objects: HashMap<usize, GdiObject>,
    pub(crate) stock_objects: HashMap<i32, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmap_new_valid_dimensions() {
        let bm = Bitmap::new(32, 16).expect("valid dimensions should create bitmap");
        assert_eq!(bm.width, 32);
        assert_eq!(bm.height, 16);
        assert_eq!(bm.pixels.len(), 32 * 16);
        assert!(bm.pixels.iter().all(|&p| p == 0));
    }

    #[test]
    fn bitmap_new_zero_width() {
        assert!(Bitmap::new(0, 16).is_none());
    }

    #[test]
    fn bitmap_new_zero_height() {
        assert!(Bitmap::new(32, 0).is_none());
    }

    #[test]
    fn bitmap_new_negative_width() {
        assert!(Bitmap::new(-1, 16).is_none());
    }

    #[test]
    fn bitmap_new_negative_height() {
        assert!(Bitmap::new(32, -8).is_none());
    }

    #[test]
    fn bitmap_new_one_by_one() {
        let bm = Bitmap::new(1, 1).expect("1x1 is valid");
        assert_eq!(bm.width, 1);
        assert_eq!(bm.height, 1);
        assert_eq!(bm.pixels.len(), 1);
    }

    #[test]
    fn bitmap_index_valid() {
        let bm = Bitmap::new(10, 10).unwrap();
        assert_eq!(bm.index(0, 0), Some(0));
        assert_eq!(bm.index(5, 3), Some(3 * 10 + 5));
        assert_eq!(bm.index(9, 9), Some(9 * 10 + 9));
    }

    #[test]
    fn bitmap_index_negative_x() {
        let bm = Bitmap::new(10, 10).unwrap();
        assert!(bm.index(-1, 0).is_none());
    }

    #[test]
    fn bitmap_index_negative_y() {
        let bm = Bitmap::new(10, 10).unwrap();
        assert!(bm.index(0, -5).is_none());
    }

    #[test]
    fn bitmap_index_out_of_bounds_x() {
        let bm = Bitmap::new(10, 10).unwrap();
        assert!(bm.index(10, 0).is_none());
        assert!(bm.index(15, 0).is_none());
    }

    #[test]
    fn bitmap_index_out_of_bounds_y() {
        let bm = Bitmap::new(10, 10).unwrap();
        assert!(bm.index(0, 10).is_none());
        assert!(bm.index(0, 20).is_none());
    }

    #[test]
    fn bitmap_index_out_of_bounds_both() {
        let bm = Bitmap::new(10, 10).unwrap();
        assert!(bm.index(10, 10).is_none());
        assert!(bm.index(100, 100).is_none());
    }
}
