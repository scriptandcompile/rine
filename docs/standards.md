# Coding Standards

How code in this repo should be written. The Standards axis of the
`code-review` skill checks diffs against these rules. Rules already enforced
by tooling (`cargo fmt`, clippy, `cargo check`) are not repeated here.

## The Win32 boundary — the central rule

rine reimplements the Windows interface. Two halves, one rule each:

- **At the interface: be Windows.** An exported name, its parameters, its
  return type, its calling ABI (`extern "win64"` on x86_64, `extern "stdcall"`
  on x86), its failure convention (`BOOL::FALSE` / `NULL` / `0` plus
  `GetLastError`), and its documented side effects are the contract. A guest
  binary cannot tell rine from Windows by calling it. Deviating from the
  published Win32 behavior is a bug, even when the deviation is "nicer".
- **Under the interface: be modern Rust.** The bitness crates
  (`crates/platform/win64-dll/*`, `crates/platform/win32-dll/*`) exist only to
  read guest pointers, decode `A`/`W` strings, and convert Win32 scalars into
  Rust types. Everything past that point is safe Rust: owned or borrowed
  `String`s, enums, structs, and `Result`s — never `LPCSTR`, never raw
  pointers, never `unsafe`.

Consequences:

- **Write every API once.** One bitness-neutral implementation in
  `crates/platform/win-common/rine-common-<dll>`, spelled `snake_case`. The
  `A`/`W` pair and the 32/64 pair are decoders at the boundary that converge
  on it. If a genuine ABI difference exists (e.g. `wsprintf` varargs), keep
  the special-case in the bitness crate and say why in a comment.
- **Never fake the interface.** Every export carries its maturity attribute
  (`#[rine_dlls::implemented]`, `#[partial]`, or `#[stubbed]`). A `partial` or
  `stubbed` export warns once when imported and lists its missing behavior in
  its doc comment's "Missing implementation features" block. Silent
  pretending — returning `TRUE` for work not done — is the worst offense in
  this repo.
- **Fail loudly at import time.** An unimplemented import fails the resolution
  report before the guest runs. Do not route guests toward
  `stub_function`'s abort as a strategy; roadmap R3 replaces it with a
  configurable disposition.

## Workspace & crates

- One Windows DLL gets the three-crate pattern: `rine-common-<dll>` (the
  implementation), `rine64-<dll>` and `rine32-<dll>` (the ABI adapters and
  export metadata). Shared machinery shared by several DLLs goes in
  `crates/platform/win-common/shared/<thing>` (see `dbcs`, `text`,
  `threading-runtime`).
- A new member crate must be added to `[workspace] members` (and
  `default-members` unless it is arch-specific), and its dependencies come
  from `[workspace.dependencies]` with `{ workspace = true }`.
- Package metadata (`version`, `edition`, `license`) is inherited from
  `[workspace.package]`.
- Any lint allowance (`#[allow(...)]` except the mechanical
  `#[allow(non_snake_case)]` on exports) needs a comment saying why.

## Naming & modules

- Exported functions keep their exact Win32 PascalCase name. Common-crate
  functions are `snake_case` verbs: `CreateFileW` decodes and calls
  `create_file`.
- Modules follow the Win32 category the APIs belong to: `file.rs`,
  `memory.rs`, `window_lifecycle.rs`, `device_context.rs`. Splitting a grown
  module into a directory of seam-named submodules is preferred over a
  thousand-line file.
- `lib.rs` opens with `//!` docs listing the modules; keep the list current.
- Do not invent domain synonyms for Win32 terms in identifiers or docs. When
  the repo's own concept has a name, `GLOSSARY.md` owns it; use that word in
  code, docs, and logs alike.

## unsafe & guest memory

- `unsafe fn` belongs to the bitness crates, at the point where guest memory
  is touched. Each one gets a `# Safety` section stating the guest contract
  (non-null, aligned, readable for N bytes).
- Common crates are safe. When one genuinely cannot be (a raw handle in, a
  callback pointer through), the exception gets a comment and a `# Safety`
  doc — and stays the exception.
- Guest pointers are read through the `rine-types` string/pointer helpers
  (`read_string`, `LPCWSTR` machinery), never by ad-hoc `ptr::read`.
- A callback the guest handed us (window procedures, exception filters) is
  called back through the ABI closure passed in by the bitness crate; the
  common crate never transmutes function pointers itself.

