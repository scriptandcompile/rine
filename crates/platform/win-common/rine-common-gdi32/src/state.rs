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
            || dc.selected_font == Some(object)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::DeviceContext;

    #[test]
    fn object_not_selected_when_no_dcs() {
        let state = GdiState::default();
        assert!(!object_selected_by_any_dc(&state, 0x10001));
    }

    #[test]
    fn object_selected_as_bitmap() {
        let mut state = GdiState::default();
        let dc_handle = 1;
        let obj_handle = 42;
        state.dcs.insert(
            dc_handle,
            DeviceContext {
                selected_bitmap: Some(obj_handle),
                ..Default::default()
            },
        );
        assert!(object_selected_by_any_dc(&state, obj_handle));
    }

    #[test]
    fn object_selected_as_brush() {
        let mut state = GdiState::default();
        let dc_handle = 1;
        let obj_handle = 42;
        state.dcs.insert(
            dc_handle,
            DeviceContext {
                selected_brush: Some(obj_handle),
                ..Default::default()
            },
        );
        assert!(object_selected_by_any_dc(&state, obj_handle));
    }

    #[test]
    fn object_selected_as_pen() {
        let mut state = GdiState::default();
        let dc_handle = 1;
        let obj_handle = 42;
        state.dcs.insert(
            dc_handle,
            DeviceContext {
                selected_pen: Some(obj_handle),
                ..Default::default()
            },
        );
        assert!(object_selected_by_any_dc(&state, obj_handle));
    }

    #[test]
    fn object_selected_as_font() {
        let mut state = GdiState::default();
        let dc_handle = 1;
        let obj_handle = 42;
        state.dcs.insert(
            dc_handle,
            DeviceContext {
                selected_font: Some(obj_handle),
                ..Default::default()
            },
        );
        assert!(object_selected_by_any_dc(&state, obj_handle));
    }

    #[test]
    fn object_selected_in_one_of_many_dcs() {
        let mut state = GdiState::default();
        let obj_handle = 99;
        state.dcs.insert(1, DeviceContext::default());
        state.dcs.insert(
            2,
            DeviceContext {
                selected_bitmap: Some(obj_handle),
                ..Default::default()
            },
        );
        state.dcs.insert(3, DeviceContext::default());
        assert!(object_selected_by_any_dc(&state, obj_handle));
    }

    #[test]
    fn object_not_selected_when_different_handle() {
        let mut state = GdiState::default();
        state.dcs.insert(
            1,
            DeviceContext {
                selected_bitmap: Some(10),
                ..Default::default()
            },
        );
        assert!(!object_selected_by_any_dc(&state, 99));
    }

    #[test]
    fn with_selected_bitmap_mut_returns_none_for_missing_dc() {
        let mut state = GdiState::default();
        let result = with_selected_bitmap_mut(&mut state, 999, |_| 42);
        assert!(result.is_none());
    }

    #[test]
    fn with_selected_bitmap_mut_returns_none_when_no_bitmap_selected() {
        let mut state = GdiState::default();
        state.dcs.insert(1, DeviceContext::default());
        let result = with_selected_bitmap_mut(&mut state, 1, |_| 42);
        assert!(result.is_none());
    }

    #[test]
    fn with_selected_bitmap_mut_returns_none_for_missing_object() {
        let mut state = GdiState::default();
        state.dcs.insert(
            1,
            DeviceContext {
                selected_bitmap: Some(100),
                ..Default::default()
            },
        );
        let result = with_selected_bitmap_mut(&mut state, 1, |_| 42);
        assert!(result.is_none());
    }

    #[test]
    fn with_selected_bitmap_mut_mutates_bitmap() {
        let mut state = GdiState::default();
        let obj_handle = 100;
        state.dcs.insert(
            1,
            DeviceContext {
                selected_bitmap: Some(obj_handle),
                ..Default::default()
            },
        );
        state
            .objects
            .insert(obj_handle, GdiObject::Bitmap(Bitmap::new(2, 2).unwrap()));

        let pixel_count = with_selected_bitmap_mut(&mut state, 1, |bmp| {
            bmp.pixels[0] = 0xFF0000;
            bmp.pixels.len()
        })
        .unwrap();
        assert_eq!(pixel_count, 4);

        let updated = match state.objects.get(&obj_handle).unwrap() {
            GdiObject::Bitmap(b) => b,
            _ => panic!("not a bitmap"),
        };
        assert_eq!(updated.pixels[0], 0xFF0000);
    }

    #[test]
    fn object_selected_does_not_panic_with_empty_state() {
        let state = GdiState::default();
        assert!(!object_selected_by_any_dc(&state, 0));
    }
}
