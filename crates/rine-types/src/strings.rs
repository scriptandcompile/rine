//! Windows string types and conversion helpers.

use core::fmt;

/// NT UNICODE_STRING — a counted (not null-terminated) wide-character string
/// used throughout the NT kernel API.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UnicodeString {
    /// Length in **bytes** (not characters), excluding any trailing null.
    pub length: u16,
    /// Maximum length in **bytes** of the buffer.
    pub maximum_length: u16,
    /// Pointer to a UTF-16LE buffer.
    pub buffer: *mut u16,
}

impl UnicodeString {
    pub const fn empty() -> Self {
        Self {
            length: 0,
            maximum_length: 0,
            buffer: core::ptr::null_mut(),
        }
    }
}

impl fmt::Debug for UnicodeString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UNICODE_STRING(len={}, max={}, buf={:?})",
            self.length, self.maximum_length, self.buffer,
        )
    }
}

/// Win32-style null-terminated string pointers, used in many APIs for input and output strings.
/// 'LPCSTR' stands for "Long Pointer to Constant String", a historical Windows convention.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LPCSTR(*const u8);

impl LPCSTR {
    /// A null pointer representing an empty string.
    pub const NULL: Self = Self(core::ptr::null());

    /// Check if the pointer is null.
    ///
    /// # Returns
    /// `true` if the pointer is null, `false` otherwise.
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get the raw pointer value.
    ///
    /// # Safety
    /// The caller must ensure that the pointer is valid for the intended use.
    /// This function does not perform any safety checks.
    ///
    /// # Returns
    /// The raw pointer as a `*const u8`.
    pub fn as_ptr(self) -> *const u8 {
        self.0
    }

    /// Check whether the pointer is an integer resource identifier (IS_INTRESOURCE).
    ///
    /// In Win32, resource identifiers are encoded as pointers whose high 16 bits are zero,
    /// with the resource ID stored in the low 16 bits. This is used by macros such as
    /// `MAKEINTRESOURCE` and functions like `LoadCursor`/`LoadIcon`/`LoadImage`.
    ///
    /// # Returns
    /// `true` if the pointer is non-null and its high 16 bits are all zero,
    /// indicating an integer resource identifier.
    pub fn is_int_resource(self) -> bool {
        !self.is_null() && (self.0 as usize >> 16) == 0
    }

    /// Extract the low 16 bits as a resource ID when `is_int_resource` is true.
    ///
    /// # Returns
    /// The resource ID as a `u16`.
    pub fn to_int_resource(self) -> u16 {
        self.0 as u16
    }

    /// Read a string from the pointer location, treating it as a null-terminated ANSI string.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid null-terminated byte string.
    /// If this is not the case, this function may cause undefined behavior.
    /// The caller is responsible for ensuring that the pointer is properly aligned for `u8` access.
    ///
    /// # Returns
    /// An `Option<String>` containing the read string, or `None` if the pointer is null or if the string is not valid UTF-8.
    pub unsafe fn read_string(self) -> Option<String> {
        unsafe { text::read_cstr(self.0) }
    }

    /// Read a string from the pointer location with an explicit character count, treating it as an ANSI string.
    ///
    /// # Arguments
    /// * `count` - The number of characters to read from the pointer. This does not include any null terminator and may be zero.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid byte string of at least `count` bytes when `count > 0`.
    /// If these conditions are not met, this function may cause undefined behavior.
    /// The caller is responsible for ensuring that the pointer is properly aligned for `u8` access.
    ///
    /// # Returns
    /// An `Option<String>` containing the read string, or `None` if the pointer is null, if `count` is negative, or if the string is not valid UTF-8.
    pub unsafe fn read_string_counted(self, count: i32) -> Option<String> {
        unsafe { text::read_cstr_counted(self.0, count) }
    }

