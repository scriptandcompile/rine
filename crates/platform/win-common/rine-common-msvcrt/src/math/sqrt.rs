use crate::crt_support::{CRT_DOMAIN_ERROR, CrtMathException, invoke_user_math_err};
use crate::errno_location;

const SQRT_NAME: &[u8] = b"sqrt\0";

/// Compute the square root of `x`.
///
/// # Arguments
/// * `x` - The value to compute the square root of.
///
/// # Safety
/// This function is safe to call with any `f64` value.
/// However, if `x` is negative, it will set `errno` to `EDOM` and invoke the user math error handler if one is registered,
/// which may have side effects depending on the handler's implementation.
///
/// # Return values
/// The square root of `x`, or `NaN` if `x` is negative.
/// In the case of a negative input, `errno` is set to `EDOM` and the user math error handler is invoked if one is registered.
pub fn sqrt(value: f64) -> f64 {
    if value < 0.0 {
        unsafe {
            *errno_location() = libc::EDOM;
        }

        let mut exception = CrtMathException {
            type_: CRT_DOMAIN_ERROR,
            name: SQRT_NAME.as_ptr() as *const i8,
            arg1: value,
            arg2: 0.0,
            retval: f64::NAN,
        };

        unsafe {
            let _ = invoke_user_math_err(&mut exception as *mut CrtMathException);
        }

        return exception.retval;
    }

    value.sqrt()
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::sqrt;
    use crate::crt_support::{CrtMathException, set_user_math_err};
    use crate::errno_location;

    static MATH_TEST_LOCK: Mutex<()> = Mutex::new(());
    static HANDLER_CALLS: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn test_matherr_handler(_exception: *mut CrtMathException) -> i32 {
        HANDLER_CALLS.fetch_add(1, Ordering::SeqCst);
        1
    }

    #[test]
    fn sqrt_positive_is_computed() {
        let _guard = MATH_TEST_LOCK.lock().unwrap();
        set_user_math_err(0);
        assert_eq!(sqrt(25.0), 5.0);
    }

    #[test]
    fn sqrt_negative_sets_errno_and_returns_nan() {
        let _guard = MATH_TEST_LOCK.lock().unwrap();
        set_user_math_err(0);
        unsafe {
            *errno_location() = 0;
        }

        let result = sqrt(-1.0);

        assert!(result.is_nan());
        assert_eq!(unsafe { *errno_location() }, libc::EDOM);
    }

    #[test]
    fn sqrt_negative_invokes_registered_matherr_handler() {
        let _guard = MATH_TEST_LOCK.lock().unwrap();
        HANDLER_CALLS.store(0, Ordering::SeqCst);
        set_user_math_err(test_matherr_handler as *const () as usize);

        let _ = sqrt(-9.0);

        assert_eq!(HANDLER_CALLS.load(Ordering::SeqCst), 1);
        set_user_math_err(0);
    }
}
