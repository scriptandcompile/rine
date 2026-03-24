# rine — Design Plan

## TL;DR

**rine** is a from-scratch Windows PE executable loader for Linux, written in Rust. It translates Windows NT syscalls to Linux syscalls in userspace and provides Rust reimplementations of core Windows DLLs. The UX goal: double-click a .exe and it runs. Right-click opens a Tauri-based config editor for per-program settings. A companion project **rine-compat** is a community-driven git repo of compatibility profiles and ratings.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│                  User clicks .exe               │
│         (binfmt_misc / .desktop handler)        │
└──────────────────────┬──────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────┐
│                 rine (main binary)              │
│  ┌─────────────┐  ┌──────────────┐  ┌────────┐  │
│  │  PE Parser  │  │ Config Mgr   │  │ Logger │  │
│  │  & Loader   │  │ (per-app)    │  │        │  │
│  └──────┬──────┘  └──────┬───────┘  └────────┘  │
│         │                │                      │
│  ┌──────▼────────────────▼───────────────────┐  │
│  │         Virtual Address Space Manager     │  │
│  │   (mmap PE sections, handle relocations)  │  │
│  └──────────────────┬────────────────────────┘  │
│                     │                           │
│  ┌──────────────────▼────────────────────────┐  │
│  │          Import Resolution Engine         │  │
│  │  (resolve DLL imports → Rust impls)       │  │
│  └──────────────────┬────────────────────────┘  │
│                     │                           │
│  ┌──────────────────▼────────────────────────┐  │
│  │       NT Syscall Translation Layer        │  │
│  │  (NtCreateFile → open(), etc.)            │  │
│  └──────────────────┬────────────────────────┘  │
│                     │                           │
│  ┌──────────────────▼────────────────────────┐  │
│  │         Reimplemented DLLs (Rust)         │  │
│  │  ntdll · kernel32 · msvcrt · advapi32 ·   │  │
│  │  user32 · gdi32 · ws2_32 · ole32 · ...    │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────┐
│          rine-config (Tauri app)                 │
│  Right-click context menu → per-app settings     │
│  - Windows version spoofing                      │
│  - DLL override preferences                      │
│  - rine-compat profile application               │
└──────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────┐
│          rine-compat (separate git repo)         │
│  - TOML/YAML compatibility profiles per app      │
│  - Community ratings (works/broken/partial)      │
│  - CLI to pull/update/query                      │
└──────────────────────────────────────────────────┘
```

---

## Phase 1 — PE Loading & "Hello World" (Foundation)

**Goal**: Load and execute a minimal Windows console .exe that calls `WriteConsoleA` or `printf`.

### Steps

1. **PE Parsing via `goblin`** (`src/pe/`)
   - Use goblin's `PE` type directly — no wrapper layer; rine-specific types live downstream in the loader
   - `parse_pe()` function: mmap file via `memmap2`, call `goblin::pe::PE::parse()` (zero-copy), validate PE64
   - Validate: must be PE32+ (64-bit), not a DLL, has a valid entry point
   - Access headers: `pe.header` (COFF + optional), `pe.sections`, `pe.entry`/`pe.image_base`
   - Access imports: `pe.imports` (parsed `Import` structs with DLL name, function name/ordinal)
   - Access relocations: section-based relocation entries via `goblin::pe::relocation`
   - Access TLS: `pe.tls_data` for TLS callbacks and index
   - Error types for parse failures, unsupported formats, and validation errors

2. **Memory Loader** (`src/loader/`)
   - Read file via `memmap2`, pass `&[u8]` to `goblin::pe::PE::parse()`
   - Allocate virtual address space via `mmap` at `pe.image_base` (or relocate)
   - Copy each section from goblin's `pe.sections` with correct permissions (RX for .text, RW for .data, R for .rdata) using section `characteristics` flags
   - Apply base relocations from goblin's relocation data if image couldn't load at preferred base
   - Process TLS directory from `pe.tls_data` if present

3. **Import Resolver** (`src/imports/`)
   - Iterate `pe.imports` — each entry has `.dll` (name), `.name` (function name), `.ordinal` (if ordinal-based)
   - Map each DLL name to a rine-dlls Rust module, look up function pointer by name/ordinal
   - Write function pointers into IAT (Import Address Table) in mapped memory
   - Handle delay-load imports via goblin's delay-load parsing

4. **Minimal ntdll + kernel32 stubs** (`src/dlls/ntdll/`, `src/dlls/kernel32/`)
   - ntdll: `NtWriteFile`, `NtTerminateProcess`, `RtlInitUnicodeString`
   - kernel32: `GetStdHandle`, `WriteConsoleA`, `WriteConsoleW`, `WriteFile`, `ExitProcess`, `GetCommandLineA/W`, `GetModuleHandleA/W`
   - Map Windows HANDLE for stdout/stderr/stdin → Linux fd 0/1/2

5. **Minimal msvcrt stubs** (`src/dlls/msvcrt/`)
   - `printf`, `puts`, `exit`, `_cexit`, `__getmainargs`
   - C runtime initialization sequence (`_initterm`, `_initterm_e`)

6. **Entry point execution** (`src/loader/`)
   - Set up initial stack frame matching Windows x64 calling convention (rcx, rdx, r8, r9, shadow space)
   - Transfer control to PE entry point (AddressOfEntryPoint)
   - Handle process exit cleanly

### Verification
- Compile a trivial `hello.c` with `x86_64-w64-mingw32-gcc` → produce `hello.exe`
- `rine hello.exe` prints "Hello, world!" and exits cleanly
- Test with both PE32 (32-bit) and PE32+ (64-bit) — start with 64-bit only

---

## Phase 2 — Core OS Subsystem (Filesystem, Processes, Threads)

**Goal**: Support executables that do file I/O, spawn processes, and use threads.

### Steps

7. **Path translation layer** (`src/subsys/filesystem.rs`)
   - Map Windows paths (`C:\Users\...`) to Linux paths (`~/.rine/drives/c/Users/...`)
   - Handle drive letter mapping (configurable per-app)
   - Translate path separators, handle case-insensitivity (configurable)
   - Support UNC paths, `\\?\` long path prefix

8. **File I/O in ntdll/kernel32** (`src/dlls/ntdll/`, `src/dlls/kernel32/`)
   - `NtCreateFile`, `NtReadFile`, `NtWriteFile`, `NtClose`, `NtQueryInformationFile`
   - kernel32: `CreateFileA/W`, `ReadFile`, `WriteFile`, `CloseHandle`, `GetFileSize`, `SetFilePointer`, `FindFirstFileA/W`, `FindNextFileA/W`
   - Windows HANDLE table → Linux fd mapping with reference counting

9. **Threading** (`src/subsys/threading.rs`)
   - `CreateThread` → `pthread_create` with Windows-compatible thread entry signature
   - TLS slot allocation (`TlsAlloc`, `TlsSetValue`, `TlsGetValue`)
   - Critical sections → `pthread_mutex`
   - `WaitForSingleObject`/`WaitForMultipleObjects` → futex or eventfd-based

10. **Process management** (`src/subsys/process.rs`)
    - `CreateProcessA/W` → `fork`+`exec` (re-invoke rine on child .exe)
    - Environment block translation (Unicode, null-separated)
    - `GetCurrentProcessId`, `GetCurrentThreadId`

11. **Synchronization primitives** (`src/subsys/sync.rs`)
    - Events (`CreateEvent`, `SetEvent`, `ResetEvent`) → eventfd
    - Mutexes (`CreateMutex`) → futex-based
    - Semaphores → Linux semaphores

12. **Heap management** (`src/subsys/heap.rs`)
    - `HeapCreate`, `HeapAlloc`, `HeapFree`, `HeapReAlloc`
    - Back with Rust allocator or direct mmap for large heaps
    - `VirtualAlloc`/`VirtualFree` → mmap/munmap with Windows permission flags

### Verification
- Run a .exe that reads a file, writes output, spawns a thread
- Test path translation correctness with unit tests
- Stress-test threading with a multi-threaded Windows binary

---

## Phase 3 — Registry, Environment & Config System

**Goal**: Emulate Windows registry, environment variables, and build the per-app config system.

### Steps

13. **Registry emulation** (`src/subsys/registry.rs`)
    - Store as a hierarchical key-value filesystem under `~/.rine/registry/`
    - TOML or binary hive files per root key (HKLM, HKCU, etc.)
    - `RegOpenKeyExA/W`, `RegQueryValueExA/W`, `RegSetValueExA/W`, `RegCloseKey`
    - Support common registry queries apps make (OS version, system paths, installed software)

14. **Environment subsystem** (`src/subsys/environment.rs`)
    - Translate Windows env vars (`%USERPROFILE%`, `%TEMP%`, `%SYSTEMROOT%`) to mapped paths
    - `GetEnvironmentVariableA/W`, `SetEnvironmentVariableA/W`, `ExpandEnvironmentStringsA/W`

15. **Per-app configuration** (`src/config/`)
    - Config stored at `~/.rine/apps/<app-hash>/config.toml`
    - Settings: Windows version (XP/7/10/11), arch override, DLL search order, drive mappings, env overrides
    - Config schema defined in Rust structs with serde
    - CLI: `rine --config <exe>` opens config editor

16. **Windows version spoofing** (`src/subsys/version.rs`)
    - `GetVersionExA/W`, `RtlGetVersion` return values matching configured Windows version
    - Version manifests in PE resources → parse and respect

### Verification
- Run an app that reads registry keys (e.g., queries OS version)
- Verify config TOML round-trips correctly
- Test version spoofing with apps that check Windows version

---

## Phase 4 — Desktop Integration

**Goal**: Double-click .exe files on Linux desktop to run them via rine.

### Steps

17. **binfmt_misc registration** (`src/integration/binfmt.rs`)
    - Register MZ magic bytes with Linux kernel's binfmt_misc
    - `echo ':DOSWin:M::MZ::/usr/bin/rine:' > /proc/sys/fs/binfmt_misc/register`
    - Installer/setup command: `rine --install-binfmt` (requires root)
    - This enables: `./program.exe` and `chmod +x program.exe && double-click`

18. **Freedesktop .desktop file** (`src/integration/desktop.rs`)
    - Install MIME type for `application/x-dosexec` and `application/x-ms-dos-executable`
    - .desktop entry: `Exec=rine %f`
    - Icon integration for .exe files (extract icon from PE resources if available)

19. **Right-click context menu** (`src/integration/context_menu.rs`)
    - Freedesktop "Open With" registration for "Configure with rine"
    - Nautilus/Dolphin script integration for "rine Settings" context action
    - Launches rine-config Tauri app with the target .exe path

20. **Tauri config editor** (`crates/rine-config/` — workspace crate)
    - Workspace layout: add `rine-config` to Cargo workspace
    - Frontend (HTML/CSS/JS):
      - App profile view: Windows version, DLL settings, drive mappings
      - rine-compat browser: search/apply community profiles
      - Log viewer: see rine execution logs for debugging
      - Compatibility test runner: quick-launch and report status
    - Backend (Rust/Tauri commands):
      - Read/write per-app config.toml
      - Query rine-compat database
      - Launch exe with rine and capture output

### Verification
- On GNOME/KDE: double-click .exe → rine executes it
- Right-click .exe → "Configure with rine" opens Tauri editor
- binfmt_misc registration persists across reboots (systemd unit)

---

## Phase 5 — GUI Subsystem (Future)

**Goal**: Support Win32 GUI applications.

### Steps

21. **Win32 window management** (`src/dlls/user32/`)
    - `CreateWindowExA/W`, `ShowWindow`, `UpdateWindow`, `DestroyWindow`
    - Message loop: `GetMessage`, `TranslateMessage`, `DispatchMessage`
    - Map to X11/Wayland via a backend abstraction (consider `winit` or raw xcb/wayland-client)
    - Window class registration, WndProc callback dispatch

22. **GDI rendering** (`src/dlls/gdi32/`)
    - Device contexts, bitmaps, brushes, pens
    - `BitBlt`, `TextOut`, `CreateCompatibleDC`
    - Backed by software rendering → surface blit to X11/Wayland

23. **Common dialogs + controls** (`src/dlls/comdlg32/`, `src/dlls/comctl32/`)
    - File open/save dialogs → map to native Linux dialogs (via XDG portals)
    - Common controls (buttons, listboxes, treeviews) → custom rendering or GTK mapping

### Verification
- Run notepad.exe (or a minimal Win32 GUI app compiled with MinGW)
- Window appears, responds to mouse/keyboard, repaints correctly

---

## Phase 6 — Advanced (Future)

24. **Networking** (`src/dlls/ws2_32/`) — Winsock → POSIX sockets
25. **COM/OLE** (`src/dlls/ole32/`) — COM interface vtable translation
26. **DirectX** (`src/dlls/d3d*/`) — translate to Vulkan (reference: DXVK approach)
27. **Audio** (`src/dlls/winmm/`, `dsound`) — ALSA/PulseAudio/PipeWire backend
28. **.NET/CLR** — Possible integration with Mono or CoreCLR

---

## rine-compat — Compatibility Database Design

**Separate git repository**: `rine-compat`

### Repository Structure

```
rine-compat/
├── schema/
│   └── profile.schema.json        # JSON Schema for profile validation
├── profiles/
│   ├── by-name/
│   │   ├── notepad-plus-plus/
│   │   │   └── profile.toml
│   │   ├── 7zip/
│   │   │   └── profile.toml
│   │   └── ...
│   └── by-hash/
│       └── <sha256-of-exe>.toml   # Auto-generated lookup by binary hash
├── ratings/
│   ├── summary.json               # Aggregated ratings (generated)
│   └── reports/
│       └── <app-name>/
│           └── <reporter>.toml    # Individual compatibility reports
├── categories/
│   └── tags.toml                  # Category/tag definitions
├── tools/
│   ├── validate.rs                # Profile validation tool
│   └── generate-index.rs          # Index generator for fast lookups
├── README.md
└── CONTRIBUTING.md
```

### Profile Format (TOML)

```toml
[app]
name = "7-Zip"
version = "23.01"          # Windows app version
homepage = "https://7-zip.org"
exe_name = "7z.exe"
exe_sha256 = "abc123..."   # Optional, for exact match
tags = ["utility", "archiver", "cli"]

