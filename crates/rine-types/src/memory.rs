use core::fmt;

// ---------------------------------------------------------------------------
// VirtualAddress
// ---------------------------------------------------------------------------

/// A virtual memory address in the loaded PE image's address space.
///
/// Wraps a `usize` to prevent accidental mixing with RVAs, file offsets,
/// sizes, or other integer quantities.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
    /// The null/zero address.
    pub const NULL: Self = Self(0);

    /// Create a `VirtualAddress` from a raw `usize`.
    #[inline]
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }

    /// Create a `VirtualAddress` from a raw pointer.
    #[inline]
    pub fn from_ptr<T>(ptr: *const T) -> Self {
        Self(ptr as usize)
    }

    /// Return the raw `usize` value.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0
    }

    /// Return as a raw `*const u8`.
    #[inline]
    pub const fn as_ptr(self) -> *const u8 {
        self.0 as *const u8
    }

    /// Return as a raw `*mut u8`.
    #[inline]
    pub const fn as_mut_ptr(self) -> *mut u8 {
        self.0 as *mut u8
    }

    /// Return true if this is the null address.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Add an arbitrary byte offset.
    #[inline]
    pub const fn add(self, bytes: usize) -> Self {
        Self(self.0 + bytes)
    }

    /// Compute the signed delta from `other` to `self` (`self - other`).
    #[inline]
    pub const fn delta(self, other: VirtualAddress) -> i64 {
        self.0 as i64 - other.0 as i64
    }

    /// Align down to a page boundary.
    #[inline]
    pub const fn align_down(self, alignment: usize) -> Self {
        Self(self.0 & !(alignment - 1))
    }

    /// Align up to a page boundary.
    #[inline]
    pub const fn align_up(self, alignment: usize) -> Self {
        Self((self.0 + alignment - 1) & !(alignment - 1))
    }
}

impl fmt::Debug for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VA({:#x})", self.0)
    }
}

impl fmt::Display for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::LowerHex for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

// ---------------------------------------------------------------------------
// RelativeVirtualAddress (RVA)
// ---------------------------------------------------------------------------

/// A 32-bit offset relative to the image base in a PE file.
///
/// RVAs come directly from PE headers (section virtual addresses, entry point,
/// relocation block RVAs, etc.). Combine with a `BaseAddress` to produce an
/// absolute `VirtualAddress`: `base.offset(rva)` or `base + rva`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct RelativeVirtualAddress(u32);

impl RelativeVirtualAddress {
    pub const ZERO: Self = Self(0);

    #[inline]
    pub const fn new(rva: u32) -> Self {
        Self(rva)
    }

    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Add a byte offset to this RVA (e.g., block RVA + relocation offset).
    #[inline]
    pub const fn add(self, offset: u32) -> Self {
        Self(self.0 + offset)
    }
}

impl fmt::Debug for RelativeVirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RVA({:#x})", self.0)
    }
}

impl fmt::Display for RelativeVirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::LowerHex for RelativeVirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for RelativeVirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

// ---------------------------------------------------------------------------
// BaseAddress
// ---------------------------------------------------------------------------

/// The base address of a loaded PE image in virtual memory.
///
/// Distinct from `VirtualAddress` to enforce that only image bases are used
/// where an image base is expected (e.g., relocation delta, mmap base).
/// Combine with a `RelativeVirtualAddress` to get a `VirtualAddress`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct BaseAddress(usize);

impl BaseAddress {
    pub const NULL: Self = Self(0);

    #[inline]
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }

    #[inline]
    pub fn from_ptr<T>(ptr: *const T) -> Self {
        Self(ptr as usize)
    }

    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0
    }

    #[inline]
    pub const fn as_ptr(self) -> *const u8 {
        self.0 as *const u8
    }

    #[inline]
    pub const fn as_mut_ptr(self) -> *mut u8 {
        self.0 as *mut u8
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Resolve an RVA to an absolute virtual address.
    #[inline]
    pub const fn offset(self, rva: RelativeVirtualAddress) -> VirtualAddress {
        VirtualAddress::new(self.0 + rva.0 as usize)
    }

    /// Compute the signed delta from `other` to `self` (`self - other`).
    #[inline]
    pub const fn delta(self, other: BaseAddress) -> i64 {
        self.0 as i64 - other.0 as i64
    }
}

