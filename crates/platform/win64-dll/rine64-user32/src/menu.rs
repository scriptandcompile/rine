use rine_common_user32::menu as common;
use rine_types::errors::WinBool;

/// Checks or unchecks a menu item, returning the previous state of the item.
///
/// # Arguments
/// * `handle_menu` - A handle to the menu that contains the item to be checked or unchecked.
/// * `id_check_item` - The identifier or position of the menu item to be checked or unchecked.
/// * `check` - The action to be performed on the menu item. This parameter can be a bitwise combination of the following values:
///     - MF_BYCOMMAND (0x00000000): Indicates that `id_check_item` specifies the identifier of the menu item.
///     - MF_BYPOSITION (0x00000400): Indicates that `id_check_item` specifies the position of the menu item.
///     - MF_CHECKED (0x00000008): Checks the menu item.
///     - MF_UNCHECKED (0x00000000): Unchecks the menu item.
///
/// # Safety
/// This function is unsafe because it interacts with raw pointers.
/// The caller must ensure that the `handle_menu` is a valid handle to a menu and that the `id_check_item`
/// corresponds to a valid menu item within that menu.
/// Additionally, the caller must ensure that the `check` parameter is a valid combination of the MF_* flags.
///
/// # Returns
/// A `PreviousMenuState` indicating the previous state of the menu item before the check/uncheck operation was performed.
/// The possible return values are:
///   `PreviousMenuState::NotChecked`: The menu item was previously unchecked.
///   `PreviousMenuState::Checked`: The menu item was previously checked.
///   `PreviousMenuState::NotAMenuItem`: The specified item was not a menu item.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn CheckMenuItem(
    handle_menu: u32,
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
/// _handle_menu must be a valid handle to a menu, and _position must correspond to a valid menu item that has an associated submenu.
/// The caller must ensure that the menu structure is properly initialized and that the specified position is within bounds.
///
/// # Returns
/// An `u32` containing the handle to the submenu if the specified menu item has an associated submenu,
/// or `0` if the menu item does not have a submenu or if the specified position is invalid.
///
/// # Notes
/// This function is currently a stub and returns `0` as a placeholder.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetSubMenu(_handle_menu: u32, _position: u32) -> u32 {
    common::get_sub_menu(_handle_menu, _position).unwrap_or_default()
}

/// Retrieves a handle to the menu assigned to the specified window.
///
/// # Arguments
/// * `_handle_window` - A handle to the window whose menu handle is to be retrieved.
///
/// # Safety
/// _handle_window must be a valid handle to a window.
/// The caller must ensure that the window structure is properly initialized.
///
/// # Returns
/// An `u32` containing the handle to the menu assigned to the specified window,
/// or `0` if the window does not have a menu or if the specified window handle is invalid.
/// if the window is a child window, the return value is undefined.
///
/// # Notes
/// This function is currently a stub and returns `0` as a placeholder.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetMenu(_handle_window: u32) -> u32 {
    common::get_menu(_handle_window).unwrap_or_default()
}

/// Enables the application to access the window menu (also known as the system menu or control menu) for copying and modifying.
///
/// # Arguments
/// * `_handle_window` - A handle to the window whose system menu is to be accessed.
/// * `_revert` - A boolean value that specifies whether to reset the system menu to its default state.
///   If this parameter is `true`, the system menu will be reset to its default state.
///   If this parameter is `false`, the system menu will be returned in its current state.
///
/// # Safety
/// _handle_window must be a valid handle to a window.
/// The caller must ensure that the window structure is properly initialized.
///
/// # Returns
/// An `u32` containing the handle to the system menu if the specified window has a system menu and the handle is
/// valid, or `0` if the window does not have a system menu or if the specified window handle is invalid.
/// If the window is a child window, the return value is undefined.
///
/// # Notes
/// This function is currently a stub and returns `0` as a placeholder.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn GetSystemMenu(_handle_window: u32) -> u32 {
    common::get_system_menu(_handle_window, false).unwrap_or_default()
}

/// Enables, disables, or grays out a menu item.
///
/// # Arguments
/// * `_handle_menu` - A handle to the menu that contains the item to be enabled, disabled, or grayed out.
/// * `_id_enable_item` - The identifier or position of the menu item to be enabled, disabled, or grayed out.
/// * `_enable` - The action to be performed on the menu item. This parameter can be a bitwise combination of the following values:
///     - MF_BYCOMMAND (0x00000000): Indicates that `id_enable_item` specifies the identifier of the menu item.
///     - MF_BYPOSITION (0x00000400): Indicates that `id_enable_item` specifies the position of the menu item.
///     - MF_ENABLED (0x00000000): Enables the menu item.
///     - MF_GRAYED (0x00000001): Grays out the menu item and disables it.
///     - MF_DISABLED (0x00000002): Disables the menu item without graying it out.
///
/// # Safety
/// This function is unsafe because it interacts with raw pointers.
/// The caller must ensure that the `handle_menu` is a valid handle to a menu and that the `id_enable_item` corresponds to a valid menu item within that menu.
/// Additionally, the caller must ensure that the `enable` parameter is a valid combination of the MF_* flags.
/// The caller must also ensure that the menu structure is properly initialized and that the specified item is within bounds.
///
/// # Returns
/// A `WinBool` indicating whether the operation was successful.
/// Returns `WinBool::TRUE` if the menu item was successfully enabled, disabled, or grayed out, and `WinBool::FALSE` if the operation
/// failed (for example, if the specified menu item was invalid).
///
/// # Notes
/// This function is currently a stub and returns `WinBool::FALSE` as a placeholder.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "win64" fn EnableMenuItem(
    _handle_menu: u32,
    _id_enable_item: u32,
    _enable: u32,
) -> WinBool {
    common::enable_menu_item(_handle_menu, _id_enable_item, _enable)
}
