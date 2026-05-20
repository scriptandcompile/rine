use std::sync::Mutex;
use tracing::warn;

/// Standard predefined system cursor resource IDs (IDC_* values).
///
/// These are used with `LoadCursor`/`LoadCursorA`/`LoadCursorW` when
/// `hinstance` is `0`. The numeric values match the Win32 `IDC_*` constants
/// passed via the `MAKEINTRESOURCE` macro.
pub mod system_cursors {
    /// Standard arrow cursor.
    pub const IDC_ARROW: u16 = 32512;
    /// I-beam cursor (text selection).
    pub const IDC_IBEAM: u16 = 32513;
    /// Hourglass cursor (wait).
    pub const IDC_WAIT: u16 = 32514;
    /// Crosshair cursor.
    pub const IDC_CROSS: u16 = 32515;
    /// Up arrow cursor.
    pub const IDC_UPARROW: u16 = 32516;
    /// Obsolete — applications should use `IDC_SIZEALL`.
    pub const IDC_SIZE: u16 = 32640;
    /// Obsolete — applications should use `IDC_SIZEALL`.
    pub const IDC_ICON: u16 = 32641;
    /// Double-pointed arrow cursor (northwest-southeast).
    pub const IDC_SIZENWSE: u16 = 32642;
    /// Double-pointed arrow cursor (northeast-southwest).
    pub const IDC_SIZENESW: u16 = 32643;
    /// Double-pointed arrow cursor (west-east).
    pub const IDC_SIZEWE: u16 = 32644;
    /// Double-pointed arrow cursor (north-south).
    pub const IDC_SIZENS: u16 = 32645;
    /// Four-pointed arrow cursor (move).
    pub const IDC_SIZEALL: u16 = 32646;
    /// Slashed circle cursor (not allowed).
    pub const IDC_NO: u16 = 32648;
    /// Hand cursor (pointing hand).
    pub const IDC_HAND: u16 = 32649;
    /// Arrow with small hourglass (application starting).
    pub const IDC_APPSTARTING: u16 = 32650;
    /// Arrow with question mark (help).
    pub const IDC_HELP: u16 = 32651;
    /// Pin cursor.
    pub const IDC_PIN: u16 = 32671;
    /// Person cursor.
    pub const IDC_PERSON: u16 = 32672;
}

/// Determines whether `name` is a resource ID string (formatted as `#<id>`).
///
/// In the wrapper layer, integer resource identifiers (MAKEINTRESOURCE) are
/// converted to `"#<id>"` strings before being passed to this function.
/// This helper detects those strings and extracts the numeric ID.
///
/// # Returns
/// `Some(u16)` if `name` starts with `"#"` followed by a non-empty decimal
/// number. Returns `None` otherwise.
fn parse_resource_id(name: &str) -> Option<u16> {
    let stripped = name.strip_prefix('#')?;
    if stripped.is_empty() {
        return None;
    }
    stripped.parse::<u16>().ok()
}

/// Loads the specified cursor resource, returning a handle to the cursor if successful.
///
/// # Arguments
/// * `hinstance` - A handle to the instance of the module whose executable file contains the
///   cursor to be loaded. If this parameter is `0`, the function loads a system predefined
///   cursor identified by the numeric resource ID embedded in `name`.
/// * `name` - The name of the cursor resource to be loaded. When called from the A/W wrapper
///   layer, integer resource identifiers (MAKEINTRESOURCE) are converted to `"#<id>"` strings.
///   For example, `IDC_ARROW` (32512) is passed as `"#32512"`.
///
/// # Safety
/// If `hinstance` is not `0`, it must be a valid handle to an instance of a module that
/// contains the cursor resource specified by `name`. The caller must ensure that the module
/// is properly loaded and that the cursor resource is correctly defined within the module.
/// If `hinstance` is `0`, the caller must ensure that `name` corresponds to a valid predefined
/// cursor name or integer identifier.
/// The caller must also ensure that the returned cursor handle is properly managed and released
/// when no longer needed to avoid resource leaks.
///
/// # Returns
/// An `Option<u32>` containing the handle to the loaded cursor if the operation was successful,
/// or `None` if the function fails to load the specified cursor (for example, if the specified
/// cursor resource does not exist or if the module handle is invalid).
///
/// # Notes
/// This function is currently a stub and returns `None` as a placeholder.
/// Resource IDs (`"#<id>"` format) are correctly parsed but no system or module cursors
/// are actually loaded. The function does not set `GetLastError` on failure.
/// The following operations are **not yet implemented**:
/// - Loading system predefined cursors (hinstance = 0) from the runtime environment
/// - Loading cursor resources from module PE resources
/// - Reference counting or reuse of already-loaded cursors
pub fn load_cursor(hinstance: u32, name: &str) -> Option<u32> {
    let _ = hinstance;

    if let Some(res_id) = parse_resource_id(name) {
        warn!(
            "load_cursor is not implemented yet (resource ID {}). Returning None as a placeholder.",
            res_id
        );
    } else {
        warn!(
            "load_cursor is not implemented yet (resource name '{}'). Returning None as a placeholder.",
            name
        );
    }

    None
}

