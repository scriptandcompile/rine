use core::sync::atomic::{AtomicU32, Ordering};
use std::path::PathBuf;

use crate::env_policy::{DialogPolicy, DialogTheme, resolve_dialog_policy};

/// Common dialog extended error codes (subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DialogErrorCode {
    None = 0,
    CderrInitialization = 0x0002,
    CderrDialogFailure = 0xFFFF,
    FnerrBufferTooSmall = 0x3003,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    Open,
    Save,
}

pub trait DialogAdapter {
    fn struct_size_valid(&self) -> bool;
    fn title(&self) -> Option<String>;
    fn initial_dir(&self) -> Option<String>;
    fn has_output_buffer(&self) -> bool;
    fn write_selected_path(&mut self, path: &str) -> Result<(), DialogErrorCode>;
    fn set_name_offsets(&mut self, file_offset: u16, file_extension: u16);
}

static LAST_ERROR: AtomicU32 = AtomicU32::new(DialogErrorCode::None as u32);

pub fn last_error() -> u32 {
    LAST_ERROR.load(Ordering::Relaxed)
}

pub fn set_last_error(code: DialogErrorCode) {
    LAST_ERROR.store(code as u32, Ordering::Relaxed);
}

pub fn update_offsets(path: &str) -> (u16, u16) {
    let file_offset = path
        .rfind(['/', '\\'])
        .map(|idx| idx + 1)
        .unwrap_or(0)
        .min(u16::MAX as usize) as u16;

    let file_extension = path[file_offset as usize..]
        .rfind('.')
        .map(|dot| file_offset as usize + dot + 1)
        .unwrap_or(0)
        .min(u16::MAX as usize) as u16;

    (file_offset, file_extension)
}

pub fn run_dialog_flow<A, PickPath, EmitOpened, EmitResult>(
    api_name: &'static str,
    kind: DialogKind,
    mut adapter: Option<A>,
    pick_path: PickPath,
    emit_opened: EmitOpened,
    mut emit_result: EmitResult,
) -> i32
where
    A: DialogAdapter,
    PickPath: FnOnce(DialogKind, Option<String>, Option<String>) -> Option<PathBuf>,
    EmitOpened: FnOnce(&'static str, DialogPolicy),
    EmitResult: FnMut(&'static str, DialogPolicy, bool, u32, Option<&str>),
{
    let policy = resolve_dialog_policy();
    emit_opened(api_name, policy);

    if adapter.is_none() {
        set_last_error(DialogErrorCode::CderrInitialization);
        emit_result(
            api_name,
            policy,
            false,
            DialogErrorCode::CderrInitialization as u32,
            None,
        );
        return 0;
    }

    let adapter = adapter.as_mut().expect("checked is_none above");
    if !adapter.struct_size_valid() {
        set_last_error(DialogErrorCode::CderrInitialization);
        emit_result(
            api_name,
            policy,
            false,
            DialogErrorCode::CderrInitialization as u32,
            None,
        );
        return 0;
    }

    if matches!(policy.theme, DialogTheme::Windows) {
        set_last_error(DialogErrorCode::CderrDialogFailure);
        emit_result(
            api_name,
            policy,
            false,
            DialogErrorCode::CderrDialogFailure as u32,
            None,
        );
        return 0;
    }

    let chosen = pick_path(kind, adapter.title(), adapter.initial_dir());
    let Some(path) = chosen else {
        set_last_error(DialogErrorCode::None);
        emit_result(api_name, policy, false, DialogErrorCode::None as u32, None);
        return 0;
    };
    let path = path.to_string_lossy().into_owned();

    if !adapter.has_output_buffer() {
        set_last_error(DialogErrorCode::CderrInitialization);
        emit_result(
            api_name,
            policy,
            false,
            DialogErrorCode::CderrInitialization as u32,
            None,
        );
        return 0;
    }

    if let Err(code) = adapter.write_selected_path(&path) {
        set_last_error(code);
        emit_result(api_name, policy, false, code as u32, None);
        return 0;
    }

    let (off, ext) = update_offsets(&path);
    adapter.set_name_offsets(off, ext);
    set_last_error(DialogErrorCode::None);
    emit_result(api_name, policy, true, DialogErrorCode::None as u32, Some(&path));
    1
}
