//! Common CRT support functions and data used by both 32-bit and 64-bit msvcrt implementations.
//! This includes functions that are expected by the CRT but not provided by the Windows API,
//! as well as shared data exports like `_commode` and `_fmode`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

static FAKE_IOB_32: LazyLock<Box<[u8; 96]>> = LazyLock::new(build_fake_iob::<96, 32>);
static FAKE_IOB_64: LazyLock<Box<[u8; 144]>> = LazyLock::new(build_fake_iob::<144, 48>);
static CRT_LOCKS: LazyLock<Mutex<HashMap<i32, Arc<RecursiveMutex>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static ONEXIT_HANDLERS: LazyLock<Mutex<Vec<usize>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static ONEXIT_DRAINED: AtomicBool = AtomicBool::new(false);
static USER_MATH_ERR_HANDLER: AtomicUsize = AtomicUsize::new(0);
static mut APP_TYPE: LazyLock<AppType> = LazyLock::new(|| AppType::ConsoleApp);
const AMSG_EXIT_PREFIX: &[u8] = b"\nruntime error R6";

pub const CRT_DOMAIN_ERROR: i32 = 1;

#[repr(C)]
pub struct CrtMathException {
    pub type_: i32,
    pub name: *const i8,
    pub arg1: f64,
    pub arg2: f64,
    pub retval: f64,
}

#[repr(i32)]
pub enum AppType {
    /// CRT equivalent of "crt_unknown_app".
    /// The CRT doesn't know what type of application this is, so it should use default behavior.
    UnknownApp = 0,
    /// CRT equivalent of "crt_console_app".
    /// The CRT treats this as a console application.
    ConsoleApp = 1,
    /// CRT equivalent of "crt_gui_app".
    /// The CRT treats this as a GUI application.
    GuiApp = 2,
    /// CRT equivalent of "crt_cui_app".
    /// The CRT treats this as a character-based user interface application.
    CuiApp = 3,
    /// CRT equivalent of "crt_app_type_max".
    /// The CRT treats this as the maximum application type value.
    Max = 4,
}

impl From<i32> for AppType {
    fn from(value: i32) -> Self {
        match value {
            0 => AppType::UnknownApp,
            1 => AppType::ConsoleApp,
            2 => AppType::GuiApp,
            3 => AppType::CuiApp,
            4 => AppType::Max,
            _ => AppType::UnknownApp,
        }
    }
}

