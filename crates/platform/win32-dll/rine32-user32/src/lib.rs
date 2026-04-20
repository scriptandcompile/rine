#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "crate `rine32-user32` must be built for a 32-bit target (for example: --target i686-unknown-linux-gnu)"
);

use rine_dlls::{DllPlugin, Export, PartialExport, as_win_api};

mod class_registration;
mod message_queue;
mod window_lifecycle;
mod window_text;

pub struct User32Plugin32;

impl DllPlugin for User32Plugin32 {
    fn dll_names(&self) -> &[&str] {
        &["user32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![
            Export::Func(
                "RegisterClassW",
                as_win_api!(class_registration::register_class_w),
            ),
            Export::Func(
                "RegisterClassExA",
                as_win_api!(class_registration::register_class_ex_a),
            ),
            Export::Func(
                "RegisterClassExW",
                as_win_api!(class_registration::register_class_ex_w),
            ),
            Export::Func(
                "UnregisterClassA",
                as_win_api!(class_registration::unregister_class_a),
            ),
            Export::Func(
                "UnregisterClassW",
                as_win_api!(class_registration::unregister_class_w),
            ),
            Export::Func(
                "CreateWindowExA",
                as_win_api!(window_lifecycle::create_window_ex_a),
            ),
            Export::Func(
                "CreateWindowExW",
                as_win_api!(window_lifecycle::create_window_ex_w),
            ),
            Export::Func(
                "DestroyWindow",
                as_win_api!(window_lifecycle::destroy_window),
            ),
            Export::Func("ShowWindow", as_win_api!(window_lifecycle::show_window)),
            Export::Func("UpdateWindow", as_win_api!(window_lifecycle::update_window)),
            Export::Func("GetMessageA", as_win_api!(message_queue::get_message_a)),
            Export::Func("GetMessageW", as_win_api!(message_queue::get_message_w)),
            Export::Func("PeekMessageA", as_win_api!(message_queue::peek_message_a)),
            Export::Func("PeekMessageW", as_win_api!(message_queue::peek_message_w)),
            Export::Func(
                "TranslateMessage",
                as_win_api!(message_queue::translate_message),
            ),
            Export::Func(
                "DispatchMessageA",
                as_win_api!(message_queue::dispatch_message_a),
            ),
            Export::Func(
                "DispatchMessageW",
                as_win_api!(message_queue::dispatch_message_w),
            ),
            Export::Func(
                "PostQuitMessage",
                as_win_api!(message_queue::post_quit_message),
            ),
            Export::Func("PostMessageA", as_win_api!(message_queue::post_message_a)),
            Export::Func("PostMessageW", as_win_api!(message_queue::post_message_w)),
            Export::Func("SendMessageA", as_win_api!(message_queue::send_message_a)),
            Export::Func("SendMessageW", as_win_api!(message_queue::send_message_w)),
            Export::Func(
                "DefWindowProcA",
                as_win_api!(message_queue::def_window_proc_a),
            ),
            Export::Func(
                "DefWindowProcW",
                as_win_api!(message_queue::def_window_proc_w),
            ),
            Export::Func(
                "SetWindowTextA",
                as_win_api!(window_text::set_window_text_a),
            ),
            Export::Func(
                "SetWindowTextW",
                as_win_api!(window_text::set_window_text_w),
            ),
            Export::Func(
                "GetWindowTextA",
                as_win_api!(window_text::get_window_text_a),
            ),
            Export::Func(
                "GetWindowTextW",
                as_win_api!(window_text::get_window_text_w),
            ),
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
        vec![PartialExport {
            name: "RegisterClassA",
            func: as_win_api!(class_registration::RegisterClassA),
        }]
    }
}