    /// Write a string from the pointer location into a caller-supplied buffer, treating the pointer as an input string.
    ///
    /// # Arguments
    /// * `buf` - A pointer to a writable buffer where the string will be written as a null-terminated byte string.
    /// * `buf_size` - The size of the buffer in bytes. The function will write at most `buf_size - 1` characters plus a null terminator.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid null-terminated byte string.
    /// The caller must also ensure that `buf` is null or points to a valid writable buffer of at least `buf_size` bytes.
    /// If these conditions are not met, this function may cause undefined behavior.
    /// The caller is responsible for ensuring that the buffer is properly aligned for `u8` access.
    ///
    /// # Returns
    /// The number of characters written (excluding the null terminator), or the required buffer size (including the null terminator)
    /// if the pointer is null, the buffer is null, or the buffer is too small to hold the string.
    pub unsafe fn write_to_buffer(self, buf: *mut u8, buf_size: u32) -> Option<u32> {
        if self.is_null() {
            return None;
        }

        unsafe {
            let s = self.read_string()?;
            let written = text::write_cstr(buf, buf_size, &s);
            Some(written)
        }
    }

    /// Write a string directly to the pointer location, treating it as an output buffer.
    ///
    /// # Arguments
    /// * `value` - The string to write. This will be encoded as a null-terminated byte string.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid writable buffer of at
    /// least `value.len() + 1` bytes (enough for the string and null terminator).
    /// If this is not the case, this function may cause undefined behavior.
    /// The caller is also responsible for ensuring that the buffer is properly aligned for `u8` access.
    ///
    /// # Returns
    /// The number of characters written (excluding the null terminator), or the required buffer size
    /// (including the null terminator) if the pointer is null or the buffer is too small.
    pub unsafe fn write_string(self, value: &str) -> Option<u32> {
        if self.is_null() {
            return None;
        }

        unsafe {
            let written = text::write_cstr(self.0 as *mut u8, value.len() as u32 + 1, value);
            Some(written)
        }
    }
}

/// Win32-style null-terminated string pointers, used in many APIs for output strings.
/// 'LPSTR' stands for "Long Pointer to String", a historical Windows convention.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LPSTR(*mut u8);

impl LPSTR {
    /// A null pointer representing an empty string.
    pub const NULL: Self = Self(core::ptr::null_mut());

    /// Check if the pointer is null.
    ///
    /// # Returns
    /// `true` if the pointer is null, `false` otherwise.
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get the raw pointer value.
    pub fn as_mut_ptr(self) -> *mut u8 {
        self.0
    }
}

/// Win32-style null-terminated string pointers, used in many APIs for output strings.
/// 'LPWSTR' stands for "Long Pointer to Wide String", a historical Windows convention.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LPWSTR(*mut u16);

impl LPWSTR {
    /// A null pointer representing an empty string.
    pub const NULL: Self = Self(core::ptr::null_mut());

    /// Check if the pointer is null.
    ///
    /// # Returns
    /// `true` if the pointer is null, `false` otherwise.
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get the raw pointer value.
    pub fn as_mut_ptr(self) -> *mut u16 {
        self.0
    }
}

/// Win32-style null-terminated wide string pointers, used in many APIs for input and output strings.
/// 'LPCWSTR' stands for "Long Pointer to Constant Wide String", a historical Windows convention for UTF-16 strings.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LPCWSTR(*const u16);

impl LPCWSTR {
    /// A null pointer representing an empty string.
    pub const NULL: Self = Self(core::ptr::null());

    /// Check if the pointer is null.
    ///
    /// # Returns
    /// `true` if the pointer is null, `false` otherwise.
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get the raw pointer value.
    ///
    /// # Safety
    /// The caller must ensure that the pointer is valid for the intended use.
    /// This function does not perform any safety checks.
    ///
    /// # Returns
    /// The raw pointer as a `*const u16`.
    pub fn as_ptr(self) -> *const u16 {
        self.0
    }

