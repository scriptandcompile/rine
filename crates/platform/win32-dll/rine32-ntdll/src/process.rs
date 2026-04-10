#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn NtTerminateProcess() -> u32 {
    tracing::warn!(
        api = "NtTerminateProcess",
        dll = "ntdll",
        "win32 stub called"
    );
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn RtlInitUnicodeString() -> u32 {
    tracing::warn!(
        api = "RtlInitUnicodeString",
        dll = "ntdll",
        "win32 stub called"
    );
    0
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn RtlGetVersion() -> u32 {
    tracing::warn!(api = "RtlGetVersion", dll = "ntdll", "win32 stub called");
    0
}
