# rine

**A Windows PE executable loader for Linux, written from scratch in Rust.**

Project website: https://scriptandcompile.github.io/rine/

rine translates Windows NT syscalls to Linux syscalls in userspace and provides Rust reimplementations of core Windows DLLs, allowing you to run Windows `.exe` files directly on Linux — no virtual machine, no CPU emulator, no Wine.

> **Status:** v0.1.0 — early development. Console applications with basic I/O, threading, and synchronization work. GUI and networking support are planned.

## How It Works

rine loads x86_64 PE binaries directly into memory, resolves their imports against reimplemented DLLs, and translates Windows API calls to Linux equivalents at runtime:

```
./app.exe
    │
    ▼
┌──────────────────────┐
│  PE Parser (goblin)  │  Parse COFF headers, validate PE64
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│  Memory Loader       │  mmap sections, apply relocations, TLS
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│  Import Resolver     │  Match imports → DLL plugin registry
└──────────┬───────────┘
           ▼
┌──────────────────────────────────────────┐
│  Reimplemented DLLs                      │
│  ntdll · kernel32 · msvcrt · advapi32    │
│  gdi32 · user32 · ws2_32                 │
└──────────┬───────────────────────────────┘
           ▼
┌──────────────────────┐
│  Linux syscalls      │  via nix / libc
└──────────────────────┘
```

## Features

- **PE Loading** — Parses and loads x64 PE executables using `goblin`, maps sections via `mmap`, applies base relocations, handles TLS
- **DLL Plugin System** — 7 core Windows DLLs reimplemented in Rust as separate crates, each implementing the `DllPlugin` trait
- **Per-App Configuration** — TOML configs with Windows version spoofing, drive mapping, DLL overrides, and environment injection
- **Desktop Integration** — `binfmt_misc` registration, `.desktop` MIME types, and file manager context menus
- **Developer Dashboard** — Real-time Tauri 2 GUI (`--dev`) showing imports, handles, threads, and events
- **Config Editor** — Tauri 2 GUI for editing per-app settings

## Supported DLLs

| DLL | Status | Coverage |
|-----|--------|----------|
| `kernel32.dll` | Active | Process, thread, file, console, memory, synchronization APIs |
| `ntdll.dll` | Active | NtCreateFile, NtReadFile, NtWriteFile, NtClose, NtTerminateProcess |
| `msvcrt.dll` | Active | printf, puts, malloc, free, exit, string ops, CRT init |
| `advapi32.dll` | Stub | Registry access, security functions |
| `comdlg32.dll` | Partial | GetOpenFileNameA/W, GetSaveFileNameA/W, CommDlgExtendedError |
| `gdi32.dll` | Partial | Graphics device interface |
| `user32.dll` | Partial | Window management, message dispatching |
| `ws2_32.dll` | Stub | Winsock networking |

## Quick Start

### Requirements

- **Rust** 1.85+ (edition 2024)
- **Linux x86_64** (WSL2 supported)
- **MinGW** cross-compiler (optional, for building test fixtures)

### Build

```bash
# Because cargo assumes only a single target platform per workspace per build process
# we are required to specificy the two target platforms explicitly for the different executables.

# build the 64bit components of rine without dev features
cargo build --release --target x86_64-unknown-linux-gnu -p rine --no-default-features -p rine-config

# build the 32bit components of rine without dev features. Required if you want to run 32bit Windows programs.
cargo build --release -p rine32 --target i686-unknown-linux-gnu --no-default-features
```

This produces two binaries:

| Binary | Description |
|--------|-------------|
| `target/release/rine` | Main loader |
| `target/release/rine-config` | Config editor GUI (Tauri 2) |

| Binary | Description |
|--------|-------------|
| `target/release/rine-dev` | Developer dashboard (Tauri 2), optionally created if default features are enabled.|


### Run a Windows executable

```bash
rine ./hello.exe
rine ./myapp.exe arg1 arg2 --flag
```

### Developer dashboard

Launch the real-time developer dashboard alongside execution:

```bash
rine --dev ./myapp.exe
```

This opens a Tauri GUI with the following tabs:

| Tab | Shows |
|-----|-------|
| **Overview** | PE info, config summary, import statistics |
| **Imports** | Resolved vs. stubbed DLL functions with call counts |
| **Handles** | Open/closed handle tracking (files, threads, events, mutexes) |
| **Threads** | Thread lifecycle, entry points, and state |
| **Events** | Chronological log of all runtime events |
| **Output** | Captured stdout/stderr from the executable |

### Per-app configuration

```bash
# Show or create a config file
rine --config ./myapp.exe

# Or use the GUI editor
rine-config ./myapp.exe
```

Configuration is stored at `~/.rine/apps/<app-hash>/config.toml`:

```toml
[windows]
version = "win10"  # winxp, win7, win10, win11

[filesystem]
case_insensitive = true

[filesystem.drives]
c = "/mnt/windows/drive_c"
d = "/mnt/media"

[dll]
builtin = ["kernel32.dll", "ntdll.dll", "msvcrt.dll"]
force_stub = []

[environment]
RINE_LOG = "debug"
```