impl core::ops::Add<RelativeVirtualAddress> for BaseAddress {
    type Output = VirtualAddress;

    #[inline]
    fn add(self, rva: RelativeVirtualAddress) -> VirtualAddress {
        self.offset(rva)
    }
}

impl fmt::Debug for BaseAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Base({:#x})", self.0)
    }
}

impl fmt::Display for BaseAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::LowerHex for BaseAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for BaseAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

// ---------------------------------------------------------------------------
// FileOffset
// ---------------------------------------------------------------------------

/// A 32-bit byte offset into the raw PE file on disk.
///
/// Used for section `pointer_to_raw_data` and similar fields to prevent
/// mixing file positions with virtual addresses or RVAs.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct FileOffset(u32);

impl FileOffset {
    pub const ZERO: Self = Self(0);

    #[inline]
    pub const fn new(offset: u32) -> Self {
        Self(offset)
    }

    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Debug for FileOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileOff({:#x})", self.0)
    }
}

impl fmt::Display for FileOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::LowerHex for FileOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for FileOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- VirtualAddress --

    #[test]
    fn va_null_is_zero() {
        assert!(VirtualAddress::NULL.is_null());
        assert_eq!(VirtualAddress::NULL.as_usize(), 0);
    }

    #[test]
    fn va_add_bytes() {
        let addr = VirtualAddress::new(0x1000);
        assert_eq!(addr.add(0x200).as_usize(), 0x1200);
    }

    #[test]
    fn va_delta() {
        let a = VirtualAddress::new(0x7f0000000000);
        let b = VirtualAddress::new(0x140000000);
        assert_eq!(a.delta(b), 0x7f0000000000_i64 - 0x140000000_i64);
    }

    #[test]
    fn va_align_down_and_up() {
        let addr = VirtualAddress::new(0x1234);
        assert_eq!(addr.align_down(0x1000).as_usize(), 0x1000);
        assert_eq!(addr.align_up(0x1000).as_usize(), 0x2000);
    }

    #[test]
    fn va_debug_format() {
        let addr = VirtualAddress::new(0xDEAD);
        assert_eq!(format!("{addr:?}"), "VA(0xdead)");
    }

    #[test]
    fn va_from_ptr_roundtrip() {
        let val: u64 = 42;
        let addr = VirtualAddress::from_ptr(&val as *const u64);
        assert_eq!(addr.as_ptr(), &val as *const u64 as *const u8);
    }

    // -- BaseAddress --

    #[test]
    fn base_offset_produces_va() {
        let base = BaseAddress::new(0x140000000);
        let rva = RelativeVirtualAddress::new(0x1000);
        let va = base.offset(rva);
        assert_eq!(va.as_usize(), 0x140001000);
    }

    #[test]
    fn base_add_rva_operator() {
        let base = BaseAddress::new(0x140000000);
        let rva = RelativeVirtualAddress::new(0x2000);
        let va = base + rva;
        assert_eq!(va.as_usize(), 0x140002000);
    }

    #[test]
    fn base_delta() {
        let actual = BaseAddress::new(0x7f0000000000);
        let preferred = BaseAddress::new(0x140000000);
        assert_eq!(
            actual.delta(preferred),
            0x7f0000000000_i64 - 0x140000000_i64
        );
    }

    #[test]
    fn base_debug_format() {
        let base = BaseAddress::new(0xBEEF);
        assert_eq!(format!("{base:?}"), "Base(0xbeef)");
    }

    // -- RelativeVirtualAddress --

    #[test]
    fn rva_add_offset() {
        let block_rva = RelativeVirtualAddress::new(0x3000);
        let combined = block_rva.add(0x10);
        assert_eq!(combined.as_u32(), 0x3010);
    }

    #[test]
    fn rva_debug_format() {
        let rva = RelativeVirtualAddress::new(0x1000);
        assert_eq!(format!("{rva:?}"), "RVA(0x1000)");
    }

    // -- FileOffset --

    #[test]
    fn file_offset_conversions() {
        let off = FileOffset::new(0x400);
        assert_eq!(off.as_u32(), 0x400);
        assert_eq!(off.as_usize(), 0x400);
    }

    #[test]
    fn file_offset_debug_format() {
        let off = FileOffset::new(0x200);
        assert_eq!(format!("{off:?}"), "FileOff(0x200)");
    }
}
