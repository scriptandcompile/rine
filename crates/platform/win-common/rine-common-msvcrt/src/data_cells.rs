use std::sync::LazyLock;

struct SyncPtr<T>(T);

unsafe impl<T> Sync for SyncPtr<T> {}
unsafe impl<T> Send for SyncPtr<T> {}

fn leaked_i32(initial: i32) -> *mut i32 {
    Box::into_raw(Box::new(initial))
}

fn leaked_usize(initial: usize) -> *mut usize {
    Box::into_raw(Box::new(initial))
}

static COMMODE_PTR: LazyLock<SyncPtr<*mut i32>> = LazyLock::new(|| SyncPtr(leaked_i32(0)));
static FMODE_PTR: LazyLock<SyncPtr<*mut i32>> = LazyLock::new(|| SyncPtr(leaked_i32(0)));
static INITENV_PTR: LazyLock<SyncPtr<*mut usize>> = LazyLock::new(|| SyncPtr(leaked_usize(0)));

pub fn commode_ptr() -> *mut i32 {
    COMMODE_PTR.0
}

pub fn fmode_ptr() -> *mut i32 {
    FMODE_PTR.0
}

pub fn initenv_ptr() -> *mut usize {
    INITENV_PTR.0
}