    /// Check whether the pointer is an integer resource identifier (IS_INTRESOURCE).
    ///
    /// In Win32, resource identifiers are encoded as pointers whose high 16 bits are zero,
    /// with the resource ID stored in the low 16 bits. This is used by macros such as
    /// `MAKEINTRESOURCE` and functions like `LoadCursor`/`LoadIcon`/`LoadImage`.
    ///
    /// # Returns
    /// `true` if the pointer is non-null and its high 16 bits are all zero,
    /// indicating an integer resource identifier.
    pub fn is_int_resource(self) -> bool {
        !self.is_null() && (self.0 as usize >> 16) == 0
    }

    /// Extract the low 16 bits as a resource ID when `is_int_resource` is true.
    ///
    /// # Returns
    /// The resource ID as a `u16`.
    pub fn to_int_resource(self) -> u16 {
        self.0 as u16
    }

    /// Read a string from the pointer location, treating it as a null-terminated UTF-16LE string.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid null-terminated UTF-16 string.
    /// If this is not the case, this function may cause undefined behavior.
    /// The caller is responsible for ensuring that the pointer is properly aligned for `u16` access.
    ///
    /// # Returns
    /// An `Option<String>` containing the read string, or `None` if the pointer is null or if the string is not valid UTF-16.
    pub unsafe fn read_string(self) -> Option<String> {
        unsafe { text::read_wstr(self.0) }
    }

    /// Read a string from the pointer location with an explicit character count, treating it as a wide string.
    ///
    /// # Arguments
    /// * `count` - The number of characters to read from the pointer. This does not include any null terminator and may be zero.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid wide string of at least `count` characters when `count > 0`.
    /// If these conditions are not met, this function may cause undefined behavior.
    /// The caller is responsible for ensuring that the pointer is properly aligned for `u16` access.
    ///
    /// # Returns
    /// An `Option<String>` containing the read string, or `None` if the pointer is null, if `count` is negative, or if the string is not valid UTF-16.
    pub unsafe fn read_string_counted(self, count: i32) -> Option<String> {
        unsafe { text::read_wstr_counted(self.0, count) }
    }

    /// Write a string from the pointer location into a caller-supplied buffer, treating the pointer as an input string.
    ///
    /// # Arguments
    /// * `buf` - A pointer to a writable buffer where the string will be written as UTF-16LE with a null terminator.
    /// * `buf_size` - The size of the buffer in `u16` elements (not bytes).
    ///   The function will write at most `buf_size - 1` characters plus a null terminator.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid null-terminated UTF-16 string.
    /// The caller must also ensure that `buf` is null or points to a valid writable buffer of at least `buf_size` `u16` elements.
    /// If these conditions are not met, this function may cause undefined behavior.
    /// The caller is responsible for ensuring that the buffer is properly aligned for `u16` access.
    ///
    /// # Returns
    /// The number of characters written (excluding the null terminator), or the required buffer size (including the null terminator)
    /// if the pointer is null, the buffer is null, or the buffer is too small to hold the string.
    pub unsafe fn write_to_buffer(self, buf: *mut u16, buf_size: u32) -> Option<u32> {
        if self.is_null() {
            return None;
        }
        unsafe {
            let s = self.read_string()?;
            let written = text::write_wstr(buf, buf_size, &s);
            Some(written)
        }
    }

    /// Write a string directly to the pointer location, treating it as an output buffer.
    ///
    /// # Arguments
    /// * `value` - The string to write. This will be encoded as UTF-16LE and null-terminated.
    ///
    /// # Safety
    /// The caller must ensure that `self.0` is null or points to a valid writable buffer of at
    /// least `value.len() * 2 + 2` bytes (enough for the UTF-16 encoding and null terminator).
    /// If this is not the case, this function may cause undefined behavior.
    /// The caller is also responsible for ensuring that the buffer is properly aligned for `u16` access.
    ///
    /// # Returns
    /// The number of characters written (excluding the null terminator), or the
    /// required buffer size (including the null terminator) if the pointer is null or the buffer is too small.
    pub unsafe fn write_string(self, value: &str) -> Option<u32> {
        if self.is_null() {
            return None;
        }
        unsafe {
            let written = text::write_wstr(self.0 as *mut u16, value.len() as u32 + 1, value);
            Some(written)
        }
    }
}
