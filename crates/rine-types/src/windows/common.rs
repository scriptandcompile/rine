//! Common Windows geometry types used across multiple subsystems.

// ---------------------------------------------------------------------------
// RECT structure
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }
}

/// The Windows POINT structure.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_default() {
        let rect = Rect::default();
        assert_eq!(rect.left, 0);
        assert_eq!(rect.top, 0);
        assert_eq!(rect.right, 0);
        assert_eq!(rect.bottom, 0);
    }

    #[test]
    fn point_default() {
        let point = Point::default();
        assert_eq!(point.x, 0);
        assert_eq!(point.y, 0);
    }
}
