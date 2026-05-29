//! kernel32 memory functions: Heap API and VirtualAlloc/VirtualFree/VirtualProtect/VirtualQuery.

use rine_common_kernel32 as common;
use rine_types::errors::BOOL;
use rine_types::handles::{HANDLE, HGLOBAL, HLOCAL};

pub mod global;
pub mod heap;
pub mod local;
pub mod virtual_memory;

pub use global::*;
pub use heap::*;
pub use local::*;
pub use virtual_memory::*;