pub struct RecursiveMutex {
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
///
/// # Note
/// This is called by the CRT initialization code before `main()` runs. We currently ignore the app type since
/// we always run as a console application. This now at least stores the app type in a variable, but we don't
/// actually use it for anything yet.
pub fn set_app_type(app_type: AppType) {
    unsafe {
        *APP_TYPE = app_type;
    }
}

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
/// This stores the handler pointer so future floating-point error paths can call it.
pub fn set_user_math_err(handler: usize) {
    USER_MATH_ERR_HANDLER.store(handler, Ordering::Release);
}

/// Invoke the user-defined math error handler, if one is set.
///
/// # Arguments
/// * `exception`: A pointer to a `CrtMathException` structure containing details about the math error that occurred.
///
/// # Returns
/// `true` if a user math error handler was set and invoked, `false` if no handler was set.
/// The CRT will use the return value to determine whether the math error was handled by the user handler or if it
/// should perform default handling.
///
/// # Safety
/// This is unsafe because it calls a user-defined handler function that must follow the correct calling convention
/// and behavior expected by the CRT.
/// The `exception` pointer must point to a valid `CrtMathException` structure with the expected layout.
/// Incorrect handling could cause undefined behavior when math errors occur.
pub unsafe fn invoke_user_math_err(exception: *mut CrtMathException) -> bool {
    let handler = USER_MATH_ERR_HANDLER.load(Ordering::Acquire);
    if handler == 0 {
        return false;
    }

    let callback: unsafe extern "C" fn(*mut CrtMathException) -> i32 =
        unsafe { std::mem::transmute(handler) };
    unsafe { callback(exception) != 0 }
}

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
/// This follows the console-path CRT behavior: it writes the `R60xx` runtime
/// error code to stderr and terminates the process with exit code 255 without
/// running CRT exit handlers.
pub fn _amsg_exit(msg_num: i32) -> ! {
    let mut message = [0u8; 32];
    let len = encode_amsg_exit_message(msg_num, &mut message);
    write_stderr_best_effort(&message[..len]);

    unsafe { libc::_exit(255) }
}

fn encode_amsg_exit_message(msg_num: i32, out: &mut [u8; 32]) -> usize {
    out[..AMSG_EXIT_PREFIX.len()].copy_from_slice(AMSG_EXIT_PREFIX);
    let mut len = AMSG_EXIT_PREFIX.len();
    len += encode_runtime_error_digits(msg_num, &mut out[len..]);
    out[len] = b'\n';
    len + 1
}

fn encode_runtime_error_digits(msg_num: i32, out: &mut [u8]) -> usize {
    if (0..1000).contains(&msg_num) {
        let value = msg_num as u32;
        out[0] = b'0' + ((value / 100) % 10) as u8;
        out[1] = b'0' + ((value / 10) % 10) as u8;
        out[2] = b'0' + (value % 10) as u8;
        return 3;
    }

    let value = msg_num as i64;
    if value < 0 {
        out[0] = b'-';
        return 1 + encode_positive_decimal((-value) as u64, &mut out[1..]);
    }

    encode_positive_decimal(value as u64, out)
}

fn encode_positive_decimal(mut value: u64, out: &mut [u8]) -> usize {
    if value == 0 {
        out[0] = b'0';
        return 1;
    }

    let mut reversed = [0u8; 20];
    let mut reversed_len = 0;
    while value != 0 {
        reversed[reversed_len] = b'0' + (value % 10) as u8;
        reversed_len += 1;
        value /= 10;
    }

    for idx in 0..reversed_len {
        out[idx] = reversed[reversed_len - 1 - idx];
    }

    reversed_len
}

fn write_stderr_best_effort(bytes: &[u8]) {
    let mut written = 0;
    while written < bytes.len() {
        let remaining = &bytes[written..];
        let result = unsafe {
            libc::write(
                libc::STDERR_FILENO,
                remaining.as_ptr().cast::<libc::c_void>(),
                remaining.len(),
            )
        };

        if result <= 0 {
            break;
        }

        written += result as usize;
    }
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
/// * `sig`: The signal number to set the handler for.
/// * `handler`: A pointer to the signal handler function to be called when the signal is raised.
///
/// # Safety
/// This is unsafe because the CRT expects the handler pointer to be valid and follow the correct calling convention.
/// Registering an invalid handler could cause undefined behavior when the signal is raised.
///
/// # Notes
/// For now we forward directly to libc and return the previous handler as an address.
pub fn signal(sig: i32, handler: usize) -> usize {
    unsafe { libc::signal(sig, handler as libc::sighandler_t) as usize }
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
    use std::env;
    use std::process::Command;
    use std::sync::Mutex;
    use std::sync::mpsc;
    use std::time::Duration;

    use super::{
        encode_amsg_exit_message, errno_location, fake_iob_32_ptr, fake_iob_64_ptr, lock, signal,
        unlock,
    };

    static SIGNAL_TEST_LOCK: Mutex<()> = Mutex::new(());

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

    #[test]
    fn signal_returns_previous_handler() {
        let _guard = SIGNAL_TEST_LOCK.lock().unwrap();
        let sig = libc::SIGUSR1;

        let previous = signal(sig, libc::SIG_IGN as usize);
        let returned_previous = signal(sig, libc::SIG_DFL as usize);

        assert_eq!(returned_previous, libc::SIG_IGN as usize);

        signal(sig, previous);
    }

    #[test]
    fn signal_invalid_signal_sets_errno_and_returns_error_handler() {
        let _guard = SIGNAL_TEST_LOCK.lock().unwrap();

        unsafe {
            *errno_location() = 0;
        }

        let result = signal(-1, libc::SIG_DFL as usize);

        assert_eq!(result, libc::SIG_ERR as usize);
        assert_eq!(unsafe { *errno_location() }, libc::EINVAL);
    }

    #[test]
    fn amsg_exit_formats_msvcrt_runtime_error_code() {
        let mut message = [0u8; 32];

        let len = encode_amsg_exit_message(31, &mut message);

        assert_eq!(&message[..len], b"\nruntime error R6031\n");
    }

    #[test]
    fn amsg_exit_reports_runtime_error_and_exits_255() {
        const CHILD_ENV: &str = "RINE_COMMON_MSVCRT_AMSG_EXIT_CHILD";

        if let Ok(msg_num) = env::var(CHILD_ENV) {
            super::_amsg_exit(msg_num.parse().unwrap());
        }

        let output = Command::new(env::current_exe().unwrap())
            .arg("--exact")
            .arg("crt_support::tests::amsg_exit_reports_runtime_error_and_exits_255")
            .arg("--nocapture")
            .env(CHILD_ENV, "31")
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(255));

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("runtime error R6031"),
            "stderr was: {stderr}"
        );
    }
}
