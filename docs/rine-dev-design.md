# rine-dev Design Document

## Summary

`rine-dev` is a Tauri 2 application that provides a real-time developer dashboard for debugging and inspecting Windows PE executables running under rine. It is launched via `rine --dev <exe>` and displays comprehensive runtime telemetry in a GUI.

---

## Invocation

```
rine --dev ./myapp.exe [-- args...]
```

This launches the PE under rine as normal, but also starts the `rine-dev` Tauri GUI connected to the running process via a local communication channel.

### CLI Change (cli.rs)

Add a `--dev` flag to `Cli`:

```rust
/// Launch the developer dashboard alongside the PE
#[arg(long)]
pub dev: bool,
```

When `--dev` is set, `main.rs` spawns `rine-dev` as a child process, passing it a channel identifier (Unix domain socket path or shared memory name), then proceeds with normal PE execution. The `run()` pipeline is augmented to emit telemetry events to the channel.

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                     rine process                     │
│                                                      │
│  PE Load → Import Resolve → TEB Init → Entry Point  │
│       │           │            │            │        │
│       └───────────┴────────────┴────────────┘        │
│                       │                              │
│              DevChannel (sender)                     │
│                       │ Unix domain socket           │
└───────────────────────┼─────────────────────────────-┘
                        │
┌───────────────────────┼──────────────────────────────┐
│              DevChannel (receiver)                    │
│                       │                              │
│              rine-dev Tauri app                       │
│  ┌──────────┬─────────┬──────────┬──────────────┐    │
│  │ Imports  │ Memory  │ Handles  │   Threads    │    │
│  │  Panel   │  Panel  │  Panel   │    Panel     │    │
│  ├──────────┴─────────┴──────────┴──────────────┤    │
│  │              Event Log / Timeline             │    │
│  └───────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────┘
```

### Communication: DevChannel

A Unix domain socket (`/tmp/rine-dev-<pid>.sock`) created by rine before PE execution begins. The rine-dev app connects as a client.

**Protocol**: Length-prefixed JSON messages (4-byte little-endian length + UTF-8 JSON payload). Simple, debuggable, no external dependencies.

**Why not shared memory?** JSON over UDS is simpler to implement, debug, and extend. Throughput is not a bottleneck — telemetry events are human-scale (hundreds/sec at most, not millions). We can revisit if profiling shows otherwise.

---

## Crate Structure

### New crates

| Crate | Path | Purpose |
|-------|------|---------|
| `rine-dev` | `crates/rine-dev/` | Tauri 2 app binary + lib (dashboard GUI) |
| `rine-channel` | `crates/rine-channel/` | Shared types + sender/receiver for DevChannel |

### Modified crates

| Crate | Change |
|-------|--------|
| `rine` | Add `--dev` flag, conditionally create DevChannel sender, emit events at instrumentation points |
| `rine-dlls` | Emit function-call events when dev mode is active |

### rine-channel

Shared between rine (sender) and rine-dev (receiver). No Tauri dependency.

```rust
// crates/rine-channel/src/lib.rs

/// Events sent from rine → rine-dev
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DevEvent {
    // --- Lifecycle ---
    PeLoaded {
        exe_path: String,
        image_base: u64,
        image_size: u64,
        entry_rva: u64,
        relocation_delta: i64,
        sections: Vec<SectionInfo>,
    },
    ConfigLoaded {
        config_path: String,
        windows_version: String,
        environment_overrides: Vec<(String, String)>,
    },

    // --- Import Resolution ---
    ImportsResolved {
        summaries: Vec<DllSummary>,
        total_resolved: usize,
        total_stubbed: usize,
    },

    // --- Runtime Function Calls ---
    FuncCalled {
        dll: String,
        name: String,
        /// Whether this is a real implementation or a stub
        is_stub: bool,
        timestamp_us: u64,
    },

    // --- Handle Table ---
    HandleCreated {
        handle: i64,
        kind: String, // "File", "Thread", "Event", "Mutex", "Heap", "RegistryKey", etc.
        detail: String, // e.g. file path, thread id
    },
    HandleClosed {
        handle: i64,
    },

    // --- Memory ---
    MemoryAllocated {
        address: u64,
        size: u64,
        source: String, // "VirtualAlloc", "HeapAlloc", etc.
    },
    MemoryFreed {
        address: u64,
    },

    // --- Threads ---
    ThreadCreated {
        handle: i64,
        thread_id: u32,
        entry_point: u64,
    },
    ThreadExited {
        thread_id: u32,
        exit_code: u32,
    },

    // --- TLS ---
    TlsAllocated { index: u32 },
    TlsFreed { index: u32 },

    // --- Registry ---
    RegistryAccess {
        operation: String, // "Open", "Create", "QueryValue", "SetValue", "Close"
        path: String,
        value_name: Option<String>,
    },

    // --- File System ---
    FileOperation {
        operation: String, // "CreateFile", "ReadFile", "WriteFile", "FindFirstFile", etc.
        path: String,
        handle: Option<i64>,
    },

    // --- Environment ---
    EnvVarAccess {
        operation: String, // "Get", "Set", "Expand"
        name: String,
        value: Option<String>,
    },

    // --- Process Exit ---
    ProcessExited { exit_code: i32 },
    
    // --- Errors ---
    StubHit {
        dll: String,
        name: String,
        /// Backtrace or call context if available
        context: String,
    },
}

