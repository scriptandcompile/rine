use crate::objects::Bitmap;
use crate::ops::bitmap_bytes;

pub(crate) fn notify_bitmap_alloc(handle: usize, bitmap: &Bitmap) {
    let detail = format!(
        r#"{{"handle":{},"width":{},"height":{},"bytes":{}}}"#,
        handle,
        bitmap.width,
        bitmap.height,
        bitmap_bytes(bitmap)
    );
    rine_types::dev_notify!(on_handle_created(handle as i64, "GdiBitmap", &detail));
    rine_types::dev_notify!(on_memory_allocated(
        bitmap.pixels.as_ptr() as u64,
        bitmap_bytes(bitmap),
        "GDI Bitmap",
    ));
}

pub(crate) fn notify_bitmap_free(bitmap: &Bitmap) {
    rine_types::dev_notify!(on_memory_freed(
        bitmap.pixels.as_ptr() as u64,
        bitmap_bytes(bitmap),
        "GDI Bitmap",
    ));
}