/// Sets the cursor shape, returning the handle to the previous cursor if successful.
///
/// # Arguments
/// * `cursor` - A handle to the cursor to be set.
///   If this parameter is `0`, the function removes the cursor from the screen
///   (the cursor is hidden until the next mouse movement or a subsequent `SetCursor` call).
///
/// # Safety
/// The cursor must have been created by either the `CreateCursor` function or the
/// `CreateIconIndirect` function, or loaded by either the `LoadCursor` function or
/// the `LoadImage` function.
///
/// # Returns
/// An `Option<u32>` containing the handle to the previous cursor if the operation was
/// successful, or `None` if the function fails to set the specified cursor (for example,
/// if the specified cursor handle is invalid).
///
/// # Notes
/// This function is currently a stub and returns `None` as a placeholder.
pub fn set_cursor(cursor: u32) -> Option<u32> {
    warn!(
        "set_cursor is not implemented yet. Returning None as a placeholder (cursor={}).",
        cursor
    );
    None
}

/// Global cursor position store used by `get_cursor_pos` and `set_cursor_pos`.
///
/// This provides a best-effort in-memory cursor position. The position is initially
/// `None` (unknown) until `set_cursor_pos` is called, meaning `get_cursor_pos` will
/// return `None` on the first call if the position has never been set.
static CURSOR_POSITION: Mutex<Option<(i32, i32)>> = Mutex::new(None);

/// Retrieves the position of the mouse cursor in screen coordinates.
///
/// # Safety
/// The cursor position is always specified in screen coordinates and is not affected by
/// the mapping mode of the window that contains the cursor.
/// The calling process must have `WINSTA_READATTRIBUTES` access to the window station.
/// The input desktop must be the current desktop when you call this function.
/// If the input desktop is not the current desktop, the function fails and returns `None`.
///
/// # Returns
/// An `Option<(i32, i32)>` containing the x and y coordinates of the cursor in screen
/// coordinates if the operation was successful, or `None` if the function fails to retrieve
/// the cursor position (for example, if the calling process does not have the required
/// access to the window station or if the input desktop is not the current desktop).
///
/// # Notes
/// This function returns a best-effort position from the in-memory cursor store. The
/// position may not reflect the actual OS cursor position — it only reflects the last
/// value written by `set_cursor_pos`. This means `get_cursor_pos` will return `None`
/// if `set_cursor_pos` has never been called, even if the OS knows the cursor position.
/// This implementation does not set the `GetLastError` value on failure.
/// The following operations are **not yet implemented**:
/// - Querying the actual OS cursor position through the windowing system (X11/Wayland)
/// - Tracking cursor movement from input events when the position is changed externally
pub fn get_cursor_pos() -> Option<(i32, i32)> {
    *CURSOR_POSITION.lock().unwrap()
}