## Handles

- Handles are opaque ids from an allocator, never host pointers. Each
  registry (kernel handles, HWNDs, GDI objects, window DCs) documents its
  handle base and its owner.
- A handle handed out must stay valid until the API that closes it runs, and
  closing must free the record. Cross-registry handle values must not
  collide: one allocator per handle space, and the spaces document their
  ranges where they share `usize`.

## Errors & return conventions

- At the boundary: Win32 conventions — `BOOL`, `HRESULT`, `NULL`/`0`, and
  `LAST_ERROR` set on failure via the kernel32 last-error seam.
- Inside: Rust `Result` with a `thiserror` enum per crate where failures are
  meaningful; convert to the Win32 convention at the boundary, not before.
- No `unwrap()`/`expect()` on fallible domain data in library code (tests
  may, with a message). `.lock().unwrap()` on our own mutexes is tolerated
  while registries are process-global; a poisoning panic means a bug, and
  per-session state (roadmap R10) will remove the pattern.

## Export documentation is load-bearing

The build script, the docs generator, and the validator all parse the export
doc comments. Keep the shape stable:

- one-line summary matching Microsoft's name for what the function does,
- `# Arguments` — one bullet per parameter,
- `# Safety` — the guest-memory contract,
- `# Returns` — success and failure values, spelled the way Win32 documents
  them,
- `Missing implementation features` — for `partial`/`stubbed` only, the
  concrete behaviors not yet honored.

The three-way duplication between the win64 crate, the win32 crate, and the
common crate's docs is real today; keep the copies in sync until codegen
generates the wrappers.

## Logging & output

- Library crates log with `tracing` macros; no `println!`/`eprintln!` in
  library code (the ad-hoc `RINE_USER32_DEBUG` env prints are legacy — prefer
  `debug!`).
- Keep the guest's stderr clean: rine's own logs must not interleave into
  the guest's stdio. The oracle wire contract (roadmap R1) makes stderr a
  parsed channel; new logging goes to the configurable sink, not stderr.
- Structured developer events go through `dev_notify!` / `rine-channel`, not
  free-form logs.

## Global state (interim rule)

Registries are process-wide statics today (`WINDOW_MANAGER`, `GDI_STATE`,
`HANDLE_TABLE`, …). Until per-session state (roadmap R10) lands:

- Do not add a new process-wide static without naming its owner and its
  handle space in a comment.
- Prefer threading state through parameters when writing new modules — the
  Device Context work and the Session direction both assume it.
- A test touching a shared static serializes behind a module-level
  `TEST_MUTEX`; never rely on execution order across test files.

## Testing

- Unit tests live in `#[cfg(test)] mod tests` at the bottom of the file they
  test, and test behavior through the crate's public interface, not by
  reaching into statics.
- New end-to-end behavior gets a C fixture in `tests/fixtures/src/<domain>/`,
  built for both arches, asserted in `crates/rine/tests/integration/` on
  exit code and stdout (prior art: `ui/gdi.rs` over `gdi_objects.c` and
  `gdi_rendering.c`).
- A new or upgraded export gets: a unit test in the common crate, and one
  fixture that crosses the real ABI (both bitnesses when relevant).
- Fixtures must not call APIs rine lacks just to pass; the roadmap R9
  direction replaces self-asserting fixtures with differential ones.

## Writing that reads

These rules bind all prose: doc comments, module docs, commit messages,
glossary entries, ADRs, and replies to the human.

- One sentence, one claim. Semicolon long enough to be two sentences? Split
  it or bullet it.
- Every sentence names an actor and a verb. Never nominalize a verb to avoid
  naming an actor; coined compounds are banned.
- The first sentence of any doc comment, commit body, or ADR is readable
  without the glossary. Later sentences may then use glossary terms for
  precision.
- Keep sentences under about 25 words. End on the fact, not the flourish.

## Commits & process

- Lowercase, plain English, past-tense verb: `added <thing>`,
  `fixed <thing> in <crate>`, `consolidated <thing>`.
- One commit per working step; the subject says what the code now does.
- API coverage changes keep the maturity attributes and export docs updated in
  the same commit — the validator (`scripts/check-attribute-metadata.py`,
  run by `scripts/full_build.sh`) fails the build otherwise.
