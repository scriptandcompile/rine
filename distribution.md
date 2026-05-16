# Distribution Packaging Plan (KDE Focus)

This document captures the packaging process for making `rine` feel first-class on KDE Plasma while remaining cross-distro friendly.

## Goal

Ship a package that:
- installs cleanly on KDE-based Linux distros,
- integrates with Dolphin and KDE app discovery tools,
- preserves Freedesktop compatibility for other desktops.

## Recommended Targets

1. Debian/Ubuntu KDE: `.deb` package (current pipeline).
2. Fedora KDE: `.rpm` package.
3. Arch KDE: `PKGBUILD`.
4. Optional cross-distro channel: Flatpak (good Discover experience with proper metadata).

## KDE-Focused Requirements

### 1. Desktop Metadata and App Discovery

- Provide AppStream metadata (`/usr/share/metainfo/*.metainfo.xml`) so KDE Discover can show rich listing details.
- Keep user-facing launchers visible (for example `rine-dev`, `rine-config`) and hide helper entries where appropriate.
- Install icons in standard hicolor locations at multiple sizes.

### 2. MIME and File Association Behavior

- Install MIME XML and desktop entries through standard Freedesktop paths.
- Set default handlers per-user, not globally.
- Validate behavior via `xdg-mime` and KDE file association settings.

### 3. Dolphin Integration

- Install a system ServiceMenu file under `/usr/share/kio/servicemenus/` for right-click actions.
- Refresh KDE service caches after install/removal (`kbuildsycoca6` when present).

## Packaging Script Work (Debian Baseline)

Extend the Debian packaging pipeline to include:

1. AppStream metadata in `/usr/share/metainfo`.
2. Dolphin ServiceMenu in `/usr/share/kio/servicemenus`.
3. Icons in `/usr/share/icons/hicolor/<size>x<size>/apps`.
4. Post-install and post-remove refresh hooks for:
   - `update-mime-database`
   - `update-desktop-database`
   - `kbuildsycoca6` (best-effort)

## Validation Checklist

Run in a clean KDE VM/container image:

1. Install package.
2. Verify launcher visibility and icon rendering.
3. Verify Dolphin right-click actions are present.
4. Verify `Open With` and double-click behavior for `.exe` files.
5. Verify uninstall cleanup for MIME, ServiceMenu, and defaults.
6. Test package upgrade path (old -> new).

## QA/Policy Tools

- Debian: run `lintian` on generated `.deb`.
- Fedora: run `rpmlint` on built `.rpm`.
- Arch: run namcap checks where applicable.

## Suggested Delivery Phases

### Phase 1: KDE-polished Debian package

- Add AppStream + system Dolphin ServiceMenu + icon install + cache refresh hooks.
- Estimated effort: 1-2 days.

### Phase 2: Multi-distro native packages

- Add RPM spec and Arch PKGBUILD with equivalent integrations.
- Estimated effort: 3-5 days total.

### Phase 3 (Optional): Flatpak

- Add Flatpak manifest and Discover-friendly metadata assets.
- Estimated effort: +1-2 days.

## Future Implementation Notes

When we resume:
- Start from `scripts/build-rine-deb.sh` and wire all KDE artifacts there first.
- Keep behavior consistent with existing per-user desktop integration logic.
- Confirm no global default association is forced during system install.
