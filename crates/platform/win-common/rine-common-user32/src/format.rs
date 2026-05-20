//! wsprintfA/W format string processing.
//!
//! Implements the shared logic for wsprintfA, which formats a string and
//! writes the result into a caller-supplied buffer. Both the 32-bit and
//! 64-bit DLL wrappers delegate to this module.

use core::ffi::CStr;

/// Maximum number of variadic arguments we support.
const MAX_ARGS: usize = 32;

/// No-mangle extern "C" entry point called by the platform-specific wrappers.
///
/// # Arguments
/// * `output`   - Destination buffer (must be large enough for the result).
/// * `format`   - Null-terminated ANSI format string.
/// * `args`     - Pointer to the first variadic argument slot.
/// * `slot_size` - Size of each argument slot in bytes (4 on x86, 8 on x64).
///
/// # Returns
/// Number of characters written, excluding the null terminator.
///
/// # Safety
/// * `output` must point to a valid writable buffer.
/// * `format` must point to a valid null-terminated ANSI string.
/// * `args` must point to at least `arg_count * slot_size` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rine_user32_wsprintf_a(
    output: *mut u8,
    format: *const u8,
    args: *const u8,
    slot_size: u32,
) -> i32 {
    if output.is_null() || format.is_null() {
        return 0;
    }

    let fmt_bytes = unsafe { CStr::from_ptr(format as *const i8) }.to_bytes();
    let mut out_pos: usize = 0;
    let mut arg_index: usize = 0;

    let mut i = 0;
    while i < fmt_bytes.len() {
        let b = fmt_bytes[i];
        if b != b'%' {
            unsafe { *output.add(out_pos) = b };
            out_pos += 1;
            i += 1;
            continue;
        }

        // We have a '%' — check for '%%' first.
        i += 1;
        if i >= fmt_bytes.len() {
            break;
        }

        if fmt_bytes[i] == b'%' {
            unsafe { *output.add(out_pos) = b'%' };
            out_pos += 1;
            i += 1;
            continue;
        }

        // Check for long modifier.
        let is_long = fmt_bytes[i] == b'l' || fmt_bytes[i] == b'L';
        if is_long {
            i += 1;
            if i >= fmt_bytes.len() {
                break;
            }
        }

        // Determine which arg slot to use (same slot regardless of long/short).
        if arg_index >= MAX_ARGS {
            break;
        }
        let slot = unsafe { args.add(arg_index * slot_size as usize) };
        arg_index += 1;

        let spec = fmt_bytes[i];
        i += 1;

        match spec {
            b'c' => {
                let ch = unsafe { *(slot as *const i32) as u8 };
                unsafe { *output.add(out_pos) = ch };
                out_pos += 1;
            }
            b'd' | b'i' => {
                let val = unsafe { *(slot as *const i32) };
                out_pos = write_i32(output, out_pos, val);
            }
            b'u' => {
                let val = unsafe { *(slot as *const u32) };
                out_pos = write_u32(output, out_pos, val);
            }
            b'x' | b'X' => {
                let val = unsafe { *(slot as *const u32) };
                let upper = spec == b'X';
                out_pos = write_hex(output, out_pos, val, upper);
            }
            b's' => {
                let ptr: *const u8 = if slot_size == 4 {
                    unsafe { *(slot as *const u32) as *const u8 }
                } else {
                    unsafe { *(slot as *const *const u8) }
                };
                if !ptr.is_null() {
                    let s = unsafe { CStr::from_ptr(ptr as *const i8) }.to_bytes();
                    unsafe {
                        core::ptr::copy_nonoverlapping(s.as_ptr(), output.add(out_pos), s.len());
                    }
                    out_pos += s.len();
                } else {
                    let null_str = b"(null)";
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            null_str.as_ptr(),
                            output.add(out_pos),
                            null_str.len(),
                        );
                    }
                    out_pos += null_str.len();
                }
            }
            _ => {
                unsafe { *output.add(out_pos) = b'%' };
                out_pos += 1;
                if is_long {
                    unsafe { *output.add(out_pos) = b'l' };
                    out_pos += 1;
                }
                unsafe { *output.add(out_pos) = spec };
                out_pos += 1;
            }
        }
    }

    unsafe { *output.add(out_pos) = 0 };
    out_pos as i32
}

fn write_i32(buf: *mut u8, pos: usize, val: i32) -> usize {
    if val == i32::MIN {
        let digits = b"-2147483648";
        unsafe {
            core::ptr::copy_nonoverlapping(digits.as_ptr(), buf.add(pos), digits.len());
        }
        return pos + digits.len();
    }

    let mut p = pos;
    if val < 0 {
        unsafe { *buf.add(p) = b'-' };
        p += 1;
        write_u32(buf, p, (-val) as u32)
    } else {
        write_u32(buf, p, val as u32)
    }
}