#[derive(Serialize, Deserialize)]
pub struct SectionInfo {
    pub name: String,
    pub virtual_address: u64,
    pub virtual_size: u64,
    pub characteristics: u32,
}

#[derive(Serialize, Deserialize)]
pub struct DllSummary {
    pub dll_name: String,
    pub resolved: usize,
    pub stubbed: usize,
    pub stubbed_names: Vec<String>,
    pub resolved_names: Vec<String>,
}
```

**Sender** (used by rine):

```rust
pub struct DevSender { /* UDS stream + optional buffering */ }

impl DevSender {
    pub fn connect(socket_path: &Path) -> io::Result<Self>;
    pub fn send(&self, event: &DevEvent) -> io::Result<()>;
}
```

**Receiver** (used by rine-dev):

```rust
pub struct DevReceiver { /* UDS listener */ }

impl DevReceiver {
    pub fn bind(socket_path: &Path) -> io::Result<Self>;
    pub fn recv(&self) -> io::Result<DevEvent>;
    pub fn into_stream(self) -> impl Iterator<Item = io::Result<DevEvent>>;
}
```

### Conditional compilation in rine

Dev channel support is gated behind a cargo feature so it has **zero cost** when not used:

```toml
# crates/rine/Cargo.toml
[features]
default = []
dev = ["dep:rine-channel"]
```

The `--dev` CLI flag is always present (cheap), but when the `dev` feature is off, it prints an error saying rine was built without dev support.

In instrumented code:

```rust
// Macro that compiles to nothing when dev feature is off
macro_rules! dev_emit {
    ($channel:expr, $event:expr) => {
        #[cfg(feature = "dev")]
        if let Some(ch) = $channel {
            let _ = ch.send(&$event);
        }
    };
}
```

---

## rine-dev Tauri Application

### Window Layout

Single window, tabbed layout. Default size: 1200x800, resizable.

```
┌──────────────────────────────────────────────────────────────────────┐
│  rine-dev ─ myapp.exe                                       [─][□][×]│
├────────┬──────────┬─────────┬──────────┬─────────┬──────────┬────────┤
│Overview│ Imports  │ Files   │ Threads  │ Mutexes │ Windows  │ Events │
├────────┴──────────┴─────────┴──────────┴─────────┴──────────┴────────┤
│                                                                      │
│  (Tab content area — see panels below)                               │
│                                                                      │
│                                                                      │
│                                                                      │
│                                                                      │
├──────────────────────────────────────────────────────────────────────┤
│ Status: Running │ Handles: 14 │ Threads: 3 │ Stubs hit: 2  │         │
└──────────────────────────────────────────────────────────────────────┘
```

### Panels

#### 1. Overview

Summary card layout:

- **PE Info**: exe path, image base, image size, entry RVA, relocation delta, section count
- **Config**: windows version, config file path, environment overrides
- **Import Summary**: total resolved / total stubbed, pie chart or bar
- **Runtime Counters**: handles open, threads running, memory allocated, TLS slots used

#### 2. Imports

Table with columns:

| DLL | Function | Status | Call Count |
|-----|----------|--------|------------|
| kernel32.dll | CreateFileA | Implemented | 12 |
| kernel32.dll | GetSystemInfo | Implemented | 1 |
| msvcrt.dll | _wcsicmp | **Stub** | 0 |
| gdi32.dll | CreateFontW | **Stub** | 3 |

- Filterable by DLL, status (implemented/stub), and whether it's been called
- Sort by call count to find hot functions
- Color-coded: green for implemented, red for stubs, orange for stubs-that-were-called (problems)

#### 3. Files

Live table of open file handles:

| Handle | Path | Opened At | Status |
|--------|------|-----------|--------|
| 0x1000 | C:\Users\user\test.txt | 0.012s | Open |
| 0x1004 | C:\Windows\System32\kernel32.dll | 0.003s | Closed |

- Shows only File-type handles (threads shown in Threads tab, other handle types in future tabs)
- Grayed-out rows for closed handles (toggle to show/hide)
- Future tabs can be added for Sockets, Events, Registry, etc.

#### 4. Threads

| Thread ID | Handle | Entry Point | Status | Exit Code |
|-----------|--------|-------------|-----------|-----------|
| 1 (main) | — | 0x140001000 | Running | — |
| 2 | 0x1008 | 0x14000A000 | Exited | 0 |

- TLS slot usage per thread

#### 5. Mutexes

Live table of open mutex handles:

| Handle | Name | Status |
|--------|------|--------|
| 0x2000 | MyAppMutex | Open |
| 0x2004 | GlobalMutex | Closed |

- Shows only Mutex-type handles
- Grayed-out rows for closed handles (toggle to show/hide)
- Displays mutex name/identifier from detail field

#### 6. Windows

Tree view of window hierarchy:

```
▼ HWND 0x2000: "MyApp Window" [TestWindowClass]
  ├─ HWND 0x2004: "Button" [BUTTON]
  ├─ HWND 0x2008: "Edit Control" [EDIT]
  └▼ HWND 0x200C: "Panel" [Panel]
     ├─ HWND 0x2010: "Label" [STATIC]
     └─ HWND 0x2014: "Checkbox" [BUTTON]
