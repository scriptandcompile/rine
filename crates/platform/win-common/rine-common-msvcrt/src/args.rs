use std::ffi::CString;
use std::sync::OnceLock;

pub struct MainArgs {
    argc: i32,
    argv_ptrs: Vec<*mut i8>,
    envp_ptrs: Vec<*mut i8>,
    _argv_strings: Vec<CString>,
    _envp_strings: Vec<CString>,
}

impl MainArgs {
    pub fn argc(&self) -> i32 {
        self.argc
    }

    pub fn argv_ptr(&self) -> *mut *mut i8 {
        self.argv_ptrs.as_ptr() as *mut *mut i8
    }

    pub fn envp_ptr(&self) -> *mut *mut i8 {
        self.envp_ptrs.as_ptr() as *mut *mut i8
    }
}

unsafe impl Send for MainArgs {}
unsafe impl Sync for MainArgs {}

static MAIN_ARGS: OnceLock<MainArgs> = OnceLock::new();

pub fn cached_main_args() -> &'static MainArgs {
    MAIN_ARGS.get_or_init(|| {
        let args: Vec<String> = std::env::args().collect();
        let argv_strings: Vec<CString> = args
            .iter()
            .map(|arg| CString::new(arg.as_str()).unwrap_or_default())
            .collect();
        let mut argv_ptrs: Vec<*mut i8> = argv_strings
            .iter()
            .map(|cstring| cstring.as_ptr() as *mut i8)
            .collect();
        argv_ptrs.push(std::ptr::null_mut());

        let envp_strings: Vec<CString> = std::env::vars()
            .map(|(key, value)| CString::new(format!("{key}={value}")).unwrap_or_default())
            .collect();
        let mut envp_ptrs: Vec<*mut i8> = envp_strings
            .iter()
            .map(|cstring| cstring.as_ptr() as *mut i8)
            .collect();
        envp_ptrs.push(std::ptr::null_mut());

        MainArgs {
            argc: args.len() as i32,
            argv_ptrs,
            envp_ptrs,
            _argv_strings: argv_strings,
            _envp_strings: envp_strings,
        }
    })
}

/// MainArgsW — wide-character (UTF-16) variant of command-line argument
/// and environment storage, intended for use with `__wgetmainargs`.
///
/// Stores program arguments and environment variables as null-terminated
/// wide character strings (`wchar_t*` / `u16`).
///
/// # Notes
/// This is currently a partial implementation. The `_do_wildcard` and
/// `_start_info` parameters from `__wgetmainargs` are not handled.
/// Wildcard expansion is not implemented.
pub struct MainArgsW {
    argc: i32,
    argv_ptrs: Vec<*mut u16>,
    envp_ptrs: Vec<*mut u16>,
    _argv_strings: Vec<Vec<u16>>,
    _envp_strings: Vec<Vec<u16>>,
}

impl MainArgsW {
    /// Returns the argument count (`argc`).
    pub fn argc(&self) -> i32 {
        self.argc
    }

    /// Returns a pointer to the wide argument vector (`wchar_t*** argv`).
    pub fn argv_ptr(&self) -> *mut *mut u16 {
        self.argv_ptrs.as_ptr() as *mut *mut u16
    }

    /// Returns a pointer to the wide environment vector (`wchar_t*** envp`).
    pub fn envp_ptr(&self) -> *mut *mut u16 {
        self.envp_ptrs.as_ptr() as *mut *mut u16
    }
}

unsafe impl Send for MainArgsW {}
unsafe impl Sync for MainArgsW {}

static WIDE_MAIN_ARGS: OnceLock<MainArgsW> = OnceLock::new();

/// Convert a Rust `&str` into a null-terminated UTF-16 `Vec<u16>`.
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Returns the cached wide-character main arguments and environment,
/// initializing them from [`std::env::args`] and [`std::env::vars`] on
/// first access.
///
/// # Notes
/// - The returned pointers remain valid for the lifetime of the process.
/// - Wildcard expansion is not performed (same as the narrow variant).
/// - Environment variables are formatted as `"KEY=VALUE"` wide strings.
pub fn cached_wide_main_args() -> &'static MainArgsW {
    WIDE_MAIN_ARGS.get_or_init(|| {
        let args: Vec<String> = std::env::args().collect();

        let argv_strings: Vec<Vec<u16>> = args.iter().map(|arg| to_wide(arg.as_str())).collect();
        let mut argv_ptrs: Vec<*mut u16> = argv_strings
            .iter()
            .map(|ws| ws.as_ptr() as *mut u16)
            .collect();
        argv_ptrs.push(std::ptr::null_mut());

        let envp_strings: Vec<Vec<u16>> = std::env::vars()
            .map(|(key, value)| to_wide(format!("{key}={value}").as_str()))
            .collect();
        let mut envp_ptrs: Vec<*mut u16> = envp_strings
            .iter()
            .map(|ws| ws.as_ptr() as *mut u16)
            .collect();
        envp_ptrs.push(std::ptr::null_mut());

        MainArgsW {
            argc: args.len() as i32,
            argv_ptrs,
            envp_ptrs,
            _argv_strings: argv_strings,
            _envp_strings: envp_strings,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_wide_produces_null_terminated_utf16() {
        let wide = to_wide("hello");
        let expected: Vec<u16> = "hello".encode_utf16().chain(std::iter::once(0)).collect();
        assert_eq!(wide, expected);
    }

    #[test]
    fn to_wide_empty_string_produces_just_null() {
        let wide = to_wide("");
        assert_eq!(wide, vec![0u16]);
    }

    #[test]
    fn main_argsw_has_at_least_one_argument() {
        let args = cached_wide_main_args();
        assert!(args.argc() >= 1);
    }

    #[test]
    fn main_argsw_argv_is_null_terminated() {
        let args = cached_wide_main_args();
        let ptrs = args.argv_ptr();
        assert!(!ptrs.is_null());
        unsafe {
            let mut i = 0;
            while !(*ptrs.add(i)).is_null() {
                i += 1;
            }
            assert_eq!(i, args.argc() as usize);
        }
    }

    #[test]
    fn main_argsw_envp_is_not_empty() {
        let args = cached_wide_main_args();
        let ptrs = args.envp_ptr();
        assert!(!ptrs.is_null());
        unsafe {
            assert!(!(*ptrs).is_null(), "envp[0] should be non-null");
        }
    }
}
