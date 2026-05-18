use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};

use goblin::pe::PE;
use rine_types::handles::{HACCEL, HANDLE, HINSTANCE, HandleEntry, handle_table};
use rine_types::windows::{
    Accel, HWND, Msg, WINDOW_MANAGER, accel_flags, virtual_key, window_message,
};

const RT_ACCELERATOR: u32 = 9;
const RES_DIR_SIZE: usize = 16;
const RES_ENTRY_SIZE: usize = 8;
const RES_DATA_ENTRY_SIZE: usize = 16;
const ACCEL_TABLE_ENTRY_SIZE: usize = 8;

static ACCELERATOR_CACHE: LazyLock<Mutex<HashMap<(usize, u16), HACCEL>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static SHIFT_DOWN: AtomicBool = AtomicBool::new(false);
static CONTROL_DOWN: AtomicBool = AtomicBool::new(false);
static ALT_DOWN: AtomicBool = AtomicBool::new(false);

/// Updates the global modifier-key state snapshot used by accelerator matching.
///
/// # Arguments
/// * `shift` - Whether Shift is currently pressed.
/// * `control` - Whether Control is currently pressed.
/// * `alt` - Whether Alt is currently pressed.
///
/// # Notes
/// This stores only Shift/Control/Alt state for accelerator matching.
/// It does not provide full Win32 keyboard-state API semantics.
pub fn update_modifier_state(shift: bool, control: bool, alt: bool) {
    SHIFT_DOWN.store(shift, Ordering::Relaxed);
    CONTROL_DOWN.store(control, Ordering::Relaxed);
    ALT_DOWN.store(alt, Ordering::Relaxed);
}

/// Updates modifier-key state from a key event.
///
/// # Arguments
/// * `vk` - Virtual-key code from the key event.
/// * `pressed` - True when the key is pressed, false when released.
///
/// # Notes
/// Only Shift/Control/Alt state is tracked.
pub fn update_modifier_key(vk: u16, pressed: bool) {
    match vk {
        virtual_key::VK_SHIFT => SHIFT_DOWN.store(pressed, Ordering::Relaxed),
        virtual_key::VK_CONTROL => CONTROL_DOWN.store(pressed, Ordering::Relaxed),
        virtual_key::VK_MENU => ALT_DOWN.store(pressed, Ordering::Relaxed),
        _ => {}
    }
}

/// Returns whether Alt is currently considered pressed.
pub fn is_alt_down() -> bool {
    ALT_DOWN.load(Ordering::Relaxed)
}

/// Loads an accelerator table from a module resource and returns an accelerator handle.
///
/// # Arguments
/// * `instance` - Module handle associated with the resource lookup.
/// * `resource_id` - Integer resource identifier for the accelerator table.
///
/// # Returns
/// A valid accelerator handle on success, or `HACCEL::NULL` if the table cannot be loaded.
///
/// # Notes
/// Missing implementation features:
/// - Only integer resource IDs are supported.
/// - Only the main guest executable module is currently inspected for resources.
/// - Name-based module lookup via `instance` is not implemented yet.
/// - Non-`FVIRTKEY` entries are skipped and currently unsupported.
/// - `FNOINVERT` visual behavior is not implemented.
/// - Unsupported hosted-window keyboard bridge paths are not implemented.
pub fn load_accelerators(instance: HINSTANCE, resource_id: u16) -> HACCEL {
    if resource_id == 0 {
        return HACCEL::NULL;
    }

    let cache_key = (instance.as_raw(), resource_id);
    if let Some(existing) = ACCELERATOR_CACHE.lock().unwrap().get(&cache_key).copied() {
        return existing;
    }

    let Some(entries) = load_accelerator_entries_from_main_module(resource_id) else {
        return HACCEL::NULL;
    };

    let handle = handle_table().insert(HandleEntry::AcceleratorTable(entries));
    let haccel = HACCEL::from_raw(handle.as_raw() as usize);

    ACCELERATOR_CACHE.lock().unwrap().insert(cache_key, haccel);
    haccel
}

