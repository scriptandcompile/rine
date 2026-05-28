mod allocation;
mod args;
mod crt_support;
mod data_cells;
mod init;
mod locale;
mod math;
mod memory;
mod stdio;
mod stdlib;
mod string;

pub use allocation::AllocationTracker;
pub use args::{MainArgs, MainArgsW, cached_main_args, cached_wide_main_args};
pub use crt_support::{
    _amsg_exit, abort_process, c_specific_handler_result, errno_location, fake_iob_32_ptr,
    fake_iob_64_ptr, lock, onexit, set_app_type, set_user_math_err, signal, unlock,
};
pub use data_cells::{commode_ptr, fmode_ptr, initenv_ptr, wcmdln_ptr, wcmdln_str, winitenv_ptr};
pub use init::{run_initterm, run_initterm_e};
pub use locale::{___lc_codepage_func, ___mb_cur_max_func, localeconv};
pub use math::sqrt;
pub use memory::{CRT_ALLOCATIONS, calloc, free, malloc, memcpy, memset, realloc};
pub use stdio::{
    fputwc_to_stream, printf_win64_thunk, printf_x86_thunk, puts_to_stdout, write_buffer_to_stream,
    write_format_to_stream, write_wide_format_to_stream,
};
pub use stdlib::{_cexit, exit};
pub use string::{strcmp, strerror, strlen, strncmp, wcslen};