/// Moves the mouse cursor to the specified screen coordinates.
///
/// # Arguments
/// * `x` - The new x-coordinate of the cursor, in screen coordinates.
/// * `y` - The new y-coordinate of the cursor, in screen coordinates.
///
/// # Returns
/// `true` if the cursor was moved successfully, or `false` if the function fails.
///
/// # Notes
/// This function stores the requested position in memory so that `get_cursor_pos`
/// can retrieve it. The position is **not** propagated to the underlying windowing
/// system, meaning the actual OS cursor does not move. Applications that rely on
/// the OS cursor position for input will not see the effect of this call.
/// This implementation does not set the `GetLastError` value on failure.
/// The following operations are **not yet implemented**:
/// - Moving the actual OS cursor through the windowing system (X11/Wayland)
/// - Input injection or synthesis of mouse move events
/// - Validating that the coordinates lie within a valid display monitor
pub fn set_cursor_pos(x: i32, y: i32) -> bool {
    *CURSOR_POSITION.lock().unwrap() = Some((x, y));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    // ── resource ID parsing ────────────────────────────────────

    #[test]
    fn parse_resource_id_valid_number() {
        assert_eq!(parse_resource_id("#32512"), Some(32512));
    }

    #[test]
    fn parse_resource_id_zero() {
        assert_eq!(parse_resource_id("#0"), Some(0));
    }

    #[test]
    fn parse_resource_id_max_u16() {
        assert_eq!(parse_resource_id("#65535"), Some(65535));
    }

    #[test]
    fn parse_resource_id_missing_hash_returns_none() {
        assert_eq!(parse_resource_id("32512"), None);
    }

    #[test]
    fn parse_resource_id_hash_only_returns_none() {
        assert_eq!(parse_resource_id("#"), None);
    }

    #[test]
    fn parse_resource_id_non_numeric_returns_none() {
        assert_eq!(parse_resource_id("#abc"), None);
    }

    #[test]
    fn parse_resource_id_overflow_returns_none() {
        assert_eq!(parse_resource_id("#70000"), None);
    }

    #[test]
    fn parse_resource_id_negative_returns_none() {
        assert_eq!(parse_resource_id("#-1"), None);
    }

    // ── load_cursor stub ────────────────────────────────────────

    #[test]
    fn load_cursor_returns_none_for_resource_name() {
        assert!(load_cursor(0x00400000, "MAINCURSOR").is_none());
    }

    #[test]
    fn load_cursor_returns_none_for_predefined_id() {
        assert!(load_cursor(0, "#32512").is_none());
    }

    #[test]
    fn load_cursor_returns_none_for_numeric_id_as_string() {
        assert!(load_cursor(0x00400000, "#101").is_none());
    }

    #[test]
    fn load_cursor_returns_none_for_zero_hinstance_and_arrow() {
        assert!(load_cursor(0, "#32512").is_none());
    }

    // ── set_cursor stub ─────────────────────────────────────────

    #[test]
    fn set_cursor_returns_none() {
        assert!(set_cursor(0).is_none());
    }

    // ── get_cursor_pos / set_cursor_pos ─────────────────────────
    //
    // These tests share the global CURSOR_POSITION state and must run
    // serially to avoid interfering with each other.

    #[test]
    #[serial]
    fn get_cursor_pos_returns_last_set_position() {
        assert!(set_cursor_pos(100, 200));
        assert_eq!(get_cursor_pos(), Some((100, 200)));
    }

    #[test]
    #[serial]
    fn get_cursor_pos_returns_overwritten_position() {
        set_cursor_pos(10, 20);
        set_cursor_pos(30, 40);
        assert_eq!(get_cursor_pos(), Some((30, 40)));
    }

    #[test]
    #[serial]
    fn get_cursor_pos_returns_negative_coordinates() {
        set_cursor_pos(-1, -1);
        assert_eq!(get_cursor_pos(), Some((-1, -1)));
    }

    #[test]
    #[serial]
    fn set_cursor_pos_returns_true() {
        assert!(set_cursor_pos(0, 0));
    }

    #[test]
    #[serial]
    fn set_cursor_pos_zero_coordinates() {
        assert!(set_cursor_pos(0, 0));
        assert_eq!(get_cursor_pos(), Some((0, 0)));
    }
}