[rine]
min_version = "0.3.0"      # Minimum rine version required

[config]
windows_version = "win10"  # win7 | win10 | win11 | winxp
arch = "x86_64"            # x86 | x86_64

[config.dlls]
# Override DLL behavior
# "builtin" = rine's Rust impl, "disabled" = don't load
kernel32 = "builtin"

[config.drives]
# Custom drive mappings for this app
# "d" = "/media/data"

[config.env]
# Environment variable overrides
# WINEDEBUG equivalent for rine
# RINE_LOG = "trace"

[config.registry]
# Registry key presets this app needs
# [[config.registry.keys]]
# path = "HKCU\\Software\\7-Zip"
# values = { "Path" = "C:\\Program Files\\7-Zip" }

[compatibility]
rating = "gold"            # platinum | gold | silver | bronze | broken
tested_rine_version = "0.5.0"
tested_date = "2026-01-15"
notes = "CLI mode works perfectly. GUI mode not yet supported by rine."
```

### Rating System

| Rating   | Meaning |
|----------|---------|
| Platinum | Works perfectly out of the box, no config needed |
| Gold     | Works with minor config (profile provides it) |
| Silver   | Works with workarounds, some features broken |
| Bronze   | Starts but major functionality broken |
| Broken   | Does not run |

### Community Reports Format

```toml
[report]
reporter = "github:username"
date = "2026-03-20"
rine_version = "0.5.0"
distro = "Ubuntu 24.04"
rating = "gold"
notes = "Works great for CLI compression. Tested 7z a, 7z x, 7z l."

