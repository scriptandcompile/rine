use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::objects::{Bitmap, GdiObject, GdiState};

static NEXT_HANDLE: AtomicUsize = AtomicUsize::new(0x10000);
static GDI_STATE: OnceLock<Mutex<GdiState>> = OnceLock::new();

pub(crate) fn gdi_state() -> &'static Mutex<GdiState> {
    GDI_STATE.get_or_init(|| Mutex::new(GdiState::default()))
}

pub(crate) fn alloc_handle() -> usize {
    NEXT_HANDLE.fetch_add(1, Ordering::Relaxed)
}

pub(crate) fn object_selected_by_any_dc(state: &GdiState, object: usize) -> bool {
    state.dcs.values().any(|dc| {
        dc.selected_bitmap == Some(object)
            || dc.selected_brush == Some(object)
            || dc.selected_pen == Some(object)
    })
}

pub(crate) fn with_selected_bitmap_mut<R>(
    state: &mut GdiState,
    dc_handle: usize,
    f: impl FnOnce(&mut Bitmap) -> R,
) -> Option<R> {
    let dc = state.dcs.get(&dc_handle)?;
    let bitmap_handle = dc.selected_bitmap?;
    let object = state.objects.get_mut(&bitmap_handle)?;
    let GdiObject::Bitmap(bitmap) = object else {
        return None;
    };
    Some(f(bitmap))
}
