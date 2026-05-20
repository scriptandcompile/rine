use rine_common_user32 as common;
use rine_types::windows::*;

/// GetDC — retrieves a device context for the client area of the specified
/// window.
///
/// The caller must release the returned HDC with `ReleaseDC` using the same
/// window handle.
///
/// # Arguments
/// * `hwnd` — Handle to the window whose client-area DC is to be retrieved.
///
/// # Safety
/// The caller must ensure that `hwnd` is either null or a valid window handle
/// that has not already been destroyed.
///
/// # Returns
/// The HDC of the device context for the client area, or `HDC::NULL` if the
/// function fails (e.g. invalid HWND).
///
/// # Notes — incomplete elements
///
/// - The returned HDC is a stub handle: it is allocated and tracked, but no
///   GDI surface is attached. Drawing into this DC via GDI functions (e.g.
///   `BitBlt`, `TextOut`) will not produce visible output.
/// - No support for `CS_OWNDC` / `CS_CLASSDC` / `CS_PARENTDC` class styles
///   that control whether `GetDC` returns a cached or fresh DC.
/// - No per-monitor DPI awareness; the DC always assumes the default DPI.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetDC(hwnd: HWND) -> HDC {
    common::get_dc(hwnd)
}

/// GetWindowDC — retrieves a device context for the entire window, including
/// its non-client area.
///
/// # Arguments
/// * `hwnd` — Handle to the window whose DC is to be retrieved.
///
/// # Safety
/// The caller must ensure that `hwnd` is either null or a valid window handle
/// that has not already been destroyed.
///
/// # Returns
/// The HDC of the device context for the entire window, or `HDC::NULL` if the
/// function fails.
///
/// # Notes — incomplete elements
///
/// - Currently behaves identically to `GetDC` — the non-client area is **not**
///   distinguished from the client area. A real implementation would return a
///   DC covering the entire window rectangle (including title bar, borders,
///   scroll bars) and post `WM_NCPAINT` when appropriate.
/// - The returned HDC is a stub handle with no GDI surface attached (same
///   limitations as `GetDC`).
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetWindowDC(hwnd: HWND) -> HDC {
    common::get_window_dc(hwnd)
}

/// ReleaseDC — releases a device context previously obtained by `GetDC` or
/// `GetWindowDC`.
///
/// # Arguments
/// * `hwnd` — Handle to the window whose DC is to be released. Must be the
///   same handle passed to the acquire call.
/// * `hdc` — Handle to the DC to be released.
///
/// # Safety
/// The caller must ensure that both `hwnd` and `hdc` are either null or valid
/// handles belonging to this runtime.
///
/// # Returns
/// 1 if the DC was released, 0 if the DC was not acquired for the given
/// window or is not a valid window-DC handle.
///
/// # Notes — incomplete elements
///
/// - Validates that the HDC was acquired for the given HWND, but does **not**
///   free any backing resources (there are none yet).
/// - `BeginPaint` / `EndPaint` are not implemented. They belong in the same
///   conceptual family and should be added alongside validation of the
///   update region (`PAINTSTRUCT`).
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn ReleaseDC(hwnd: HWND, hdc: HDC) -> i32 {
    common::release_dc(hwnd, hdc)
}
