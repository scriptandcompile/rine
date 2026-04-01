//! Heap management subsystem.
//!
//! The core heap API entry-points (`HeapCreate`, `HeapAlloc`, `HeapFree`,
//! `HeapReAlloc`, `GetProcessHeap`) and virtual-memory functions
//! (`VirtualAlloc`, `VirtualFree`) live in `rine64-kernel32::memory`.
//!
//! Heap state is tracked via [`rine_types::handles::HeapState`] entries in
//! the global handle table.  `VirtualAlloc` regions are tracked in a
//! module-level table inside the kernel32 memory module.