```

- Tree layout showing parent-child relationships
- Shows HWND, window title, and class name
- Expandable/collapsible nodes
- Destroyed windows shown grayed out

#### 7. Events (Log)

Chronological stream of all DevEvents, filterable by category:

```
[0.000s] ConfigLoaded  version=Win11  overrides=2
[0.001s] PeLoaded      base=0x140000000  size=0x5A000  sections=6
[0.003s] ImportsResolved  resolved=47  stubbed=12
[0.004s] HandleCreated handle=0x1000  type=File  detail="C:\Windows\System32\..."
[0.005s] FuncCalled    kernel32.dll!GetSystemInfo
[0.005s] FuncCalled    kernel32.dll!GetStartupInfoA
[0.006s] StubHit       gdi32.dll!CreateFontW  ← WARNING HIGHLIGHT
...
```

- Color-coded by severity (normal, warning for stubs hit, error for crashes)
- Pause/resume streaming
- Search/filter by text
- Export to file

---

## Instrumentation Points in rine

Where DevEvents are emitted in the `run()` pipeline:

| Step | Location | Events |
|------|----------|--------|
| Config load | `commands/run.rs` L26-33 | `ConfigLoaded` |
| PE parse | `commands/run.rs` L42-47 | `PeLoaded` |
| Import resolve | `commands/run.rs` L56-78 | `ImportsResolved` |
| Version init | `commands/run.rs` L84 | (included in ConfigLoaded) |
| TEB init | `commands/run.rs` L88 | `ThreadCreated` (main thread) |
| Entry execute | `commands/run.rs` L91 | — |
| Process exit | `loader/entry.rs` | `ProcessExited` |

Runtime instrumentation (in DLL implementations):

| Subsystem | Source | Events |
|-----------|--------|--------|
| Handle table | `rine-types/handles.rs` `insert()`/`remove()` | `HandleCreated` / `HandleClosed` |
| File I/O | `kernel32` file functions | `FileOperation` |
| Memory | `kernel32` VirtualAlloc/HeapAlloc | `MemoryAllocated` / `MemoryFreed` |
| Threading | `kernel32` CreateThread | `ThreadCreated` / `ThreadExited` |
| TLS | `kernel32` TlsAlloc/TlsFree | `TlsAllocated` / `TlsFreed` |
| Registry | `advapi32` Reg* functions | `RegistryAccess` |
| Environment | `kernel32` Get/SetEnvironmentVariable | `EnvVarAccess` |
| Stub hits | `rine-dlls/registry.rs` stub fn | `StubHit` |
| Any DLL call | `rine-dlls/registry.rs` resolve | `FuncCalled` (if tracing enabled) |

### Function Call Tracing

For `FuncCalled` events on every DLL call, we generate **thin wrapper trampolines** around resolved function pointers when dev mode is active. Instead of writing `real_fn_ptr` into the IAT, we write a trampoline that:

1. Emits `FuncCalled { dll, name, is_stub, timestamp }`
2. Tail-calls the real function

This way call tracing has zero cost when dev mode is off (no trampolines generated).

---

## rine-dev Tauri Backend (Rust)

```rust
// crates/rine-dev/src/main.rs

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let socket_path = app.get_cli_matches()...;  // from CLI arg
            let receiver = DevReceiver::bind(&socket_path)?;

            // Background thread: read events, emit to frontend
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                for event in receiver.into_stream() {
                    match event {
                        Ok(ev) => { handle.emit("dev-event", &ev).unwrap(); }
                        Err(_) => break, // rine process exited
                    }
                }
                handle.emit("dev-event", &DevEvent::ProcessExited { exit_code: 0 }).ok();
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state_snapshot,  // Return current accumulated state
        ])
        .run(tauri::generate_context!())
        .expect("failed to run rine-dev");
}
```

The backend accumulates state from the event stream (handle table snapshot, thread list, import table with call counts, memory map) and exposes it via Tauri commands. The frontend listens to `dev-event` for real-time updates.

---

## Frontend

Technology: Vanilla HTML/CSS/JS (matching rine-config's approach) or a lightweight framework. No heavy build tooling.

### Key frontend behaviors:

- **Reactive updates**: Listen to `dev-event` via `listen()`, update DOM
- **State accumulation**: Maintain JS-side maps for handles, threads, imports
- **Filtering**: Client-side filter/search on all tables
- **Export**: Event log exportable as JSON or plain text
- **Auto-scroll**: Event log auto-scrolls to bottom unless user scrolls up (pauses)

---

## Startup Sequence

```
1. User runs:  rine --dev myapp.exe
2. rine (main.rs):
   a. Generates socket path: /tmp/rine-dev-<pid>.sock
   b. Spawns: rine-dev --socket /tmp/rine-dev-<pid>.sock
   c. Creates DevSender, connects to socket
   d. Proceeds with normal run() pipeline, emitting events
