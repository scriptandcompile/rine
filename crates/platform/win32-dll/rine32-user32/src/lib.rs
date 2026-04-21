#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-user32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

use rine_dlls::{DllPlugin, Export, PartialExport, StubExport, as_win_api};

mod class_registration;
mod message_queue;
mod window_lifecycle;
mod window_text;

pub struct User32Plugin32;

impl DllPlugin for User32Plugin32 {
    fn dll_names(&self) -> &[&str] {
        &["user32.dll"]
    }

    fn stubs(&self) -> Vec<StubExport> {
        vec![
            StubExport {
                name: "TranslateMessage",
                func: as_win_api!(message_queue::TranslateMessage),
            },
            StubExport {
                name: "DefWindowProcA",
                func: as_win_api!(message_queue::DefWindowProcA),
            },
            StubExport {
                name: "DefWindowProcW",
                func: as_win_api!(message_queue::DefWindowProcW),
            },
        ]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            // window_lifecycle.rs
            Export::Func("UpdateWindow", as_win_api!(window_lifecycle::UpdateWindow)),
            // message_queue.rs
            Export::Func(
                "PostQuitMessage",
                as_win_api!(message_queue::PostQuitMessage),
            ),
            // window_text.rs
            Export::Func(
                "GetWindowTextLengthA",
                as_win_api!(window_text::get_window_text_length_a),
            ),
            Export::Func(
                "GetWindowTextLengthW",
                as_win_api!(window_text::get_window_text_length_w),
            ),
        ]
    }

    fn partials(&self) -> Vec<PartialExport> {
        vec![
            // class_registeration.rs
            PartialExport {
                name: "RegisterClassA",
                func: as_win_api!(class_registration::RegisterClassA),
            },
            PartialExport {
                name: "RegisterClassW",
                func: as_win_api!(class_registration::RegisterClassW),
            },
            PartialExport {
                name: "RegisterClassExA",
                func: as_win_api!(class_registration::RegisterClassExA),
            },
            PartialExport {
                name: "RegisterClassExW",
                func: as_win_api!(class_registration::RegisterClassExW),
            },
            PartialExport {
                name: "UnregisterClassA",
                func: as_win_api!(class_registration::UnregisterClassA),
            },
            PartialExport {
                name: "UnregisterClassW",
                func: as_win_api!(class_registration::UnregisterClassW),
            },
            // message_queue.rs
            PartialExport {
                name: "GetMessageA",
                func: as_win_api!(message_queue::GetMessageA),
            },
            PartialExport {
                name: "GetMessageW",
                func: as_win_api!(message_queue::GetMessageW),
            },
            PartialExport {
                name: "PeekMessageA",
                func: as_win_api!(message_queue::PeekMessageA),
            },
            PartialExport {
                name: "PeekMessageW",
                func: as_win_api!(message_queue::PeekMessageW),
            },
            PartialExport {
                name: "DispatchMessageA",
                func: as_win_api!(message_queue::DispatchMessageA),
            },
            PartialExport {
                name: "DispatchMessageW",
                func: as_win_api!(message_queue::DispatchMessageW),
            },
            PartialExport {
                name: "PostMessageA",
                func: as_win_api!(message_queue::PostMessageA),
            },
            PartialExport {
                name: "PostMessageW",
                func: as_win_api!(message_queue::PostMessageW),
            },
            PartialExport {
                name: "SendMessageA",
                func: as_win_api!(message_queue::SendMessageA),
            },
            PartialExport {
                name: "SendMessageW",
                func: as_win_api!(message_queue::SendMessageW),
            },
            // window_lifecycle.rs
            PartialExport {
                name: "CreateWindowExA",
                func: as_win_api!(window_lifecycle::CreateWindowExA),
            },
            PartialExport {
                name: "CreateWindowExW",
                func: as_win_api!(window_lifecycle::CreateWindowExW),
            },
            PartialExport {
                name: "DestroyWindow",
                func: as_win_api!(window_lifecycle::DestroyWindow),
            },
            PartialExport {
                name: "ShowWindow",
                func: as_win_api!(window_lifecycle::ShowWindow),
            },
            // window_text.rs
            PartialExport {
                name: "SetWindowTextA",
                func: as_win_api!(window_text::SetWindowTextA),
            },
            PartialExport {
                name: "SetWindowTextW",
                func: as_win_api!(window_text::SetWindowTextW),
            },
            PartialExport {
                name: "GetWindowTextA",
                func: as_win_api!(window_text::GetWindowTextA),
            },
            PartialExport {
                name: "GetWindowTextW",
                func: as_win_api!(window_text::GetWindowTextW),
            },
        ]
    }
}
