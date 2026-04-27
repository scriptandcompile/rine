use rine_common_user32::menu as common;

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
pub unsafe extern "stdcall" fn CheckMenuItem(
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
/// This function is currently a stub and returns `None` as a placeholder.
#[rine_dlls::stubbed]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn GetSubMenu(_handle_menu: u32, _position: u32) -> u32 {
    common::get_sub_menu(_handle_menu, _position).unwrap_or_default()
}
