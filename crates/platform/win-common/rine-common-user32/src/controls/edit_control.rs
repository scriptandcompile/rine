use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use rine_types::errors::BOOL;
use rine_types::windows::{HWND, PaintStruct, window_message};

use crate::device_context::{begin_paint, end_paint};
use crate::window_lifecycle::invalidate_rect;

fn font_store() -> &'static Mutex<HashMap<HWND, usize>> {
    static STORE: OnceLock<Mutex<HashMap<HWND, usize>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn default_proc(hwnd: HWND, msg: u32, w_param: usize, l_param: isize) -> isize {
    match msg {
        window_message::WM_SETFONT => {
            font_store().lock().unwrap().insert(hwnd, w_param);
            if l_param != 0 {
                let _ = invalidate_rect(hwnd, None, BOOL::FALSE);
            }
            0
        }
        window_message::WM_GETFONT => font_store()
            .lock()
            .unwrap()
            .get(&hwnd)
            .copied()
            .unwrap_or(0) as isize,
        window_message::WM_PAINT => {
            let mut ps = PaintStruct {
                hdc: rine_types::windows::HDC::NULL,
                f_erase: BOOL::FALSE,
                rc_paint: rine_types::windows::Rect::default(),
                f_restore: BOOL::FALSE,
                f_inc_update: BOOL::FALSE,
                rgb_reserved: [0; 32],
            };
            unsafe {
                let hdc = begin_paint(hwnd, &mut ps);
                if !hdc.is_null() {
                    let _ = end_paint(hwnd, &ps);
                }
            }
            0
        }
        window_message::WM_DESTROY => {
            font_store().lock().unwrap().remove(&hwnd);
            0
        }
        _ => 0,
    }
}
