pub fn nt_terminate_process() -> u32 {
    tracing::warn!(
        api = "NtTerminateProcess",
        dll = "ntdll",
        "NtTerminateProcess stub called. Returned success"
    );
    0
}

pub fn rtl_init_unicode_string() -> u32 {
    tracing::warn!(
        api = "RtlInitUnicodeString",
        dll = "ntdll",
        "RtlInitUnicodeString stub called. Returned success"
    );
    0
}

pub fn rtl_get_version() -> u32 {
    tracing::warn!(
        api = "RtlGetVersion",
        dll = "ntdll",
        "RtlGetVersion stub called. Returned success"
    );
    0
}
