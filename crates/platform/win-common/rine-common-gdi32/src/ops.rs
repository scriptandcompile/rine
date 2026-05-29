use rine_types::errors::BOOL;
use rine_types::strings::{LPCSTR, LPCWSTR};
use rine_types::windows::{Rect, Size};

use crate::objects::{Bitmap, Brush, DeviceContext, Font, GdiObject, Pen};
use crate::state::{alloc_handle, gdi_state, object_selected_by_any_dc, with_selected_bitmap_mut};
use crate::telemetry::{notify_bitmap_alloc, notify_bitmap_free};
use crate::text::{PSEUDO_GLYPH_HEIGHT, PSEUDO_GLYPH_WIDTH, draw_text};

mod constants;
mod drawing;
mod object_ops;
mod text;

pub use constants::{
    BLACK_BRUSH, BLACK_PEN, DC_BRUSH, DC_PEN, DKGRAY_BRUSH, GRAY_BRUSH, HOLLOW_BRUSH,
    LTGRAY_BRUSH, MM_ANISOTROPIC, MM_HIENGLISH, MM_HIMETRIC, MM_ISOTROPIC, MM_LOENGLISH,
    MM_LOMETRIC, MM_TEXT, MM_TWIPS, NULL_BRUSH, NULL_PEN, OPAQUE, SRCCOPY, TRANSPARENT,
    WHITE_BRUSH, WHITE_PEN,
};
pub(crate) use constants::bitmap_bytes;
use constants::{is_valid_bk_mode, is_valid_map_mode, stock_object_for_index};

pub use drawing::bit_blt;
pub use object_ops::{
    create_compatible_bitmap, create_compatible_dc, create_font_a, create_font_w, create_pen,
    create_solid_brush, delete_dc, delete_object, get_stock_object, select_object, set_bk_mode,
    set_map_mode,
};
pub use text::{get_text_extent_point_a, text_out};
