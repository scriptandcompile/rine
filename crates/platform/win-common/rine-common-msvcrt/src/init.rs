#[allow(clippy::missing_safety_doc)]
pub unsafe fn run_initterm<F, Invoke>(
    start: *const Option<F>,
    end: *const Option<F>,
    mut invoke: Invoke,
) where
    F: Copy,
    Invoke: FnMut(F),
{
    if start.is_null() || end.is_null() || start >= end {
        return;
    }

    let count = unsafe { end.offset_from(start) } as usize;
    for index in 0..count {
        if let Some(func) = unsafe { *start.add(index) } {
            invoke(func);
        }
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn run_initterm_e<F, Invoke>(
    start: *const Option<F>,
    end: *const Option<F>,
    mut invoke: Invoke,
) -> i32
where
    F: Copy,
    Invoke: FnMut(F) -> i32,
{
    if start.is_null() || end.is_null() || start >= end {
        return 0;
    }

    let count = unsafe { end.offset_from(start) } as usize;
    for index in 0..count {
        if let Some(func) = unsafe { *start.add(index) } {
            let result = invoke(func);
            if result != 0 {
                return result;
            }
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::{run_initterm, run_initterm_e};

    #[test]
    fn initterm_calls_entries() {
        use std::sync::atomic::{AtomicU32, Ordering};

        static COUNTER: AtomicU32 = AtomicU32::new(0);

        unsafe fn inc() {
            COUNTER.fetch_add(1, Ordering::Relaxed);
        }

        let table: [Option<unsafe fn()>; 3] = [Some(inc), None, Some(inc)];

        COUNTER.store(0, Ordering::Relaxed);
        unsafe {
            run_initterm(table.as_ptr(), table.as_ptr().add(table.len()), |func| {
                func();
            });
        }
        assert_eq!(COUNTER.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn initterm_tolerates_null_range() {
        unsafe {
            run_initterm::<unsafe fn(), _>(std::ptr::null(), std::ptr::null(), |_| {});
        }
    }

    #[test]
    fn initterm_e_stops_on_error() {
        use std::sync::atomic::{AtomicU32, Ordering};

        static COUNTER: AtomicU32 = AtomicU32::new(0);

        unsafe fn ok() -> i32 {
            COUNTER.fetch_add(1, Ordering::Relaxed);
            0
        }

        unsafe fn fail() -> i32 {
            COUNTER.fetch_add(1, Ordering::Relaxed);
            42
        }

        unsafe fn unreachable_init() -> i32 {
            COUNTER.fetch_add(100, Ordering::Relaxed);
            0
        }

        let table: [Option<unsafe fn() -> i32>; 3] = [Some(ok), Some(fail), Some(unreachable_init)];

        COUNTER.store(0, Ordering::Relaxed);
        let result = unsafe {
            run_initterm_e(table.as_ptr(), table.as_ptr().add(table.len()), |func| {
                func()
            })
        };
        assert_eq!(result, 42);
        assert_eq!(COUNTER.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn initterm_e_returns_zero_on_success() {
        unsafe fn ok() -> i32 {
            0
        }

        let table: [Option<unsafe fn() -> i32>; 2] = [Some(ok), Some(ok)];

        let result = unsafe {
            run_initterm_e(table.as_ptr(), table.as_ptr().add(table.len()), |func| {
                func()
            })
        };
        assert_eq!(result, 0);
    }
}
