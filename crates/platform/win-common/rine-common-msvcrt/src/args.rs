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