/// Translates a key message by matching it against an accelerator table.
///
/// # Arguments
/// * `hwnd` - Frame window handle for command dispatch.
/// * `haccel` - Accelerator table handle returned by `LoadAccelerators`.
/// * `msg` - Pointer to the message being translated.
/// * `call_wnd_proc` - Callback used to invoke the target window procedure.
///
/// # Safety
/// `msg` must point to a valid `Msg` when non-null.
/// `call_wnd_proc` must safely invoke the provided window procedure pointer.
///
/// # Returns
/// Returns `1` when an accelerator match is translated to `WM_COMMAND`, else `0`.
///
/// # Notes
/// Missing implementation features:
/// - Matching currently supports only `FVIRTKEY` entries.
/// - Character/dead-key/layout-aware accelerator translation is not implemented.
/// - `FNOINVERT` behavior is ignored.
/// - Hosted-window keyboard event translation is not implemented.
pub unsafe fn translate_accelerator(
    hwnd: HWND,
    haccel: HACCEL,
    msg: *const Msg,
    call_wnd_proc: impl Fn(usize, HWND, u32, usize, isize) -> isize,
) -> i32 {
    if hwnd.is_null() || haccel.is_null() || msg.is_null() {
        return 0;
    }

    let msg = unsafe { &*msg };
    let is_key_message =
        msg.message == window_message::WM_KEYDOWN || msg.message == window_message::WM_SYSKEYDOWN;
    if !is_key_message {
        return 0;
    }

    if !is_window_or_descendant(msg.hwnd, hwnd) {
        return 0;
    }

    let Some(table_entries) = handle_table()
        .with_accelerator_table(HANDLE::from_raw(haccel.as_raw() as isize), |entries| {
            entries.to_vec()
        })
    else {
        return 0;
    };

    let current_mods = current_modifier_bits();
    let vk = msg.w_param as u16;

    for entry in &table_entries {
        if (entry.f_virt & accel_flags::FVIRTKEY) == 0 {
            continue;
        }

        let required_mods =
            entry.f_virt & (accel_flags::FSHIFT | accel_flags::FCONTROL | accel_flags::FALT);
        if entry.key == vk && required_mods == current_mods {
            let Some(target_state) = WINDOW_MANAGER.get_window(hwnd) else {
                return 0;
            };

            let w_param = (entry.cmd as usize) | (1usize << 16);
            let _ = call_wnd_proc(
                target_state.wnd_proc,
                hwnd,
                window_message::WM_COMMAND,
                w_param,
                0,
            );
            return 1;
        }
    }

    0
}

fn current_modifier_bits() -> u8 {
    let mut bits = 0u8;
    if SHIFT_DOWN.load(Ordering::Relaxed) {
        bits |= accel_flags::FSHIFT;
    }
    if CONTROL_DOWN.load(Ordering::Relaxed) {
        bits |= accel_flags::FCONTROL;
    }
    if ALT_DOWN.load(Ordering::Relaxed) {
        bits |= accel_flags::FALT;
    }
    bits
}

fn is_window_or_descendant(mut candidate: HWND, target: HWND) -> bool {
    if candidate.is_null() || target.is_null() {
        return false;
    }

    let mut hops = 0usize;
    while hops < 128 {
        if candidate == target {
            return true;
        }

        let Some(state) = WINDOW_MANAGER.get_window(candidate) else {
            return false;
        };

        if state.parent.is_null() {
            return false;
        }

        candidate = state.parent;
        hops += 1;
    }

    false
}

fn load_accelerator_entries_from_main_module(resource_id: u16) -> Option<Vec<Accel>> {
    let executable_path = guest_main_module_path()?;
    let bytes = std::fs::read(executable_path).ok()?;
    let pe = PE::parse(&bytes).ok()?;
    let rsrc = find_resource_section(&bytes, &pe)?;

    let (data_rva, data_size) = find_accelerator_data_entry(&rsrc, resource_id)?;
    let data = rsrc.slice_at_rva(data_rva, data_size as usize)?;

    let mut parsed = Vec::new();
    let mut skipped_non_virtual_key = 0usize;

    for chunk in data.chunks_exact(ACCEL_TABLE_ENTRY_SIZE) {
        let flags = u16::from_le_bytes([chunk[0], chunk[1]]);
        let key = u16::from_le_bytes([chunk[2], chunk[3]]);
        let cmd = u16::from_le_bytes([chunk[4], chunk[5]]);

        let f_virt = (flags & 0x00FF) as u8;
        if (f_virt & accel_flags::FVIRTKEY) == 0 {
            skipped_non_virtual_key += 1;
        } else {
            parsed.push(Accel { f_virt, key, cmd });
        }

        if (f_virt & accel_flags::FLAST) != 0 {
            break;
        }
    }

    if skipped_non_virtual_key != 0 {
        tracing::warn!(
            skipped_non_virtual_key,
            resource_id,
            "LoadAccelerators: skipped unsupported non-FVIRTKEY entries"
        );
    }

    Some(parsed)
}

fn guest_main_module_path() -> Option<std::path::PathBuf> {
    std::env::args_os()
        .nth(1)
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_exe().ok())
}

struct ResourceSection<'a> {
    data: &'a [u8],
    section_rva: u32,
    directory_offset: u32,
}

