//! MSVCRT CRT support functions needed during CRT startup.
//!
//! These are called by the MinGW CRT startup code before `main()` runs.
//! Most are no-ops or minimal stubs for Phase 1.

/// Wrapper to make a raw pointer Sync for use in statics.
struct SyncPtr<T>(T);
unsafe impl<T> Sync for SyncPtr<T> {}
unsafe impl<T> Send for SyncPtr<T> {}

// PE CRT startup writes to these with plain `mov` instructions, so they MUST
// live in heap-allocated (writable) memory — `static AtomicI32` gets placed
// in a read-only section and faults.
static COMMODE_PTR: std::sync::LazyLock<SyncPtr<*mut i32>> =
    std::sync::LazyLock::new(|| SyncPtr(Box::into_raw(Box::new(0i32))));
static FMODE_PTR: std::sync::LazyLock<SyncPtr<*mut i32>> =
    std::sync::LazyLock::new(|| SyncPtr(Box::into_raw(Box::new(0i32))));

static INITENV_PTR: std::sync::LazyLock<SyncPtr<*mut usize>> =
    std::sync::LazyLock::new(|| SyncPtr(Box::into_raw(Box::new(0usize))));

/// __set_app_type — set the application type (console/GUI).
///
/// No-op: rine always runs as a console application.
pub unsafe extern "win64" fn __set_app_type(_app_type: i32) {
    tracing::trace!("msvcrt::__set_app_type({_app_type})");
}

/// __setusermatherr — register a custom math error handler.
///
/// No-op: we don't support custom math error handlers.
pub unsafe extern "win64" fn __setusermatherr(_handler: usize) {
    tracing::trace!("msvcrt::__setusermatherr");
}

/// __C_specific_handler — SEH personality function for x64 Windows.
///
/// Stub: returns ExceptionContinueSearch (1). Called only if an exception is
/// thrown, which shouldn't happen in a simple hello world.
pub unsafe extern "win64" fn __C_specific_handler(
    _exception_record: usize,
    _establisher_frame: usize,
    _context_record: usize,
    _dispatcher_context: usize,
) -> i32 {
    tracing::warn!("msvcrt::__C_specific_handler called — exceptions not supported");
    1 // ExceptionContinueSearch
}

/// _commode — return a pointer to the commit mode variable.
pub unsafe extern "win64" fn _commode() -> *mut i32 {
    COMMODE_PTR.0
}

/// Return the raw pointer to the _commode variable for data-export registration.
pub fn commode_data_ptr() -> *mut i32 {
    COMMODE_PTR.0
}

/// _fmode — return a pointer to the default file translation mode.
pub unsafe extern "win64" fn _fmode() -> *mut i32 {
    FMODE_PTR.0
}

/// Return the raw pointer to the _fmode variable for data-export registration.
pub fn fmode_data_ptr() -> *mut i32 {
    FMODE_PTR.0
}

/// __initenv — return a pointer to the initial environment pointer.
///
/// Returns a pointer to a NULL pointer (empty environment at CRT level;
/// the real environment is provided via `__getmainargs`).
pub unsafe extern "win64" fn __initenv() -> *const *const i8 {
    INITENV_PTR.0 as *const *const i8
}

/// Return the raw pointer to the __initenv variable for data-export registration.
pub fn initenv_data_ptr() -> *mut usize {
    INITENV_PTR.0
}

// Fake FILE table for __iob_func. Windows CRT __iob_func returns a pointer
// to an array of three FILE structs (stdin, stdout, stderr). The MinGW CRT
// uses these for stdio operations. We provide a minimal fake that stores
// just enough to identify each stream.
//
// Windows FILE struct is 48 bytes; we allocate enough space for 3 entries.
// Pre-initialized with fd markers: stdin=0, stdout=1, stderr=2 in the first
// 4 bytes of each 48-byte entry.
static FAKE_IOB: std::sync::LazyLock<Box<[u8; 144]>> = std::sync::LazyLock::new(|| {
    let mut buf = Box::new([0u8; 144]);
    // Write fd markers into the first 4 bytes of each FILE entry.
    buf[0..4].copy_from_slice(&0i32.to_ne_bytes()); // stdin fd=0
    buf[48..52].copy_from_slice(&1i32.to_ne_bytes()); // stdout fd=1
    buf[96..100].copy_from_slice(&2i32.to_ne_bytes()); // stderr fd=2
    buf
});

/// __iob_func — return pointer to the stdio FILE table.
///
/// Returns a fake FILE table. The first 3 entries represent stdin (0),
/// stdout (1), stderr (2). We store a marker fd in the first field of
/// each entry so fwrite/fprintf can identify the stream.
pub unsafe extern "win64" fn __iob_func() -> *mut u8 {
    FAKE_IOB.as_ptr() as *mut u8
}

/// _onexit — register a function to be called at exit.
///
/// Stub: returns the function pointer (success) but does not actually
/// register it for later calling. Full atexit support in a later phase.
pub unsafe extern "win64" fn _onexit(func: usize) -> usize {
    tracing::trace!("msvcrt::_onexit");
    func // return non-NULL to indicate success
}

/// _amsg_exit — display a runtime error message and abort.
pub unsafe extern "win64" fn _amsg_exit(msg_num: i32) {
    eprintln!("rine: msvcrt runtime error (msg_num={msg_num})");
    std::process::abort();
}

/// abort — abnormally terminate the process.
pub unsafe extern "win64" fn abort() {
    tracing::debug!("msvcrt::abort");
    std::process::abort();
}

/// signal — install a signal handler.
///
/// Stub: returns SIG_DFL (0). Minimal implementation since Windows signals
/// are rarely used in practice.
pub unsafe extern "win64" fn signal(
    _sig: i32,
    _handler: usize, // void (*)(int)
) -> usize {
    0 // SIG_DFL
}

/// _lock — acquire an internal CRT lock.
///
/// No-op for single-threaded Phase 1.
pub unsafe extern "win64" fn _lock(_locknum: i32) {}

/// _unlock — release an internal CRT lock.
///
/// No-op for single-threaded Phase 1.
pub unsafe extern "win64" fn _unlock(_locknum: i32) {}

/// _errno — return a pointer to the per-thread errno value.
///
/// Returns a pointer to libc's errno, which is thread-local.
pub unsafe extern "win64" fn _errno() -> *mut i32 {
    unsafe { libc::__errno_location() }
}

/// __p__environ — return a pointer to the environment variable array.
///
/// Returns a pointer to a NULL pointer (minimal stub).
pub unsafe extern "win64" fn __p__environ() -> *const *const *const i8 {
    INITENV_PTR.0 as *const *const *const i8
}

/// __p__fmode — return a pointer to the global file mode variable.
///
/// Returns the same pointer as `_fmode()`.
pub unsafe extern "win64" fn __p__fmode() -> *mut i32 {
    FMODE_PTR.0
}

/// __p__commode — return a pointer to the global commit mode variable.
///
/// Returns the same pointer as `_commode()`.
pub unsafe extern "win64" fn __p__commode() -> *mut i32 {
    COMMODE_PTR.0
}
