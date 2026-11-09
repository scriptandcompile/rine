# Glossary

The words this repo uses for its own domain. Use them exactly in code, doc
comments, log messages, commit messages, and issue text; do not invent
synonyms. Architecture vocabulary (module, interface, seam, adapter, depth,
leverage, locality) comes from the codebase-design standards and is not
repeated here.

## Runtime

| Term | Meaning |
|---|---|
| guest | The Windows PE executable rine loads and runs, and the code inside it. |
| host | The Linux process and OS that rine runs on. |
| loader | The part of rine that maps the guest image, applies relocations, and resolves imports. |
| import resolution | Matching a guest's imported DLL functions to rine's implementations, then writing the pointers into the guest's IAT. Fails before entry if an import is unimplemented. |
| entry point | The guest address execution starts at, after import resolution succeeds. |

## DLL layer

| Term | Meaning |
|---|---|
| DLL plugin | One crate that reimplements a Windows DLL and registers its exports. |
| export | A single Windows API function a DLL plugin offers. |
| wrapper (ABI) crate | `rine64-<dll>` or `rine32-<dll>`: reads guest pointers, decodes `A`/`W` strings, calls the common implementation. Holds no logic. |
| common crate | `rine-common-<dll>`: the bitness-neutral, safe-Rust implementation of one DLL's behavior. |
| maturity | An export's declared state: `implemented`, `partial`, or `stubbed`. Declared with an attribute and parsed by codegen. |
| dynamic provider | The C-ABI seam a DLL plugin `.so` exposes so the loader can load it at runtime. |
| forwarder | An export that resolves to a function in another DLL. Not yet resolved by rine. |

## Handles & state

| Term | Meaning |
|---|---|
| handle | An opaque id from an allocator, never a host pointer. Each handle space documents its base and its owner. |
| handle space | One allocator's range of handles: kernel objects, HWNDs, GDI objects, device contexts. |
| registry | A table mapping handles to records. Process-wide today; per-session per roadmap R10. |
| session | The planned per-run bundle of registries (roadmap R10). Not implemented; the Device Context module takes received state as its first step. |

## Windowing (user32)

| Term | Meaning |
|---|---|
| window | A top-level or child window record in the window registry, addressed by HWND. |
| top-level window | A window with no parent window; owns a backend host window. |
| child window | A window hosted inside a top-level window; never creates its own host window. |
| window class | A registered class: style bits, default procedure, background, and the control identity when the class is a built-in control. |
| built-in control | A control whose class rine registers itself: BUTTON, EDIT, STATIC. |
| control identity | Which built-in control a window is. Recorded at class registration; must not be re-derived by matching class-name strings. |
| window procedure | The guest (or rine) callback that receives a window's messages. |
| default procedure | rine's window procedure for a window whose class registered the internal default. |
| message queue | The per-thread queue of messages for a guest thread. |
| paint session | The `BeginPaint`/`EndPaint` pair: validate the update region, fill `PAINTSTRUCT`, trigger present. |
| backend adapter | The window host behind the windowing seam: `native` (winit + X11) or `emulated` (socket to an external host process). Chosen at compile time. |
| present | Publishing a window's surface to the screen. |
| composition | The path that assembles a top-level frame from window and control surfaces. |

## Graphics (gdi32)

| Term | Meaning |
|---|---|
| device context (DC) | The drawing target a guest selects objects into and draws through. Every DC has exactly one surface. |
| window DC | A DC obtained by `GetDC`/`GetWindowDC`, bound to a window's client or whole window surface. |
| paint DC | The DC `BeginPaint` hands out, valid for one paint session. |
| memory DC | A DC created by `CreateCompatibleDC`, drawing into a bitmap the guest selects. |
| surface | The pixels a DC draws into. |
| GDI object | A selectable drawing object: bitmap, brush, pen, or font. |
| selected object | The GDI object a DC currently draws with. One per kind per DC. |
| stock object | A predefined GDI object addressable by a fixed id, such as `WHITE_BRUSH`. |

## Docs & process

| Term | Meaning |
|---|---|
| export docs | The doc comment on an export: summary, `# Arguments`, `# Safety`, `# Returns`, and for `partial`/`stubbed` a "Missing implementation features" block. Parsed by tooling, so the shape is load-bearing. |
| dll-support data | The generated JSON describing per-DLL function coverage, consumed by the project website. |
| fixture | A C program under `tests/fixtures/src`, compiled to PE and run as a guest to assert end-to-end behavior. |