impl<'a> ResourceSection<'a> {
    fn slice_at_directory_offset(&self, offset: u32, size: usize) -> Option<&'a [u8]> {
        let base = self.directory_offset as usize;
        let start = base.checked_add(offset as usize)?;
        let end = start.checked_add(size)?;
        self.data.get(start..end)
    }

    fn slice_at_rva(&self, rva: u32, size: usize) -> Option<&'a [u8]> {
        let offset = rva.checked_sub(self.section_rva)? as usize;
        let end = offset.checked_add(size)?;
        self.data.get(offset..end)
    }
}

fn find_resource_section<'a>(bytes: &'a [u8], pe: &PE<'_>) -> Option<ResourceSection<'a>> {
    let optional_header = pe.header.optional_header.as_ref()?;
    let resource_dir = optional_header.data_directories.get_resource_table()?;

    let resource_rva = resource_dir.virtual_address;
    let resource_size = resource_dir.size as usize;
    if resource_rva == 0 || resource_size == 0 {
        return None;
    }

    for section in &pe.sections {
        let section_rva = section.virtual_address;
        let section_vsize = section.virtual_size;
        if resource_rva < section_rva || resource_rva >= section_rva.saturating_add(section_vsize) {
            continue;
        }

        let raw_offset = section.pointer_to_raw_data as usize;
        let raw_size = section.size_of_raw_data as usize;
        let raw_end = raw_offset.checked_add(raw_size)?;
        if raw_end > bytes.len() {
            return None;
        }

        return Some(ResourceSection {
            data: &bytes[raw_offset..raw_end],
            section_rva,
            directory_offset: resource_rva - section_rva,
        });
    }

    None
}

fn find_accelerator_data_entry(rsrc: &ResourceSection<'_>, resource_id: u16) -> Option<(u32, u32)> {
    let (_, type_entries) = read_resource_directory_header(rsrc, 0)?;

    for i in 0..type_entries {
        let entry_offset = RES_DIR_SIZE + i * RES_ENTRY_SIZE;
        let type_entry = rsrc.slice_at_directory_offset(entry_offset as u32, RES_ENTRY_SIZE)?;
        let type_name_or_id = read_u32(type_entry, 0)?;
        let type_data_or_dir = read_u32(type_entry, 4)?;

        if (type_name_or_id & 0x8000_0000) != 0 {
            continue;
        }
        if type_name_or_id != RT_ACCELERATOR {
            continue;
        }
        if (type_data_or_dir & 0x8000_0000) == 0 {
            return None;
        }

        let name_level_dir = type_data_or_dir & 0x7FFF_FFFF;
        let (_, name_entries) = read_resource_directory_header(rsrc, name_level_dir)?;

        for j in 0..name_entries {
            let entry_offset = name_level_dir as usize + RES_DIR_SIZE + j * RES_ENTRY_SIZE;
            let entry = rsrc.slice_at_directory_offset(entry_offset as u32, RES_ENTRY_SIZE)?;
            let name_or_id = read_u32(entry, 0)?;
            let data_or_dir = read_u32(entry, 4)?;

            if (name_or_id & 0x8000_0000) != 0 {
                continue;
            }
            if name_or_id != u32::from(resource_id) {
                continue;
            }
            if (data_or_dir & 0x8000_0000) == 0 {
                return read_data_entry(rsrc, data_or_dir);
            }

            let lang_level_dir = data_or_dir & 0x7FFF_FFFF;
            let (_, lang_entries) = read_resource_directory_header(rsrc, lang_level_dir)?;

            for k in 0..lang_entries {
                let leaf_offset = lang_level_dir as usize + RES_DIR_SIZE + k * RES_ENTRY_SIZE;
                let leaf = rsrc.slice_at_directory_offset(leaf_offset as u32, RES_ENTRY_SIZE)?;
                let leaf_data_or_dir = read_u32(leaf, 4)?;
                if (leaf_data_or_dir & 0x8000_0000) != 0 {
                    continue;
                }
                return read_data_entry(rsrc, leaf_data_or_dir);
            }
        }
    }

    None
}

fn read_data_entry(rsrc: &ResourceSection<'_>, offset: u32) -> Option<(u32, u32)> {
    let raw = rsrc.slice_at_directory_offset(offset, RES_DATA_ENTRY_SIZE)?;
    let rva = read_u32(raw, 0)?;
    let size = read_u32(raw, 4)?;
    Some((rva, size))
}

fn read_resource_directory_header(
    rsrc: &ResourceSection<'_>,
    offset: u32,
) -> Option<(usize, usize)> {
    let raw = rsrc.slice_at_directory_offset(offset, RES_DIR_SIZE)?;
    let named = read_u16(raw, 12)? as usize;
    let ids = read_u16(raw, 14)? as usize;
    let total = named.checked_add(ids)?;
    Some((named, total))
}