### Desktop integration

```bash
# Register with binfmt_misc (run .exe files directly, requires root)
sudo rine --install-binfmt

# After registration:
chmod +x app.exe
./app.exe  # runs through rine automatically

# File manager integration
rine --install-desktop         # .desktop MIME type handler
rine --install-context-menu    # right-click context menu

# Check status
rine --binfmt_status
rine --desktop_status
rine --context_menu_status
```

## Architecture

```
┌─────────────────────────────────────────────┐
│                   rine CLI                  │
├──────────┬──────────┬───────────┬───────────┤
│ PE Parser│  Config  │  Loader   │  Commands │
│ (goblin) │ Manager  │ (mmap,    │ (run,     │
│          │          │  relocs,  │  binfmt,  │
│          │          │  TLS)     │  desktop) │
├──────────┴──────────┴───────────┴───────────┤
│          Import Resolution Engine           │
├─────────────────────────────────────────────┤
│              DLL Plugin Registry            │
├───────┬────────┬───────┬────────┬───────────┤
│ntdll  │kernel32│msvcrt │advapi32│user32 ... │
├───────┴────────┴───────┴────────┴───────────┤
│        OS Subsystem Translation Layer       │
│  (filesystem, threading, registry, sync,    │
│   memory, environment, process, heap)       │
├─────────────────────────────────────────────┤
│               Linux (syscalls)              │
└─────────────────────────────────────────────┘
```

## Project Structure

```
crates/
├── rine/                  # Main CLI binary
│   └── src/
│       ├── commands/      # run, config, binfmt, desktop, context_menu
│       ├── loader/        # PE memory mapping and entry point
│       ├── pe/            # PE file parsing
│       ├── subsys/        # OS subsystem emulation
│       ├── config/        # Per-app configuration
│       ├── compat/        # Compatibility database
│       └── integration/   # Desktop integration
├── rine-types/            # Shared types (handles, strings, errors, threading, registry)
├── rine-dlls/             # DllPlugin trait and registry
├── rine-channel/          # IPC protocol (Unix domain socket) for dev dashboard
├── rine-dev/              # Developer dashboard (Tauri 2)
├── rine-config/           # Config editor GUI (Tauri 2)
├── rine-frontend-common/  # Shared frontend assets
└── platform/win64-dll/    # DLL implementations
    ├── rine64-kernel32/
    ├── rine64-ntdll/
    ├── rine64-msvcrt/
    ├── rine64-advapi32/
    ├── rine64-gdi32/
    ├── rine64-user32/
    └── rine64-ws2_32/
```

## Testing

```bash
# Install MinGW cross-compilers (Debian/Ubuntu)
sudo apt install gcc-mingw-w64-x86-64 gcc-mingw-w64-i686

# Build rine so it is ready to run
./scripts/build-rine.sh

# Build rine and run all unit tests
./scripts/build-rine-unit-tests.sh

# Build everything needed for integration tests
./scripts/build-integration-prereqs.sh

# Run the test suite
cargo test
```

Test fixtures are C programs under `tests/fixtures/src/` (organized by domain, e.g. `core/`, `system/`, `ui/`) compiled to PE executables in `tests/fixtures/bin/x64/` and `tests/fixtures/bin/x86/`.
By default, integration tests use `x64` fixtures; set `RINE_FIXTURE_ARCH=x86` to target x86 fixture binaries.

## Debian Packaging

Build a Debian package that includes all runtime-facing components:

- `rine` (main runtime)
- `rine32` (x86 helper runtime)
- `rine-dev` (developer dashboard)
- `rine-config` (configuration editor)

```bash
./scripts/build-rine-deb.sh
```

Output artifact:

- `target/debian/rine_<version>_<arch>.deb`

Install locally:

```bash
sudo dpkg -i target/debian/rine_<version>_<arch>.deb
```

Package install behavior:

- Registers `binfmt_misc` automatically (best-effort, non-fatal if unavailable).
- Installs system-wide desktop + MIME association for `.exe` files on Freedesktop environments (including GNOME).
- Installs per-user file-manager right-click context menu actions automatically for local desktop users (UID >= 1000).

## Roadmap

- [x] PE parsing and memory loading
- [x] Import resolution and DLL plugin system
- [x] Basic kernel32, ntdll, msvcrt (console I/O, threading, sync)
- [x] Per-app configuration system
- [x] Desktop integration (binfmt_misc, .desktop, context menus)
- [x] Developer dashboard (rine-dev)
- [x] File I/O subsystem with path translation and drive mapping
- [x] Full threading and TLS support
- [x] Registry emulation
- [ ] GUI subsystem (user32/gdi32 → X11/Wayland)
- [ ] Networking (ws2_32 → POSIX sockets)
- [ ] COM/OLE, DirectX, audio

## License

[MIT](https://opensource.org/licenses/MIT)