[report.system]
kernel = "6.8.0"
cpu = "x86_64"
gpu = "AMD RX 7900"  # Relevant for GUI/DirectX apps
```

### rine-compat CLI Integration

Built into rine as a subcommand:
- `rine compat update` — git pull latest rine-compat data
- `rine compat search <query>` — search profiles by name/tag
- `rine compat apply <profile> [exe]` — apply a profile to an exe's config
- `rine compat report <exe>` — submit a compatibility report (opens editor or Tauri)
- `rine compat info <exe>` — show compatibility info for an exe (by name or hash)

The Tauri config editor integrates this: browse profiles, one-click apply, submit reports.

---

## Project Structure (Cargo Workspace)

```
rine/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── rine/               # Main binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── cli.rs      # Argument parsing (clap)
│   │       ├── pe/
│   │       │   ├── mod.rs
│   │       │   └── parser.rs       # PE parsing, validation, error types
│   │       ├── loader/
│   │       │   ├── mod.rs
│   │       │   ├── memory.rs       # mmap, virtual address space
│   │       │   ├── resolver.rs     # Import resolution → DLL shims
│   │       │   └── entry.rs        # Entry point setup & execution
│   │       ├── subsys/
│   │       │   ├── mod.rs
│   │       │   ├── filesystem.rs   # Path translation, drive mapping
│   │       │   ├── threading.rs    # Thread creation, TLS
│   │       │   ├── process.rs      # Process management
│   │       │   ├── sync.rs         # Events, mutexes, semaphores
│   │       │   ├── heap.rs         # HeapAlloc, VirtualAlloc
│   │       │   ├── registry.rs     # Registry emulation
│   │       │   ├── environment.rs  # Env var translation
│   │       │   └── version.rs      # Windows version spoofing
│   │       ├── config/
│   │       │   ├── mod.rs
│   │       │   ├── schema.rs       # Config struct definitions
│   │       │   └── manager.rs      # Load/save per-app config
│   │       ├── compat/
│   │       │   ├── mod.rs
│   │       │   ├── client.rs       # rine-compat repo operations
│   │       │   └── search.rs       # Profile search/matching
│   │       └── integration/
│   │           ├── mod.rs
│   │           ├── binfmt.rs       # binfmt_misc registration
│   │           ├── desktop.rs      # .desktop file generation
│   │           └── context_menu.rs # Right-click integration
│   │
│   ├── rine-dlls/           # Reimplemented Windows DLLs
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ntdll/
│   │       │   ├── mod.rs
│   │       │   ├── file.rs        # NtCreateFile, NtReadFile, etc.
│   │       │   ├── process.rs     # NtTerminateProcess, etc.
│   │       │   ├── memory.rs      # NtAllocateVirtualMemory, etc.
│   │       │   └── rtl.rs         # Rtl* utility functions
│   │       ├── kernel32/
│   │       │   ├── mod.rs
│   │       │   ├── file.rs        # CreateFile, ReadFile, etc.
│   │       │   ├── console.rs     # WriteConsole, GetStdHandle, etc.
│   │       │   ├── process.rs     # CreateProcess, ExitProcess, etc.
│   │       │   ├── thread.rs      # CreateThread, TLS, etc.
│   │       │   ├── memory.rs      # HeapAlloc, VirtualAlloc, etc.
│   │       │   └── sync.rs        # CreateEvent, WaitFor*, etc.
│   │       ├── msvcrt/
│   │       │   ├── mod.rs
│   │       │   ├── stdio.rs       # printf, puts, fopen, etc.
│   │       │   └── stdlib.rs      # malloc, free, exit, etc.
│   │       ├── advapi32/
│   │       │   └── mod.rs         # Registry, security functions
│   │       ├── user32/
│   │       │   └── mod.rs         # Window mgmt (Phase 5)
│   │       ├── gdi32/
│   │       │   └── mod.rs         # GDI rendering (Phase 5)
│   │       └── ws2_32/
│   │           └── mod.rs         # Winsock (Phase 6)
│   │
│   └── rine-types/           # Shared Windows type definitions
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── handles.rs    # HANDLE, HMODULE, etc.
│           ├── strings.rs    # LPSTR, LPWSTR, Unicode conversion
│           ├── errors.rs     # Win32 error codes, NTSTATUS
│           └── structs.rs    # SECURITY_ATTRIBUTES, OVERLAPPED, etc.
│
│   └── rine-config/          # Tauri config editor app
│       ├── Cargo.toml
│       ├── tauri.conf.json
│       ├── src-tauri/
│       │   └── src/
│       │       └── main.rs   # Tauri commands
│       └── src/              # Frontend (HTML/CSS/JS)
│           ├── index.html
│           ├── style.css
│           └── app.js
│
├── docs/
│   └── Overview.md
└── README.md
```

---

## Key Dependencies (Planned)

| Crate | Purpose |
|-------|---------|
| `goblin` | PE/COFF parsing — headers, sections, imports, relocations, TLS, resources (zero-copy, PE32+PE32+) |
| `nix` | Linux syscall wrappers (mmap, ioctl, etc.) |
| `libc` | Low-level C FFI for syscalls |
| `clap` | CLI argument parsing |
| `serde` + `toml` | Config serialization |
| `tauri` | Config editor GUI |
| `tracing` | Structured logging |
| `memmap2` | Memory-mapped file I/O (feed mmap'd bytes to goblin) |
| `bitflags` | Windows flag constants |

---

## Decisions

- **Custom PE loader, not WINE wrapper** — full independence, Rust-native
- **`goblin` crate for PE parsing** — zero-copy, handles PE32/PE32+, imports, relocations, TLS, resources; rine focuses on loading/execution, not reinventing parsing
- **Userspace syscall translation** — no kernel modules, portable
- **Reimplemented DLLs in Rust** — no dependency on Windows or WINE DLLs
- **64-bit first** (PE32+), 32-bit (PE32) support later
- **Freedesktop standard** for desktop integration (GNOME + KDE)
- **Tauri** for the config editor (HTML/CSS/JS frontend, Rust backend)
- **rine-compat** as separate git repo for compatibility database
- **TOML** for all config and profile formats
- **No WINE interop** — clean independent ecosystem

## Scope Boundaries

**Included**: PE loading, core Win32 API subset (console, files, threads, registry), desktop integration, config system, compat database design
**Excluded (for now)**: 32-bit PE support, GUI subsystem (user32/gdi32), DirectX, COM, .NET, audio, networking — these are future phases

---

## Further Considerations

1. **x86 (32-bit) support**: Running 32-bit PEs on 64-bit Linux may require multilib or a translation layer. Recommend deferring and starting 64-bit only.
2. **Structured exception handling (SEH)**: Windows x64 uses table-based SEH. This needs integration with the loader for try/catch in C++ apps. Can stub initially but must be addressed for real apps.
3. **Unicode handling**: Windows uses UTF-16 internally. Every string API has A/W variants. The type system in rine-types should make UTF-8↔UTF-16 conversion ergonomic and zero-copy where possible.