fn read_u16(raw: &[u8], offset: usize) -> Option<u16> {
    let bytes = raw.get(offset..offset + 2)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_u32(raw: &[u8], offset: usize) -> Option<u32> {
    let bytes = raw.get(offset..offset + 4)?;
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn test_window_state(parent: HWND, wnd_proc: usize) -> rine_types::windows::WindowState {
        rine_types::windows::WindowState {
            hwnd: HWND::NULL,
            class_name: "test-class".to_owned(),
            title: "test".to_owned(),
            style: 0,
            ex_style: 0,
            rect: rine_types::windows::Rect::default(),
            client_rect: rine_types::windows::Rect::default(),
            parent,
            visible: true,
            enabled: true,
            wnd_proc,
            user_data: 0,
        }
    }

    fn insert_test_accelerator_table(entries: Vec<Accel>) -> HACCEL {
        let h = handle_table().insert(HandleEntry::AcceleratorTable(entries));
        HACCEL::from_raw(h.as_raw() as usize)
    }

    #[test]
    fn translate_accelerator_generates_wm_command_for_matching_entry() {
        let _guard = TEST_MUTEX.lock().unwrap();

        update_modifier_state(false, true, false);

        let frame = WINDOW_MANAGER.create_window(test_window_state(HWND::NULL, 0x1111));
        let haccel = insert_test_accelerator_table(vec![Accel {
            f_virt: accel_flags::FVIRTKEY | accel_flags::FCONTROL,
            key: virtual_key::VK_S,
            cmd: 0x00F1,
        }]);

        let msg = Msg {
            hwnd: frame,
            message: window_message::WM_KEYDOWN,
            w_param: virtual_key::VK_S as usize,
            l_param: 0,
            time: 0,
            pt: rine_types::windows::Point::default(),
        };

        let seen = Mutex::new(None);
        let translated = unsafe {
            translate_accelerator(frame, haccel, &msg, |proc_fn, hwnd, m, wp, lp| {
                *seen.lock().unwrap() = Some((proc_fn, hwnd, m, wp, lp));
                0
            })
        };

        assert_eq!(translated, 1);
        let Some((proc_fn, dispatch_hwnd, message, w_param, l_param)) = seen.lock().unwrap().take()
        else {
            panic!("expected translated WM_COMMAND");
        };

        assert_eq!(proc_fn, 0x1111);
        assert_eq!(dispatch_hwnd, frame);
        assert_eq!(message, window_message::WM_COMMAND);
        assert_eq!(w_param & 0xFFFF, 0x00F1);
        assert_eq!((w_param >> 16) & 0xFFFF, 1);
        assert_eq!(l_param, 0);

        WINDOW_MANAGER.destroy_window(frame);
        update_modifier_state(false, false, false);
    }

    #[test]
    fn translate_accelerator_returns_zero_when_no_match() {
        let _guard = TEST_MUTEX.lock().unwrap();

        update_modifier_state(false, false, false);

        let frame = WINDOW_MANAGER.create_window(test_window_state(HWND::NULL, 0x2222));
        let haccel = insert_test_accelerator_table(vec![Accel {
            f_virt: accel_flags::FVIRTKEY | accel_flags::FCONTROL,
            key: virtual_key::VK_O,
            cmd: 0x0100,
        }]);

        let msg = Msg {
            hwnd: frame,
            message: window_message::WM_KEYDOWN,
            w_param: virtual_key::VK_O as usize,
            l_param: 0,
            time: 0,
            pt: rine_types::windows::Point::default(),
        };

        let translated = unsafe {
            translate_accelerator(frame, haccel, &msg, |_proc_fn, _hwnd, _m, _wp, _lp| 0)
        };

        assert_eq!(translated, 0);

        WINDOW_MANAGER.destroy_window(frame);
    }

    #[test]
    fn translate_accelerator_accepts_child_message_target() {
        let _guard = TEST_MUTEX.lock().unwrap();

        update_modifier_state(true, false, false);

        let frame = WINDOW_MANAGER.create_window(test_window_state(HWND::NULL, 0x3333));
        let child = WINDOW_MANAGER.create_window(test_window_state(frame, 0x4444));
        let haccel = insert_test_accelerator_table(vec![Accel {
            f_virt: accel_flags::FVIRTKEY | accel_flags::FSHIFT,
            key: virtual_key::VK_F3,
            cmd: 0x0200,
        }]);

        let msg = Msg {
            hwnd: child,
            message: window_message::WM_KEYDOWN,
            w_param: virtual_key::VK_F3 as usize,
            l_param: 0,
            time: 0,
            pt: rine_types::windows::Point::default(),
        };

        let translated = unsafe {
            translate_accelerator(frame, haccel, &msg, |_proc_fn, _hwnd, _m, _wp, _lp| 0)
        };

        assert_eq!(translated, 1);

        WINDOW_MANAGER.destroy_window(child);
        WINDOW_MANAGER.destroy_window(frame);
        update_modifier_state(false, false, false);
    }
}
