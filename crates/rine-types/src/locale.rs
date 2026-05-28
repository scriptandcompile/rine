#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct LCID(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct LCTYPE(pub u32);

// SAFETY: The `lconv` struct is always used behind a process-lifetime static.
// The raw pointers within point to leaked, never-mutated C strings, making
// the struct effectively read-only after construction.
unsafe impl Send for lconv {}
unsafe impl Sync for lconv {}

/// C-compatible `struct lconv` describing numeric and monetary formatting
/// conventions for the current locale, as returned by `localeconv()`.
///
/// # Layout
/// This struct must match the MSVCRT definition of `struct lconv` from
/// `<locale.h>`. String fields are null-terminated C strings. The `CHAR_MAX`
/// value (127) in a numeric field indicates "not available" in the C locale.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct lconv {
    pub decimal_point: *mut libc::c_char,
    pub thousands_sep: *mut libc::c_char,
    pub grouping: *mut libc::c_char,
    pub int_curr_symbol: *mut libc::c_char,
    pub currency_symbol: *mut libc::c_char,
    pub mon_decimal_point: *mut libc::c_char,
    pub mon_thousands_sep: *mut libc::c_char,
    pub mon_grouping: *mut libc::c_char,
    pub positive_sign: *mut libc::c_char,
    pub negative_sign: *mut libc::c_char,
    pub int_frac_digits: libc::c_char,
    pub frac_digits: libc::c_char,
    pub p_cs_precedes: libc::c_char,
    pub p_sep_by_space: libc::c_char,
    pub n_cs_precedes: libc::c_char,
    pub n_sep_by_space: libc::c_char,
    pub p_sign_posn: libc::c_char,
    pub n_sign_posn: libc::c_char,
    pub int_n_cs_precedes: libc::c_char,
    pub int_n_sep_by_space: libc::c_char,
    pub int_n_sign_posn: libc::c_char,
    pub int_p_cs_precedes: libc::c_char,
    pub int_p_sep_by_space: libc::c_char,
    pub int_p_sign_posn: libc::c_char,
}