fn write_u32(buf: *mut u8, pos: usize, val: u32) -> usize {
    let mut temp = [0u8; 12];
    let mut idx = temp.len();
    let mut n = val;

    if n == 0 {
        unsafe { *buf.add(pos) = b'0' };
        return pos + 1;
    }

    while n > 0 {
        idx -= 1;
        temp[idx] = b'0' + (n % 10) as u8;
        n /= 10;
    }

    let len = temp.len() - idx;
    unsafe {
        core::ptr::copy_nonoverlapping(temp.as_ptr().add(idx), buf.add(pos), len);
    }
    pos + len
}

fn write_hex(buf: *mut u8, pos: usize, val: u32, upper: bool) -> usize {
    let mut temp = [0u8; 10];
    let mut idx = temp.len();
    let mut n = val;

    if n == 0 {
        unsafe { *buf.add(pos) = b'0' };
        return pos + 1;
    }

    while n > 0 {
        idx -= 1;
        let digit = (n & 0xF) as u8;
        temp[idx] = if upper {
            if digit < 10 {
                b'0' + digit
            } else {
                b'A' + digit - 10
            }
        } else {
            if digit < 10 {
                b'0' + digit
            } else {
                b'a' + digit - 10
            }
        };
        n >>= 4;
    }

    let len = temp.len() - idx;
    unsafe {
        core::ptr::copy_nonoverlapping(temp.as_ptr().add(idx), buf.add(pos), len);
    }
    pos + len
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run tests using a slot-size that matches the host pointer width.
    /// On a 32-bit host this is 4, on 64-bit it is 8.
    fn host_slot_size() -> u32 {
        core::mem::size_of::<usize>() as u32
    }

    mod basic {
        use super::*;

        #[test]
        fn null_output_returns_zero() {
            let fmt = b"hello\0";
            let result = unsafe {
                rine_user32_wsprintf_a(std::ptr::null_mut(), fmt.as_ptr(), std::ptr::null(), 4)
            };
            assert_eq!(result, 0);
        }

        #[test]
        fn null_format_returns_zero() {
            let mut buf = [0u8; 16];
            let result = unsafe {
                rine_user32_wsprintf_a(buf.as_mut_ptr(), std::ptr::null(), std::ptr::null(), 4)
            };
            assert_eq!(result, 0);
        }

        #[test]
        fn literal_only() {
            let fmt = b"hello world\0";
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(buf.as_mut_ptr(), fmt.as_ptr(), std::ptr::null(), 4)
            };
            assert_eq!(result, 11);
            assert_eq!(&buf[..12], b"hello world\0");
        }

        #[test]
        fn literal_percent() {
            let fmt = b"100%%\0";
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(buf.as_mut_ptr(), fmt.as_ptr(), std::ptr::null(), 4)
            };
            assert_eq!(result, 4);
            assert_eq!(&buf[..5], b"100%\0");
        }
    }

    mod integer {
        use super::*;

        #[test]
        fn format_d() {
            let fmt = b"%d\0";
            let args = 42i32;
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(
                    buf.as_mut_ptr(),
                    fmt.as_ptr(),
                    &args as *const i32 as *const u8,
                    4,
                )
            };
            assert_eq!(result, 2);
            assert_eq!(&buf[..3], b"42\0");
        }

        #[test]
        fn format_d_negative() {
            let fmt = b"%d\0";
            let args = -123i32;
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(
                    buf.as_mut_ptr(),
                    fmt.as_ptr(),
                    &args as *const i32 as *const u8,
                    4,
                )
            };
            assert_eq!(result, 4);
            assert_eq!(&buf[..5], b"-123\0");
        }

        #[test]
        fn format_d_i32_min() {
            let fmt = b"%d\0";
            let args = i32::MIN;
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(
                    buf.as_mut_ptr(),
                    fmt.as_ptr(),
                    &args as *const i32 as *const u8,
                    4,
                )
            };
            assert_eq!(result, 11);
            assert_eq!(&buf[..12], b"-2147483648\0");
        }

        #[test]
        fn format_u() {
            let fmt = b"%u\0";
            let args = 300u32;
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(
                    buf.as_mut_ptr(),
                    fmt.as_ptr(),
                    &args as *const u32 as *const u8,
                    4,
                )
            };
            assert_eq!(result, 3);
            assert_eq!(&buf[..4], b"300\0");
        }

        #[test]
        fn format_x() {
            let fmt = b"%x\0";
            let args = 255u32;
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(
                    buf.as_mut_ptr(),
                    fmt.as_ptr(),
                    &args as *const u32 as *const u8,
                    4,
                )
            };
            assert_eq!(result, 2);
            assert_eq!(&buf[..3], b"ff\0");
        }

        #[test]
        fn format_uppercase_x() {
            let fmt = b"%X\0";
            let args = 0xABCDu32;
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(
                    buf.as_mut_ptr(),
                    fmt.as_ptr(),
                    &args as *const u32 as *const u8,
                    4,
                )
            };
            assert_eq!(result, 4);
            assert_eq!(&buf[..5], b"ABCD\0");
        }

        #[test]
        fn format_zero() {
            let fmt = b"%d\0";
            let args = 0i32;
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(
                    buf.as_mut_ptr(),
                    fmt.as_ptr(),
                    &args as *const i32 as *const u8,
                    4,
                )
            };
            assert_eq!(result, 1);
            assert_eq!(&buf[..2], b"0\0");
        }

        #[test]
        fn format_char() {
            let fmt = b"%c\0";
            let args = 0x41i32;
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(
                    buf.as_mut_ptr(),
                    fmt.as_ptr(),
                    &args as *const i32 as *const u8,
                    4,
                )
            };
            assert_eq!(result, 1);
            assert_eq!(&buf[..2], b"A\0");
        }

        #[test]
        fn format_long_d() {
            let fmt = b"%ld\0";
            let args = 12345678i32;
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(
                    buf.as_mut_ptr(),
                    fmt.as_ptr(),
                    &args as *const i32 as *const u8,
                    4,
                )
            };
            assert_eq!(result, 8);
            assert_eq!(&buf[..9], b"12345678\0");
        }
    }

    mod string {
        use super::*;

        #[test]
        fn format_s() {
            let ss = host_slot_size();
            let fmt = b"hello %s\0";
            let s = b"world\0";
            let ptr: usize = s.as_ptr() as *const u8 as usize;
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(
                    buf.as_mut_ptr(),
                    fmt.as_ptr(),
                    &ptr as *const usize as *const u8,
                    ss,
                )
            };
            assert_eq!(result, 11);
            assert_eq!(&buf[..12], b"hello world\0");
        }

        #[test]
        fn format_s_null() {
            let ss = host_slot_size();
            let fmt = b"%s\0";
            let ptr: usize = 0;
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(
                    buf.as_mut_ptr(),
                    fmt.as_ptr(),
                    &ptr as *const usize as *const u8,
                    ss,
                )
            };
            assert_eq!(result, 6);
            assert_eq!(&buf[..7], b"(null)\0");
        }
    }

    mod multi_arg {
        use super::*;

        #[test]
        fn multiple_args() {
            let fmt = b"%d + %d = %d\0";
            let args: [i32; 3] = [1, 2, 3];
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(
                    buf.as_mut_ptr(),
                    fmt.as_ptr(),
                    args.as_ptr() as *const u8,
                    4,
                )
            };
            assert_eq!(result, 9);
            assert_eq!(&buf[..10], b"1 + 2 = 3\0");
        }

        #[test]
        fn mixed_types() {
            let ss = host_slot_size();
            let fmt = b"%s = %d\0";
            let name = b"answer\0";
            let s_ptr: usize = name.as_ptr() as *const u8 as usize;
            let int_val: i32 = 42;
            let slot_size = ss as usize;
            let mut slot_storage = vec![0u8; slot_size * 2];
            if slot_size == 4 {
                slot_storage[..4].copy_from_slice(&(s_ptr as u32).to_ne_bytes());
                slot_storage[4..8].copy_from_slice(&int_val.to_ne_bytes());
            } else {
                slot_storage[..8].copy_from_slice(&(s_ptr as u64).to_ne_bytes());
                slot_storage[8..16].copy_from_slice(&(int_val as i64).to_ne_bytes());
            }
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(buf.as_mut_ptr(), fmt.as_ptr(), slot_storage.as_ptr(), ss)
            };
            assert_eq!(result, 11);
            assert_eq!(&buf[..12], b"answer = 42\0");
        }
    }

    mod x64_slots {
        use super::*;

        #[test]
        fn format_d_64bit_slot() {
            let fmt = b"%d\0";
            let val: i64 = 42;
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(
                    buf.as_mut_ptr(),
                    fmt.as_ptr(),
                    &val as *const i64 as *const u8,
                    8,
                )
            };
            assert_eq!(result, 2);
            assert_eq!(&buf[..3], b"42\0");
        }

        #[test]
        fn format_s_64bit_slot() {
            let fmt = b"%s\0";
            let s = b"hello\0";
            let ptr: *const u8 = s.as_ptr();
            let val: u64 = ptr as u64;
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(
                    buf.as_mut_ptr(),
                    fmt.as_ptr(),
                    &val as *const u64 as *const u8,
                    8,
                )
            };
            assert_eq!(result, 5);
            assert_eq!(&buf[..6], b"hello\0");
        }

        #[test]
        fn multiple_args_64bit_slot() {
            let fmt = b"%d + %d = %d\0";
            let args: [i64; 3] = [10, 20, 30];
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(
                    buf.as_mut_ptr(),
                    fmt.as_ptr(),
                    args.as_ptr() as *const u8,
                    8,
                )
            };
            assert_eq!(result, 12);
            assert_eq!(&buf[..13], b"10 + 20 = 30\0");
        }
    }

    mod unknown_specifier {
        use super::*;

        #[test]
        fn unknown_specifier_written_literally() {
            let fmt = b"%q\0";
            let mut buf = [0u8; 64];
            let result = unsafe {
                rine_user32_wsprintf_a(buf.as_mut_ptr(), fmt.as_ptr(), std::ptr::null(), 4)
            };
            assert_eq!(result, 2);
            assert_eq!(&buf[..3], b"%q\0");
        }
    }
}
