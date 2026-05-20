use rine_common_user32::menu as common;
use rine_types::errors::BOOL;
use rine_types::handles::HMENU;
use rine_types::windows::HWND;

/// Checks or unchecks a menu item (64-bit `win64` ABI).
///
/// See the [common `check_menu_item`](common::check_menu_item) for full documentation.
///
/// # Arguments
/// * `handle_menu` - A handle to the menu that contains the item.
/// * `id_check_item` - The identifier or position of the menu item.
/// * `check` - Combination of `MF_CHECKED`/`MF_UNCHECKED` and
///   `MF_BYCOMMAND`/`MF_BYPOSITION`.
///
/// # Returns
/// The previous state as an `i32`: `0` (unchecked), `1` (checked), or
/// `-1` (item not found).
///
/// # Safety
/// See common `check_menu_item`.
///
/// # Notes
/// Currently stubbed — always returns `-1` (`NotAMenuItem`).
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CheckMenuItem(
    handle_menu: HMENU,
    id_check_item: u32,
    check: u32,
) -> i32 {
    common::check_menu_item(handle_menu, id_check_item, check) as i32
}

/// Retrieves a handle to the submenu activated by the specified menu item.
///
/// # Arguments
/// * `_handle_menu` - A handle to the menu that contains the submenu.
/// * `_position` - The zero-based index position of the menu item that activates the submenu.
///
/// # Safety
/// `_handle_menu` must be a valid handle to a menu, and `_position` must correspond to a valid
/// menu item that has an associated submenu. The caller must ensure that the menu structure is
/// properly initialized and that the specified position is within bounds.
///
/// # Returns
/// A handle to the submenu if the specified menu item has an associated submenu,
/// or `HMENU::NULL` (0) if the menu item does not have a submenu or if the specified position is invalid.
///
/// # Notes
/// This function is currently a stub and returns `HMENU::NULL` unconditionally.
/// Full implementation requires tracking a menu-item tree with submenu associations per `HMENU`.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetSubMenu(_handle_menu: HMENU, _position: u32) -> HMENU {
    common::get_sub_menu(_handle_menu, _position).unwrap_or(HMENU::NULL)
}

/// Retrieves a handle to the menu assigned to the specified window.
///
/// # Arguments
/// * `handle_window` - A handle to the window whose menu handle is to be retrieved.
///
/// # Safety
/// `handle_window` must be a valid handle to a window. The caller must ensure that the window
/// structure is properly initialized.
///
/// # Returns
/// A handle to the menu assigned to the specified window, or `HMENU::NULL` (0) if the window
/// does not have a menu or if the specified window handle is invalid.
/// If the window is a child window, the return value is undefined.
///
/// # Notes
/// The menu handle is read from `WindowState::menu`. Currently, `CreateWindowEx` and `SetMenu`
/// do not populate this field, so this function will typically return `HMENU::NULL` until those
/// code paths are wired up.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetMenu(handle_window: HWND) -> HMENU {
    common::get_menu(handle_window).unwrap_or(HMENU::NULL)
}

/// Enables the application to access the window menu (also known as the system menu or
/// control menu) for copying and modifying.
///
/// # Arguments
/// * `handle_window` - A handle to the window whose system menu is to be accessed.
/// * `revert` - A boolean value that specifies whether to reset the system menu to its
///   default state. If `true`, the system menu is reset to the default; if `false`, the
///   current system menu is returned. Currently, `revert` is ignored and behaves as `false`.
///
/// # Safety
/// `handle_window` must be a valid handle to a window. The caller must ensure that the window
/// structure is properly initialized.
///
/// # Returns
/// A handle to the system menu if the specified window has a system menu, or `HMENU::NULL` (0)
/// if the window does not have a system menu or if the specified window handle is invalid.
/// If the window is a child window, the return value is undefined.
///
/// # Notes
/// The system menu handle is read from `WindowState::system_menu`. The system menu is not
/// automatically created for new top-level windows, so this function will typically return
/// `HMENU::NULL`. When `revert` is `true`, the call is delegated to the common implementation
/// which logs a warning and returns `HMENU::NULL`.
#[rine_dlls::partial]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetSystemMenu(handle_window: HWND, revert: BOOL) -> HMENU {
    common::get_system_menu(handle_window, revert.is_true()).unwrap_or(HMENU::NULL)
}

/// Enables, disables, or grays out a menu item (64-bit `win64` ABI).
///
/// See the [common `enable_menu_item`](common::enable_menu_item) for full documentation.
///
/// # Arguments
/// * `handle_menu` - A handle to the menu that contains the item.
/// * `id_enable_item` - The identifier or position of the menu item.
/// * `enable` - Combination of `MF_ENABLED`/`MF_DISABLED`/`MF_GRAYED` and
///   `MF_BYCOMMAND`/`MF_BYPOSITION`.
///
/// # Returns
/// `BOOL::TRUE` on success, `BOOL::FALSE` on failure.
///
/// # Safety
/// See common `enable_menu_item`.
///
/// # Notes
/// Currently stubbed — always returns `BOOL::FALSE`.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn EnableMenuItem(
    handle_menu: HMENU,
    id_enable_item: u32,
    enable: u32,
) -> BOOL {
    common::enable_menu_item(handle_menu, id_enable_item, enable)
}
