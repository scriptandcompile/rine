# rine Roadmap — the oracle track

> **Purpose:** index of open tracks toward one objective: **make rine a
> testing oracle good enough to replace Wine in calxgloss's capture lanes**
> (baseline, reference-app, scenario). API conformance work continues
> underneath, but coverage picks are steered by this roadmap — the loader
> family first, then wherever the differential runner says we diverge.
>
> **Where this comes from:** the calxgloss research corpus
> `docs/research/rine-oracle/` (reviewed at this repo's `da46c32`). Its
> finding: rine's strategic advantage over Wine as a harness is that one
> codegen seam — `rine-dll-build::generate_metadata_code` + `DllRegistry` —
> can serve tracing, hooking, mocking, and provenance at once. The tracks
> below widen that seam before the API list grows past it.
>
> **Rule of this file:** table order = dependency order. A picked-up track
> gets a GitHub issue (`scriptandcompile/rine`) with the design; done tracks
> are removed from the matrix and their decisions land as code + notes here.

## The wire contract we must honor

calxgloss drives its emulator through a narrow contract, pinned in
`crates/pipeline/calxgloss-testgen/src/wine.rs`. To drop into those lanes
`CALXGLOSS_WINE`-style, rine must:

- answer `--version` with a parseable version string;
- run a cross-compiled `x86_64-pc-windows-gnu` console EXE with piped
  stdio and a controlled env/CWD, and exit with the guest's own exit code —
  including a crash path that exits `133` via the harness's
  `SetUnhandledExceptionFilter` → `TerminateProcess`, with no debugger and
  no GUI;
- keep **stderr free of rine's own logs** (calxgloss scans stderr for the
  harness's `module base = 0x…` line);
- support a host-path drive mapping that tolerates non-ASCII host paths.

R1 makes this testable; the cutover definition-of-done (§Endgame) replays it.

## Status matrix

| # | Track | Area | Status | Blocked by | Next step |
|---|-------|------|--------|-----------|-----------|
| R1 | **Provenance + CLI discipline** — parseable `--version`; every run writes a machine-readable run record: rine build hash, guest image sha256, actual load base, config hash, host triple; add `EnvFilter` to the tracing subscriber (`RUST_LOG` is inert today, `crates/rine/src/main.rs:18`); default log sink off stderr | `rine`, `rine-runtime-core`, `rine-types` | ⬜ not started | — | file the issue with the run-record schema |
| R2 | **Finish the loader family** — `LoadLibraryA/W/ExW` (incl. `DONT_RESOLVE_DLL_REFERENCES`, search path, `DllMain`), `GetProcAddress` with **forwarders** and ordinal lookup, `FreeLibrary`, `GetModuleHandleA/W`, `GetModuleHandleExW(FROM_ADDRESS)`, **delay-import resolution** (`resolver.rs:252` currently warns-only), and emit `#[ordinal]` exports from codegen (parsed but never generated, `rine-dll-build/src/lib.rs:597`) | `rine-common-kernel32`, `rine-runtime-core`, `rine-dll-build` | ⬜ not started | — | spike: `DONT_RESOLVE_DLL_REFERENCES` mapping + export-table reads |
| R3 | **Memory + exception fidelity as data** — `VirtualQuery`/`VirtualProtect` answering from `VIRTUAL_REGIONS` incl. host-allocated (non-module) regions and `PAGE_GUARD` reported without faulting; exact exception chain (SEH → `SetUnhandledExceptionFilter` → `TerminateProcess` exit-code path); privileged instructions (INT1/ICEBP) surfaced as named fault records, not crashes; replace `stub_function`'s `abort()` (`registry.rs:490`) with a configurable disposition (return-0 / raise / terminate-case) | `rine-common-kernel32`, `rine-runtime-core`, `rine-dlls` | ⬜ not started | R2 (lookup semantics) | design note on the exception dispatch path |
| R4 | **API-call recording** — optional recording wrapper emitted by `generate_metadata_code()` around every registered export: name, scalar args, return value, and the `implemented`/`partial`/`stubbed` maturity tag; stream as a new `DevEvent::ApiCall` on the existing socket (`rine-channel`) and/or write `trace.json` beside the run | `rine-dll-build`, `rine-dlls`, `rine-types`, `rine-channel` | ⬜ not started | — | wrapper codegen prototype on one DLL |
| R5 | **Hermetic runs + determinism knobs** — in-memory config/registry option (no `$HOME` writes; today state is keyed by the guest's absolute path, `config.rs:240-277`); scratch-root override; wire the dead `PathTranslator` (`crates/rine/src/subsys/filesystem.rs:51`) so `[filesystem.drives]` and non-ASCII host paths work; virtual clock + seeded RNG behind config; per-run hang budget at the exec seam; port the `golden-harness` branch's canonical-env knobs (`RINE_CANONICAL_OS_BUILD` etc.) onto main | `rine-types`, `rine-config`, `rine-common-*` (time/env) | ⬜ not started | R1 | rebase or cherry-pick `golden-harness` |
| R6 | **Provider ABI as hook/mock framework** — extend the dynamic-provider ABI (`dynamic_provider.rs`) with **per-export override** and a typed **call-through spy** (wrap one export, keep the DLL's rest); wire `DllConfig { search_order, force_stub }` (`config.rs:181`, parsed but never read) so a DLL can be forced to stub mode from config; post-load IAT re-patch utility for tests that hook after bind | `rine-dlls`, `rine-runtime-core`, `rine-types` | ⬜ not started | R2 | ABI v2 sketch: override + call-through descriptors |
| R7 | **Embedding API** — `crates/rine` gains `src/lib.rs`: a session builder (guest path, env allowlist, hermetic opts, dev-event callback, provenance out, exit code in); driver code is typed Rust, no stdin/stdout wire protocol needed; keep process-per-guest as the isolation boundary | `rine` (lib target) | ⬜ not started | R4, R5 | API sketch mirroring `run.rs:347-546` as a struct |
| R8 | **State snapshots + queries** — generalize `rine-memory-snapshot-v1` into a query format over the dev socket / embedding API: handle table, virtual-region map, window/GDI registries, TEB/PEB; session-scoped enumeration of guest threads/child processes (so consumers stop sniffing `/proc`) | `rine-runtime-core`, `rine-types`, `rine-channel` | ⬜ not started | R4 | define the query schema; region-map dump first |
| R9 | **Differential conformance runner** — port a slice of Wine's LGPL conformance tests (kernel32/user32 first); one runner executes the same cases under Wine **and** rine, diffs the captures, and files divergences as coverage bugs with the R4 trace attached; replaces the self-asserting C fixtures over time (`tests/fixtures` currently avoid APIs rine lacks) | `crates/rine/tests`, new `conformance/` harness | ⬜ not started | R2, R3, R4 | license check + port of `kernel32` thread tests |
| R10 | **Per-session context** — move `HANDLE_TABLE`, `TLS_BITMAP`, `VIRTUAL_REGIONS`/`DEFAULT_HEAP`, window/GDI registries, version globals behind a session struct; enables multi-guest-per-process, replay from recorded events, and snapshot/restore | `rine-types`, `rine-common-*`, `rine-runtime-core` | ⬜ not started | R7, R8 | inventory of the singletons with owners (audit-only PR) |

## Ordering rationale

- **R1 first** because it is the acceptance gate for everything else and
  costs a day — an oracle nobody can version-stamp is unusable regardless of
  coverage.
- **R2–R3 before breadth**: these are the APIs calxgloss's *harness itself*
  calls (map a DLL, read exports, RWX trampolines, bounded pointer probes,
  exception filter). Without them no real harness EXE loads — they buy the
  instrument, not just coverage. Fixing `GetModuleHandleExW(FROM_ADDRESS)`
  also deletes a workaround calxgloss carries for Wine's own bug.
- **R4 runs in parallel from day one** — it is the highest-leverage item and
  independent of the loader family; every future conformance drop then emits
  a captured trace for free.
- **R5 before any baseline is trusted** (hermetic + provenance = reproducible
  captures); **R6 after R2** (per-export mocks need ordinal/forwarder
  semantics to be meaningful).
- **R7–R8** turn the CLI product into an embeddable instrument once the
  events (R4) and hermetic config (R5) exist to expose.
- **R9** makes conformance work self-steering: divergence diffs pick the next
  API targets, and each divergence carries its exact call + maturity tag.
- **R10** is long-horizon; do the singleton audit early, the refactor late.

## Endgame — definition of done for the oracle objective

The objective is met when calxgloss points `CALXGLOSS_WINE` at a rine
wrapper and, against its existing fixture DLLs and scenario fixtures:

1. the wire contract above replays green (batch verdicts, crash→exit 133,
   clean stderr, `--version` preflight);
2. baseline captures carry rine's run record (P44 on the calxgloss side
   consumes `provenance.json` instead of inferring it);
3. the scenario lane gets its guest process list from R8's session query
   instead of `/proc` preloader sniffing;
4. the proxy-DLL capture lane shrinks: R4's `ApiCall` trace covers what the
   recording proxy exists to record.

Wine stays as the *reference* oracle R9 diffs against — the goal is not to
delete Wine from the picture, but to stop relying on it as the thing we
execute under.
