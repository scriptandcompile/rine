mod allocation;
mod args;
mod crt_support;
mod data_cells;
mod init;

pub use allocation::AllocationTracker;
pub use args::{MainArgs, cached_main_args};
pub use crt_support::{
    abort_process, amsg_exit, c_specific_handler_result, errno_location, fake_iob_32_ptr,
    fake_iob_64_ptr, lock, onexit, set_app_type, set_usermatherr, signal_default, unlock,
};
pub use data_cells::{commode_ptr, fmode_ptr, initenv_ptr};
pub use init::{run_initterm, run_initterm_e};
