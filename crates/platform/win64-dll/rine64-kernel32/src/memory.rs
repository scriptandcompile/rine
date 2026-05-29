//! kernel32 memory functions: Heap API and VirtualAlloc/VirtualFree/VirtualProtect/VirtualQuery.

use rine_common_kernel32 as common;
use rine_types::errors::BOOL;
use rine_types::handles::{HANDLE, HGLOBAL, HLOCAL};

pub mod heap;
pub mod global;
pub mod local;
pub mod virtual_memory;

pub mod local_global {
	pub use super::global::*;
	pub use super::local::*;
}

pub use heap::*;
pub use global::*;
pub use local::*;
pub use virtual_memory::*;
