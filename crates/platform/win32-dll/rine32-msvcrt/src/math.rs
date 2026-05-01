//! msvcrt math functions.

use rine_common_msvcrt as common;

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
#[rine_dlls::implemented]
pub unsafe extern "C" fn sqrt(x: f64) -> f64 {
    common::sqrt(x)
}
