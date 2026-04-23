//! Common CRT support functions and data used by both 32-bit and 64-bit msvcrt implementations.
//! This includes functions that are expected by the CRT but not provided by the Windows API,
//! as well as shared data exports like `_commode` and `_fmode`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

static FAKE_IOB_32: LazyLock<Box<[u8; 96]>> = LazyLock::new(build_fake_iob::<96, 32>);
static FAKE_IOB_64: LazyLock<Box<[u8; 144]>> = LazyLock::new(build_fake_iob::<144, 48>);
static CRT_LOCKS: LazyLock<Mutex<HashMap<i32, Arc<RecursiveMutex>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static ONEXIT_HANDLERS: LazyLock<Mutex<Vec<usize>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static ONEXIT_DRAINED: AtomicBool = AtomicBool::new(false);

struct RecursiveMutex {
    raw: *mut libc::pthread_mutex_t,
}

unsafe impl Send for RecursiveMutex {}
unsafe impl Sync for RecursiveMutex {}

impl RecursiveMutex {
    fn new() -> Self {
        unsafe {
            let raw = Box::into_raw(Box::new(std::mem::zeroed::<libc::pthread_mutex_t>()));
            let mut attr = std::mem::zeroed::<libc::pthread_mutexattr_t>();

            libc::pthread_mutexattr_init(&mut attr);
            libc::pthread_mutexattr_settype(&mut attr, libc::PTHREAD_MUTEX_RECURSIVE);
            libc::pthread_mutex_init(raw, &attr);
            libc::pthread_mutexattr_destroy(&mut attr);

            Self { raw }
        }
    }

    fn lock(&self) {
        unsafe {
            libc::pthread_mutex_lock(self.raw);
        }
    }

    fn unlock(&self) {
        unsafe {
            libc::pthread_mutex_unlock(self.raw);
        }
    }
}

impl Drop for RecursiveMutex {
    fn drop(&mut self) {
        unsafe {
            libc::pthread_mutex_destroy(self.raw);
            drop(Box::from_raw(self.raw));
        }
    }
}

/// An internal function used at startup to tell the CRT what type of application we're running (console, GUI, etc).
///
/// # Arguments
/// * `app_type`: An integer representing the application type. The CRT uses this to configure its behavior accordingly.
///   The specific values and their meanings are defined by the CRT, but common values include:
///   0 = _crt_unknown_app
///   1 = _crt_console_app
///   2 = _crt_gui_app
///   3 = _crt_cui_app
///   4 = _crt_app_type_max
///
/// # Note
/// This is called by the CRT initialization code before `main()` runs. We currently ignore the app type since
/// we always run as a console application, but a production implementation would use this to configure CRT behavior accordingly.
/// Currently, this is just a no-op.
pub fn set_app_type(_app_type: i32) {}

/// Set a custom math error handler.
///
/// # Arguments
/// * `handler`: A pointer to a user-defined math error handler function.
///   The CRT will call this function when a math error occurs (like divide-by-zero or overflow).
///
/// # Safety
/// This is unsafe because the handler must follow the correct calling convention and behavior expected by the CRT.
/// Installing an invalid handler could cause undefined behavior when math errors occur.
///
/// # Notes
/// This is a no-op currently; a production implementation would let the user install a handler for floating-point errors.
pub fn set_user_math_err(_handler: usize) {}

/// Called by the CRT when a SEH exception is thrown.
///
/// # Arguments
/// * `_exception_record`: A pointer to an EXCEPTION_RECORD structure containing information about the exception.
/// * `_establisher_frame`: A pointer to the frame of the function where the exception occurred.
/// * `_context_record`: A pointer to a CONTEXT structure containing the CPU context at the time of the exception.
///
/// # Safety
/// This is called by the CRT when a SEH exception is thrown.
/// The arguments are pointers to CRT-defined structures with specific layouts,
/// and the function must return a valid handler code expected by the CRT.
/// Incorrect handling could lead to undefined behavior when exceptions occur.
///
/// # Returns
/// This is a stub currently that just returns "continue search" (1).
///
/// # Notes
/// This is called by the CRT when a SEH exception is thrown.
/// We don't support SEH exceptions currently, so this is just a stub that returns "continue search" (1) to
/// indicate that the CRT should call the next handler.
/// In a production implementation, this would analyze the exception record and return the appropriate handler code
/// (1 = continue execution, 0 = call next handler).
pub fn c_specific_handler_result(
    _exception_record: usize,
    _establisher_frame: usize,
    _context_record: usize,
    _dispatcher_context: usize,
) -> i32 {
    1
}

