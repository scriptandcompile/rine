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

#[derive(Default)]
pub(crate) struct DeviceContext {
    pub(crate) selected_bitmap: Option<usize>,
    pub(crate) selected_brush: Option<usize>,
    pub(crate) selected_pen: Option<usize>,
    pub(crate) owned_objects: Vec<usize>,
}

#[derive(Default)]
pub(crate) struct GdiState {
    pub(crate) dcs: HashMap<usize, DeviceContext>,
    pub(crate) objects: HashMap<usize, GdiObject>,
}
