# EXE Internal Icon Thumbnailer Implementation Plan

## Goal

Show embedded Windows executable icons in Linux file browsers by generating thumbnails for PE files (`.exe`, optionally `.dll`, `.msi`, `.cpl`) across:

- GNOME Files (Nautilus)
- Nemo
- KDE Dolphin

## Success Criteria

- Browsing a folder with `.exe` files displays per-file extracted icons, not a generic executable icon.
- Works on a default GNOME session and a default KDE Plasma session after installation.
- Fails safely (generic icon fallback) on malformed, oversized, or hostile PE files.
- Thumbnail generation is deterministic and fast enough for normal browsing.

## High-Level Architecture

Implement this in three layers:

1. Core extractor and renderer (Rust)
2. Desktop integration adapters (GNOME/Nemo and KDE)
3. Packaging and install hooks

### 1) Core Extractor (shared Rust crate)

Create a new crate in this workspace (suggested name: `crates/rine-thumbnailer-core`) that:

- Accepts an input path and target size.
- Parses PE resources (`RT_GROUP_ICON`, `RT_ICON`) for 32-bit and 64-bit binaries.
- Selects best icon variant for requested size and quality.
- Decodes icon payloads (BMP/PNG inside ICO resources).
- Renders to PNG at requested size with alpha preserved.
- Exposes:
  - Rust library API
  - CLI-friendly function signatures for adapters

Suggested core API:

```rust
pub struct ThumbnailRequest<'a> {
    pub input_path: &'a std::path::Path,
    pub size_px: u32,
}

pub enum ThumbnailError {
    UnsupportedFormat,
    NoIconResource,
    MalformedResource,
    DecodeFailure,
    Io(std::io::Error),
    // ...
}

pub fn generate_png_thumbnail(req: &ThumbnailRequest) -> Result<Vec<u8>, ThumbnailError>;
```

Implementation notes:

- Prefer existing parsing/image crates where stable, then add small custom logic only where needed.
- Keep strict bounds checks for all resource offsets and lengths.
- Enforce max decoded dimensions and memory budget.
- Add a short processing timeout in adapter layer (not deep in pure parser logic).

### 2) Integration Adapters

#### GNOME Files + Nemo (Freedesktop thumbnailer)

Create a CLI binary crate (suggested: `crates/rine-thumbnailer-cli`) that:

- Matches Freedesktop thumbnailer contract.
- Inputs: source URI/path + output PNG path + requested size.
- Calls core crate and writes PNG atomically to output.
- Returns non-zero on hard failure.

Install descriptor file:

- Path: `/usr/share/thumbnailers/rine.thumbnailer` (or user-local equivalent)
- Declares handled MIME types, for example:
  - `application/x-ms-dos-executable`
  - `application/x-msdownload`
  - `application/vnd.microsoft.portable-executable`
  - `application/x-dosexec`

Descriptor skeleton:

```ini
[Thumbnailer Entry]
TryExec=/usr/bin/rine-thumbnailer
Exec=/usr/bin/rine-thumbnailer %u %o %s
MimeType=application/x-ms-dos-executable;application/x-msdownload;application/vnd.microsoft.portable-executable;application/x-dosexec;
```

Notes:

- `%u` may be URI-encoded; adapter must decode robustly.
- Confirm behavior in both Nautilus and Nemo; both use Freedesktop thumbnail cache patterns.

#### KDE Dolphin (KIO thumbnailer)

KDE typically expects a KIO thumbnailer plugin, not only a `.thumbnailer` descriptor.

Plan:

- Build a small KDE plugin in C++ implementing `ThumbCreator`.
- Plugin delegates extraction/rendering to the Rust CLI (`rine-thumbnailer`) via process call.
- Register MIME support in plugin metadata.

Why this approach:

- Keeps PE parsing in Rust (single source of truth).
- Minimizes C++/Qt code to glue only.
- Reduces duplicate parser bugs.

Alternative (future):

- Native Rust-to-Qt binding or fully native C++ parser, if performance/process overhead becomes a concern.

### 3) Packaging and Installation

Add install assets and scripts to this repository:

- Install binary to `/usr/bin/rine-thumbnailer`.
- Install GNOME/Nemo descriptor to `/usr/share/thumbnailers/rine.thumbnailer`.
- Install KDE plugin to distro-appropriate KIO plugin path.

Post-install hooks:

- Refresh MIME database where needed (`update-mime-database`).
- Refresh desktop DB (`update-desktop-database`) if required by distro packaging.
- In docs, mention restarting file manager or clearing thumbnail cache when validating.

Debian packaging targets for this repo:

- Add package entries/scripts under existing Debian build flow (`target/debian/...` generation path is already used in this workspace).
- Keep package split optional:
  - `rine-thumbnailer-core` (internal)
  - `rine-thumbnailer` (CLI + GNOME/Nemo)
  - `rine-thumbnailer-kde` (KIO plugin)

## Detailed Work Breakdown

## Phase 0: Discovery (0.5 day)

- Verify current crate boundaries and where platform integration belongs.
- Validate available shared helpers in `rine-types` before adding utility code.
- Enumerate MIME values seen in target distros for `.exe`.

Deliverable:

- Short design notes and final crate/module placement.

## Phase 1: Core PE Icon Extraction (1-2 days)

- Add new core crate and initial API.
- Parse and map group icon resources to icon bitmaps.
- Choose best match by requested size and bit depth.
- Output PNG bytes.
- Add fixture-based unit tests:
  - PE with multiple icon sizes
  - PE with PNG-compressed icon entries
  - PE with malformed resource tables
  - No-icon PE

Deliverable:

- Green unit tests in core crate.

## Phase 2: CLI Thumbnailer for GNOME/Nemo (1 day)

- Implement CLI argument parsing and URI/path handling.
- Connect CLI to core API.
- Implement atomic write to `%o`.
- Add integration test for end-to-end thumbnail generation.
- Add `.thumbnailer` template and install docs.

Deliverable:

- Working thumbnails in Nautilus and Nemo on a test machine.

## Phase 3: KDE Dolphin Integration (2-4 days)

- Create minimal KIO `ThumbCreator` plugin.
- Register supported MIME types.
- Delegate generation to CLI and return `QImage`.
- Handle timeout and failure path to prevent UI hangs.
- Validate in Dolphin with cold and warm cache.

Deliverable:

- Working thumbnails in Dolphin.

## Phase 4: Hardening and Packaging (1-2 days)

- Add size and memory guards.
- Add coarse timeout handling.
- Add logging (disabled by default, opt-in env var).
- Package integration for deb build scripts.
- Add troubleshooting docs.

Deliverable:

- Installable package artifacts and reproducible validation steps.

## Security and Reliability Requirements

- Treat all PE files as untrusted.
- Never trust embedded offsets without bounds checks.
- Cap decoded image dimensions (for example 1024 or 2048 max side).
- Cap overall decode memory budget.
- Reject suspicious resource recursion/depth.
- Avoid panics on malformed inputs; return structured errors.

## Performance Targets

- Median thumbnail generation: < 50 ms for typical small/medium executables.
- P95 thumbnail generation: < 200 ms on moderate hardware.
- Avoid expensive re-parsing where possible; rely on file manager cache semantics.

## Testing Plan

Unit tests:

- Resource parser coverage for valid and malformed inputs.
- Icon selection logic by size/depth.
- Decoder behavior for BMP and PNG icon payloads.

Integration tests:

- CLI invocation with local file path and URI input.
- Verify PNG output dimensions and alpha channel presence.

Manual desktop verification matrix:

- GNOME Files on a GNOME distro session.
- Nemo on Cinnamon (or Nemo standalone on GNOME).
- Dolphin on KDE Plasma.

Regression tests:

- Corrupt PE fuzz corpus should not crash.
- Timeout and fallback behavior on large files.

## Repository Changes (proposed)

- New crates:
  - `crates/rine-thumbnailer-core/`
  - `crates/rine-thumbnailer-cli/`
  - optional `crates/rine-thumbnailer-kde/` (or external plugin subproject)
- New assets:
  - `packaging/thumbnailers/rine.thumbnailer`
  - KDE plugin metadata/install files
- Docs:
  - Add section to `README.md`
  - Optional deeper doc: `docs/thumbnailer-integration.md`

## Risks and Mitigations

- KDE plugin ABI/packaging variance across distros.
  - Mitigation: keep KDE plugin tiny; use distro-specific packaging recipes.
- MIME mismatch by distro leading to no invocation.
  - Mitigation: include multiple known PE MIME aliases and document verification command.
- Malformed PE attack surface.
  - Mitigation: strict bounds checks, fuzzing corpus, caps and timeouts.

## Timeline Estimate

- MVP (GNOME + Nemo): 2-4 days
- KDE Dolphin support added: +3-7 days
- Polished and packaged across targets: total ~1-2 weeks

## First Execution Slice (recommended)

Implement this narrow slice first:

1. Build `rine-thumbnailer-core` with tests against fixture `.exe` files.
2. Build `rine-thumbnailer-cli` and generate PNG from one fixture.
3. Add `.thumbnailer` descriptor and verify in Nautilus.

Once this slice is stable, add KDE plugin integration.