/// Get a pointer to the CRT's internal array of three FILE structures for stdin, stdout, and stderr.
///
/// # Returns
/// A pointer to an array of three FILE structures expected by the CRT for standard I/O operations.
/// The CRT expects this to be exported as `_iob` and used by functions like `printf` and `fprintf`.
pub fn fake_iob_32_ptr() -> *mut u8 {
    FAKE_IOB_32.as_ptr() as *mut u8
}

/// Get a pointer to the CRT's internal array of three FILE structures for stdin, stdout, and stderr.
///
/// # Returns
/// A pointer to an array of three FILE structures expected by the CRT for standard I/O operations.
/// The CRT expects this to be exported as `_iob` and used by functions like `printf` and `fprintf`.
pub fn fake_iob_64_ptr() -> *mut u8 {
    FAKE_IOB_64.as_ptr() as *mut u8
}

/// Register a function to be called at process exit.
///
/// # Arguments
/// * `func`: A pointer to a function that takes no arguments and returns void.
///   This function will be called when the process exits, either normally or via `exit()`.
///
/// # Safety
/// This is unsafe because the CRT expects the function pointer to be valid and follow the correct calling convention.
/// Registering an invalid function could cause undefined behavior when the process exits.
///
/// # Notes
/// Handlers are executed in reverse registration order (LIFO) when `_cexit`/`exit`
/// performs CRT teardown.
pub fn onexit(func: usize) -> usize {
    if func == 0 {
        return 0;
    }

    let mut handlers = ONEXIT_HANDLERS.lock().unwrap();
    handlers.push(func);
    func
}

/// Run registered `_onexit` handlers once in LIFO order.
///
/// # Safety
/// Each stored address must point to a valid C ABI function with signature `fn() -> i32`.
pub unsafe fn run_onexit_handlers() {
    if ONEXIT_DRAINED.swap(true, Ordering::AcqRel) {
        return;
    }

    let handlers = {
        let mut guard = ONEXIT_HANDLERS.lock().unwrap();
        std::mem::take(&mut *guard)
    };

    for func in handlers.into_iter().rev() {
        if func == 0 {
            continue;
        }

        let callback: unsafe extern "C" fn() -> i32 = unsafe { std::mem::transmute(func) };
        let _ = unsafe { callback() };
    }
}

/// Display a CRT error message and abort the process.
///
/// # Arguments
/// * `msg_num`: An integer representing the error message number. The CRT uses this to determine which error message to display.
///
/// # Notes
/// This is a stub implementation that just prints the message number and aborts the process.
pub fn amsg_exit(msg_num: i32) -> ! {
    eprintln!("rine: msvcrt runtime error (msg_num={msg_num})");
    std::process::abort();
}

/// Abort the process immediately without unwinding or running exit handlers.
///
/// # Safety
/// This is unsafe because it will terminate the process immediately without running any cleanup code or exit handlers.
/// It should only be called in situations where the process is in an unrecoverable state and cannot continue safely.
///
/// # Notes
/// This is a stub implementation that just calls `std::process::abort()`.
pub fn abort_process() -> ! {
    std::process::abort();
}

/// Set a signal handler for the specified signal.
///
/// # Arguments
/// * `_sig`: The signal number to set the handler for.
/// * `_handler`: A pointer to the signal handler function to be called when the signal is raised.
///
/// # Safety
/// This is unsafe because the CRT expects the handler pointer to be valid and follow the correct calling convention.
/// Registering an invalid handler could cause undefined behavior when the signal is raised.
///
/// # Notes
/// This is a stub implementation that does nothing and returns 0.
pub fn signal_default(_sig: i32, _handler: usize) -> usize {
    0
}

/// Acquire a CRT lock for the specified lock number.
///
/// # Arguments
/// * `locknum`: The lock number to acquire. The CRT uses this to synchronize access to internal resources.
///
/// # Safety
/// This is unsafe because the CRT expects locks to be properly acquired and released to avoid deadlocks and ensure thread safety.
/// Incorrect usage could lead to undefined behavior when multiple threads access CRT resources.
pub fn lock(locknum: i32) {
    let mutex = {
        let mut locks = CRT_LOCKS.lock().unwrap();
        locks
            .entry(locknum)
            .or_insert_with(|| Arc::new(RecursiveMutex::new()))
            .clone()
    };

    mutex.lock();
}

