use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use rine_types::errors::BOOL;
use rine_types::windows::{HWND, PaintStruct, window_message};

use crate::device_context::{begin_paint, end_paint};
use crate::window_lifecycle::invalidate_rect;

#[derive(Clone, Copy, Default)]
struct ButtonState {
    font: usize,
    pressed: bool,
}

fn state_store() -> &'static Mutex<HashMap<HWND, ButtonState>> {
    static STORE: OnceLock<Mutex<HashMap<HWND, ButtonState>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn update_state(hwnd: HWND, updater: impl FnOnce(&mut ButtonState)) {
    let mut store = state_store().lock().unwrap();
    let state = store.entry(hwnd).or_default();
    updater(state);
}

pub fn default_proc(hwnd: HWND, msg: u32, w_param: usize, l_param: isize) -> isize {
    match msg {
        window_message::WM_SETFONT => {
            update_state(hwnd, |state| state.font = w_param);
            if l_param != 0 {
                let _ = invalidate_rect(hwnd, None, BOOL::FALSE);
            }
            0
        }
        window_message::WM_GETFONT => state_store()
            .lock()
            .unwrap()
            .get(&hwnd)
            .map(|state| state.font)
            .unwrap_or(0) as isize,
        window_message::WM_LBUTTONDOWN => {
            update_state(hwnd, |state| state.pressed = true);
            let _ = invalidate_rect(hwnd, None, BOOL::FALSE);
            0
        }
        window_message::WM_LBUTTONUP => {
            update_state(hwnd, |state| state.pressed = false);
            let _ = invalidate_rect(hwnd, None, BOOL::FALSE);
            0
        }
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
            state_store().lock().unwrap().remove(&hwnd);
            0
        }
        _ => 0,
    }
}
