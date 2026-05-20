//! Device-context management — GetDC / GetWindowDC / ReleaseDC shared logic.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use rine_types::errors::BOOL;
use rine_types::windows::*;

// ---------------------------------------------------------------------------
// Global window-DC state
// ---------------------------------------------------------------------------

static NEXT_WINDOW_DC_HANDLE: AtomicUsize = AtomicUsize::new(0x20000);

static WINDOW_DC_STATE: std::sync::OnceLock<Mutex<WindowDcState>> = std::sync::OnceLock::new();

fn window_dc_state() -> &'static Mutex<WindowDcState> {
    WINDOW_DC_STATE.get_or_init(|| Mutex::new(WindowDcState::new()))
}

struct WindowDcState {
    /// Maps each allocated HDC to the HWND it was acquired for.
    dc_to_window: HashMap<HDC, HWND>,
    /// Reverse lookup — every HDC currently acquired for a given window.
    window_dcs: HashMap<HWND, Vec<HDC>>,
}

impl WindowDcState {
    fn new() -> Self {
        Self {
            dc_to_window: HashMap::new(),
            window_dcs: HashMap::new(),
        }
    }

    fn allocate(&mut self, hwnd: HWND) -> HDC {
        let raw = NEXT_WINDOW_DC_HANDLE.fetch_add(1, Ordering::Relaxed);
        let hdc = HDC::from_raw(raw);

        self.dc_to_window.insert(hdc, hwnd);
        self.window_dcs.entry(hwnd).or_default().push(hdc);

        let detail = format!(r#"{{"hdc":{},"hwnd":{}}}"#, raw, hwnd.as_raw());
        rine_types::dev_notify!(on_handle_created(raw as i64, "WindowDc", &detail));

        hdc
    }

    fn release(&mut self, hwnd: HWND, hdc: HDC) -> BOOL {
        match self.dc_to_window.get(&hdc) {
            Some(&owner) if owner == hwnd => {}
            _ => return BOOL::FALSE,
        }

        self.dc_to_window.remove(&hdc);

        if let Some(dcs) = self.window_dcs.get_mut(&hwnd) {
            dcs.retain(|&h| h != hdc);
            if dcs.is_empty() {
                self.window_dcs.remove(&hwnd);
            }
        }

        rine_types::dev_notify!(on_handle_closed(hdc.as_raw() as i64));
        BOOL::TRUE
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// GetDC — retrieves a device context (DC) for the client area of the
/// specified window.
///
/// The returned DC can be used for GDI drawing. The caller **must** call
/// `release_dc` with the same `hwnd` when drawing is complete.
///
/// # Arguments
/// * `hwnd` — Handle to the window whose client-area DC is to be retrieved.
///
/// # Safety
/// This function is safe to call, but the caller must ensure that `hwnd` is
/// either null or a valid window handle that has not already been destroyed.
///
/// # Returns
/// The `HDC` for the client area of the specified window, or `HDC::NULL` if
/// `hwnd` is null or the window is not recognised.
///
/// # Notes — incomplete elements
///
/// - The returned `HDC` is a **stub handle**: it is allocated and tracked, but
///   no actual GDI surface is attached. Drawing into this DC via GDI functions
///   (e.g. `BitBlt`, `TextOut`) will **not** produce visible output on the
///   window. A future implementation should either (a) draw into the winit
///   window's native surface, or (b) composite into an internal buffer and
///   blit it to the window on `ReleaseDC` / `EndPaint`.
/// - No support for `CS_OWNDC` / `CS_CLASSDC` / `CS_PARENTDC` class styles.
///   On Windows these control whether `GetDC` returns a cached or fresh DC.
/// - No per-monitor DPI awareness; the DC always assumes the default DPI.
pub fn get_dc(hwnd: HWND) -> HDC {
    if hwnd.is_null() {
        return HDC::NULL;
    }

    if WINDOW_MANAGER.get_window(hwnd).is_none() {
        return HDC::NULL;
    }

    let mut state = window_dc_state().lock().unwrap();
    state.allocate(hwnd)
}

/// GetWindowDC — retrieves a DC for the **entire** window, including its
/// non-client area (title bar, borders, menu bar, scroll bars).
///
/// # Arguments
/// * `hwnd` — Handle to the window whose DC is to be retrieved.
///
/// # Safety
/// This function is safe to call, but the caller must ensure that `hwnd` is
/// either null or a valid window handle that has not already been destroyed.
///
/// # Returns
/// The `HDC` for the entire window, or `HDC::NULL` if `hwnd` is null or
/// unrecognised.
///
/// # Notes — incomplete elements
///
/// - Currently behaves identically to `get_dc` — the non-client area is **not**
///   distinguished from the client area. A real implementation would return a
///   DC that covers the entire window rectangle (including title bar, borders,
///   scroll bars) and post `WM_NCPAINT` when appropriate.
/// - The returned `HDC` is a stub handle with no GDI surface attached (same
///   limitations as `get_dc`).
pub fn get_window_dc(hwnd: HWND) -> HDC {
    get_dc(hwnd)
}

/// ReleaseDC — releases a DC previously acquired by `get_dc` or
/// `get_window_dc`.
///
/// # Arguments
/// * `hwnd` — Handle to the window whose DC is to be released. Must be the
///   same handle passed to the acquire call.
/// * `hdc` — Handle to the DC to be released.
///
/// # Safety
/// This function is safe to call, but the caller must ensure that both
/// `hwnd` and `hdc` are either null or valid handles belonging to this
/// runtime.
///
/// # Returns
/// 1 on success, 0 if the DC was not acquired for the given window or is
/// not a valid window-DC handle.
///
/// # Notes — incomplete elements
///
/// - Validates that the `HDC` was acquired for the given `HWND`, but does
///   **not** free any backing resources (there are none yet).
/// - `BeginPaint` / `EndPaint` are not implemented. They belong in the same
///   conceptual family and should be added alongside a real validation of the
///   update region (`PAINTSTRUCT`).
pub fn release_dc(hwnd: HWND, hdc: HDC) -> i32 {
    if hwnd.is_null() || hdc.is_null() {
        return 0;
    }

    let mut state = window_dc_state().lock().unwrap();
    if state.release(hwnd, hdc).is_true() {
        1
    } else {
        0
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn test_window_state() -> WindowState {
        WindowState {
            hwnd: HWND::NULL,
            class_name: "test-dc".to_owned(),
            title: "".to_owned(),
            style: 0,
            ex_style: 0,
            rect: Rect::default(),
            client_rect: Rect::default(),
            parent: HWND::NULL,
            visible: true,
            enabled: true,
            wnd_proc: 0,
            user_data: 0,
        }
    }

    #[test]
    fn get_dc_returns_null_for_null_hwnd() {
        assert_eq!(get_dc(HWND::NULL), HDC::NULL);
    }

    #[test]
    fn get_dc_returns_null_for_invalid_hwnd() {
        let bogus = HWND::from_raw(0xDEAD);
        assert_eq!(get_dc(bogus), HDC::NULL);
    }

    #[test]
    fn get_dc_and_release_dc_roundtrip() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        assert!(!hwnd.is_null());

        let hdc = get_dc(hwnd);
        assert!(!hdc.is_null());

        assert_eq!(release_dc(hwnd, hdc), 1);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn release_dc_fails_with_wrong_hwnd() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let a = WINDOW_MANAGER.create_window(test_window_state());
        let b = WINDOW_MANAGER.create_window(test_window_state());

        let hdc = get_dc(a);
        assert!(!hdc.is_null());

        assert_eq!(release_dc(b, hdc), 0);
        assert_eq!(release_dc(a, hdc), 1);

        WINDOW_MANAGER.destroy_window(a);
        WINDOW_MANAGER.destroy_window(b);
    }

    #[test]
    fn release_dc_fails_with_null_params() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());
        assert_eq!(release_dc(hwnd, HDC::NULL), 0);
        assert_eq!(release_dc(HWND::NULL, HDC::NULL), 0);
        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn multiple_dcs_per_window() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());

        let a = get_dc(hwnd);
        let b = get_dc(hwnd);
        assert!(!a.is_null());
        assert!(!b.is_null());
        assert_ne!(a, b);

        assert_eq!(release_dc(hwnd, a), 1);
        assert_eq!(release_dc(hwnd, b), 1);

        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_window_dc_behaves_like_get_dc() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());

        let hdc = get_window_dc(hwnd);
        assert!(!hdc.is_null());
        assert_eq!(release_dc(hwnd, hdc), 1);

        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn release_dc_fails_for_double_release() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let hwnd = WINDOW_MANAGER.create_window(test_window_state());

        let hdc = get_dc(hwnd);
        assert_eq!(release_dc(hwnd, hdc), 1);
        assert_eq!(release_dc(hwnd, hdc), 0);

        WINDOW_MANAGER.destroy_window(hwnd);
    }

    #[test]
    fn get_window_dc_returns_null_for_null_hwnd() {
        assert_eq!(get_window_dc(HWND::NULL), HDC::NULL);
    }

    #[test]
    fn get_window_dc_returns_null_for_invalid_hwnd() {
        assert_eq!(get_window_dc(HWND::from_raw(0xDEAD)), HDC::NULL);
    }
}