3. rine-dev:
   a. Creates DevReceiver, binds socket
   b. Launches Tauri window
   c. Background thread reads events → emits to frontend
4. When PE exits:
   a. rine emits ProcessExited, drops DevSender (closes socket)
   b. rine-dev detects disconnect, shows "Process exited (code X)"
   c. Dashboard remains open for post-mortem inspection
```

**Note**: The socket creation order matters. rine-dev must bind/listen first, then rine connects. So rine spawns rine-dev, waits briefly for the socket to appear (poll with backoff, max ~2s), then connects.

---

## File Structure

```
crates/
  rine-channel/
    Cargo.toml
    src/
      lib.rs          # DevEvent enum, SectionInfo, DllSummary
      sender.rs       # DevSender (UDS client)
      receiver.rs     # DevReceiver (UDS server)
  rine-dev/
    Cargo.toml
    tauri.conf.json
    build.rs
    src/
      main.rs         # Tauri app entry, event bridge
      lib.rs          # Tauri commands, state accumulation
      state.rs        # Accumulated runtime state (handle table, thread list, etc.)
    frontend/
      dist/
        index.html
        style.css
        app.js
    icons/
      ...
```

---

## Workspace Additions

```toml
# Root Cargo.toml [workspace] members addition:
"crates/rine-channel",
"crates/rine-dev",
```

---

## Implementation Phases

### Phase 1: Channel + Static Dashboard
- Create `rine-channel` with DevEvent types and UDS sender/receiver
- Create `rine-dev` Tauri skeleton with Overview tab
- Add `--dev` flag to CLI
- Emit `PeLoaded`, `ConfigLoaded`, `ImportsResolved` events from `run()`
- Display static load-time information in dashboard

### Phase 2: Handle & Thread Tracking
- Instrument HandleTable `insert()`/`remove()` with `dev_emit!`
- Instrument thread creation/exit
- Add Handles and Threads tabs to frontend

### Phase 3: Function Call Tracing
- Generate IAT trampolines for call counting in dev mode
- Add Imports tab with call counts
- Add `StubHit` events from stub function

### Phase 4: Live Event Stream
- Events tab with streaming log
- Filter, search, export
- Memory tracking events

### Phase 5: Polish
- Status bar with live counters
- Post-mortem mode (process exited, data frozen)
- Keyboard shortcuts, accessibility
