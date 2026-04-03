use std::sync::LazyLock;

pub fn set_app_type(_app_type: i32) {}

pub fn set_usermatherr(_handler: usize) {}

pub fn c_specific_handler_result() -> i32 {
    1
}

pub fn fake_iob_32_ptr() -> *mut u8 {
    FAKE_IOB_32.as_ptr() as *mut u8
}

pub fn fake_iob_64_ptr() -> *mut u8 {
    FAKE_IOB_64.as_ptr() as *mut u8
}

pub fn onexit(func: usize) -> usize {
    func
}

pub fn amsg_exit(msg_num: i32) -> ! {
    eprintln!("rine: msvcrt runtime error (msg_num={msg_num})");
    std::process::abort();
}

pub fn abort_process() -> ! {
    std::process::abort();
}

pub fn signal_default(_sig: i32, _handler: usize) -> usize {
    0
}

pub fn lock(_locknum: i32) {}

pub fn unlock(_locknum: i32) {}

pub fn errno_location() -> *mut i32 {
    unsafe { libc::__errno_location() }
}

fn build_fake_iob<const SIZE: usize, const ENTRY_SIZE: usize>() -> Box<[u8; SIZE]> {
    let mut buf = Box::new([0u8; SIZE]);
    buf[0..4].copy_from_slice(&0i32.to_ne_bytes());
    buf[ENTRY_SIZE..ENTRY_SIZE + 4].copy_from_slice(&1i32.to_ne_bytes());
    buf[ENTRY_SIZE * 2..ENTRY_SIZE * 2 + 4].copy_from_slice(&2i32.to_ne_bytes());
    buf
}

static FAKE_IOB_32: LazyLock<Box<[u8; 96]>> = LazyLock::new(build_fake_iob::<96, 32>);
static FAKE_IOB_64: LazyLock<Box<[u8; 144]>> = LazyLock::new(build_fake_iob::<144, 48>);

#[cfg(test)]
mod tests {
    use super::{fake_iob_32_ptr, fake_iob_64_ptr};

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
}