/// Release a CRT lock for the specified lock number.
///
/// # Arguments
/// * `locknum`: The lock number to release. This should match a previously acquired lock number.
///
/// # Safety
/// This is unsafe because the CRT expects locks to be properly acquired and released to avoid deadlocks and ensure thread safety.
/// Incorrect usage (like unlocking a lock that wasn't acquired) could lead to undefined behavior when multiple
/// threads access CRT resources.
pub fn unlock(locknum: i32) {
    let mutex = {
        let locks = CRT_LOCKS.lock().unwrap();
        locks.get(&locknum).cloned()
    };

    if let Some(mutex) = mutex {
        mutex.unlock();
    }
}

/// Get a pointer to the thread-local `errno` value.
///
/// # Safety
/// This is unsafe because the CRT expects this to return a valid pointer to a thread-local variable that holds
/// the error code for the last failed system call.
/// The CRT and C code will read and write to this variable to get and set the error code for the last failed system call.
/// Incorrect handling could lead to undefined behavior when CRT functions access this variable.
///
/// # Returns
/// A pointer to the thread-local `errno` variable.
/// The CRT and C code will read and write to this variable to get and set the error code for the last failed system call.
pub fn errno_location() -> *mut i32 {
    unsafe { libc::__errno_location() }
}

/// Creates fake stdio control structures expected by the CRT,
///
/// # Safety
/// This is unsafe because the CRT expects the returned pointer to point to a valid array of
/// three FILE structures with specific layout and contents.
/// Incorrect handling could lead to undefined behavior when CRT functions access this array.
/// The returned pointer should be exported as `_iob` and used by CRT functions that perform standard I/O operations.
///
/// # Returns
/// A pointer to an array of three FILE structures expected by the CRT for standard I/O operations.
/// The CRT expects this to be exported as `_iob` and used by functions like `printf` and `fprintf`.
///
/// # Notes
/// This function creates a fake `_iob` array with the expected layout and some dummy values for the
/// file descriptors (0 for stdin, 1 for stdout, 2 for stderr).
fn build_fake_iob<const SIZE: usize, const ENTRY_SIZE: usize>() -> Box<[u8; SIZE]> {
    let mut buf = Box::new([0u8; SIZE]);
    buf[0..4].copy_from_slice(&0i32.to_ne_bytes());
    buf[ENTRY_SIZE..ENTRY_SIZE + 4].copy_from_slice(&1i32.to_ne_bytes());
    buf[ENTRY_SIZE * 2..ENTRY_SIZE * 2 + 4].copy_from_slice(&2i32.to_ne_bytes());
    buf
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use super::{fake_iob_32_ptr, fake_iob_64_ptr, lock, unlock};

    #[test]
    fn fake_iob_32_has_expected_markers() {
        let ptr = fake_iob_32_ptr() as *const u8;
        let bytes = unsafe { std::slice::from_raw_parts(ptr, 96) };
        assert_eq!(&bytes[0..4], &0i32.to_ne_bytes());
        assert_eq!(&bytes[32..36], &1i32.to_ne_bytes());
        assert_eq!(&bytes[64..68], &2i32.to_ne_bytes());
    }

    #[test]
    fn fake_iob_64_has_expected_markers() {
        let ptr = fake_iob_64_ptr() as *const u8;
        let bytes = unsafe { std::slice::from_raw_parts(ptr, 144) };
        assert_eq!(&bytes[0..4], &0i32.to_ne_bytes());
        assert_eq!(&bytes[48..52], &1i32.to_ne_bytes());
        assert_eq!(&bytes[96..100], &2i32.to_ne_bytes());
    }

    #[test]
    fn crt_locks_are_recursive_per_locknum() {
        lock(11);
        lock(11);
        unlock(11);
        unlock(11);
    }

    #[test]
    fn crt_locks_block_other_threads_until_unlocked() {
        let locknum = 23;
        let (ready_tx, ready_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();

        lock(locknum);

        let worker = std::thread::spawn(move || {
            ready_tx.send(()).unwrap();
            lock(locknum);
            unlock(locknum);
            done_tx.send(()).unwrap();
        });

        ready_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(done_rx.recv_timeout(Duration::from_millis(50)).is_err());

        unlock(locknum);

        done_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        worker.join().unwrap();
    }
}
