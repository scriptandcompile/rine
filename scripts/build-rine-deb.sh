#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_32="i686-unknown-linux-gnu"

HOST_PROVIDER_LIBS=(
    "librine64_kernel32.so"
    "librine64_msvcrt.so"
    "librine64_ntdll.so"
    "librine64_advapi32.so"
    "librine64_gdi32.so"
    "librine64_comdlg32.so"
    "librine64_user32.so"
    "librine64_ws2_32.so"
)

TARGET32_PROVIDER_PACKAGES=(
    "rine32-kernel32"
    "rine32-msvcrt"
    "rine32-ntdll"
    "rine32-advapi32"
    "rine32-gdi32"
    "rine32-comdlg32"
    "rine32-user32"
    "rine32-ws2_32"
)

TARGET32_PROVIDER_LIBS=(
    "librine32_kernel32.so"
    "librine32_msvcrt.so"
    "librine32_ntdll.so"
    "librine32_advapi32.so"
    "librine32_gdi32.so"
    "librine32_comdlg32.so"
    "librine32_user32.so"
    "librine32_ws2_32.so"
)

if ! command -v dpkg-deb >/dev/null 2>&1; then
    echo "error: dpkg-deb is required to build Debian packages" >&2
    exit 1
fi

if ! command -v rustup >/dev/null 2>&1; then
    echo "error: rustup not found; cannot ensure $TARGET_32 target is installed" >&2
    echo "hint: install rustup, then run: rustup target add $TARGET_32" >&2
    exit 1
fi

if ! rustup target list --installed | grep -qx "$TARGET_32"; then
    echo "Installing missing Rust target: $TARGET_32"
    rustup target add "$TARGET_32"
fi

cd "$REPO_ROOT"

VERSION="$(sed -n 's/^version = "\([^"]*\)"$/\1/p' Cargo.toml | head -n1)"
if [[ -z "$VERSION" ]]; then
    echo "error: failed to resolve workspace version from Cargo.toml" >&2
    exit 1
fi

DEB_VERSION="$VERSION"

case "$(dpkg --print-architecture)" in
    amd64|arm64|armhf|i386)
        DEB_ARCH="$(dpkg --print-architecture)"
        ;;
    *)
        echo "error: unsupported Debian architecture: $(dpkg --print-architecture)" >&2
        exit 1
        ;;
esac

OUT_DIR="$REPO_ROOT/target/debian"
STAGING_DIR="$OUT_DIR/.staging"
BRAND_ICON_SOURCE="$REPO_ROOT/crates/rine-frontend-common/assets/rine-mark.svg"
ICON_256_RINE_SOURCE="$REPO_ROOT/crates/rine-config/icons/icon.png"
ICON_256_RINE_DEV_SOURCE="$REPO_ROOT/crates/rine-dev/icons/icon.png"
mkdir -p "$OUT_DIR" "$STAGING_DIR"

if [[ ! -f "$BRAND_ICON_SOURCE" ]]; then
    echo "error: expected brand icon not found: $BRAND_ICON_SOURCE" >&2
    exit 1
fi

if [[ ! -f "$ICON_256_RINE_SOURCE" ]]; then
    echo "error: expected 256px icon not found: $ICON_256_RINE_SOURCE" >&2
    exit 1
fi

if [[ ! -f "$ICON_256_RINE_DEV_SOURCE" ]]; then
    echo "error: expected 256px icon not found: $ICON_256_RINE_DEV_SOURCE" >&2
    exit 1
fi

echo "==> Building release binaries for plain rine (dev feature disabled)"
cargo build --release -p rine --no-default-features -p rine-config

BIN_RINE_NODEV="$STAGING_DIR/rine-nodev"
install -m 0755 "$REPO_ROOT/target/release/rine" "$BIN_RINE_NODEV"

echo "==> Building release binaries for rine-dev package (current default behavior)"
cargo build --release -p rine -p rine-dev -p rine-config

echo "==> Building 32-bit helper runtime"
cargo build --release -p rine32 --target "$TARGET_32"

echo "==> Building 32-bit dynamic providers"
for package in "${TARGET32_PROVIDER_PACKAGES[@]}"; do
    cargo build --release --target "$TARGET_32" -p "$package"
done

BIN_RINE_DEV_FEATURE="$REPO_ROOT/target/release/rine"
BIN_RINE_DEV_DASH="$REPO_ROOT/target/release/rine-dev"
BIN_RINE_CONFIG="$REPO_ROOT/target/release/rine-config"
BIN_RINE32="$REPO_ROOT/target/$TARGET_32/release/rine32"

for bin in "$BIN_RINE_NODEV" "$BIN_RINE_DEV_FEATURE" "$BIN_RINE_DEV_DASH" "$BIN_RINE_CONFIG" "$BIN_RINE32"; do
    if [[ ! -x "$bin" ]]; then
        echo "error: expected binary not found: $bin" >&2
        exit 1
    fi
done

for provider_lib in "${HOST_PROVIDER_LIBS[@]}"; do
    provider_src="$REPO_ROOT/target/release/$provider_lib"
    if [[ ! -f "$provider_src" ]]; then
        echo "error: expected x64 dynamic provider not found: $provider_src" >&2
        exit 1
    fi
done

for provider_lib in "${TARGET32_PROVIDER_LIBS[@]}"; do
    provider_src="$REPO_ROOT/target/$TARGET_32/release/$provider_lib"
    if [[ ! -f "$provider_src" ]]; then
        echo "error: expected x86 dynamic provider not found: $provider_src" >&2
        exit 1
    fi
done

write_desktop_and_mime_assets() {
    local pkg_dir="$1"
    local package_name="$2"
    local include_rine_dev_bin="$3"
    local dolphin_actions="configure"

    if [[ "$include_rine_dev_bin" == "yes" ]]; then
        dolphin_actions="configure;dev"
    fi

    mkdir -p "$pkg_dir/usr/share/icons/hicolor/scalable/apps"
    mkdir -p "$pkg_dir/usr/share/icons/hicolor/256x256/apps"
    mkdir -p "$pkg_dir/usr/share/metainfo"
    mkdir -p "$pkg_dir/usr/share/kio/servicemenus"

    install -m 0644 "$BRAND_ICON_SOURCE" "$pkg_dir/usr/share/icons/hicolor/scalable/apps/rine.svg"
    install -m 0644 "$BRAND_ICON_SOURCE" "$pkg_dir/usr/share/icons/hicolor/scalable/apps/rine32.svg"
    install -m 0644 "$BRAND_ICON_SOURCE" "$pkg_dir/usr/share/icons/hicolor/scalable/apps/rine-config.svg"
    install -m 0644 "$ICON_256_RINE_SOURCE" "$pkg_dir/usr/share/icons/hicolor/256x256/apps/rine.png"
    install -m 0644 "$ICON_256_RINE_SOURCE" "$pkg_dir/usr/share/icons/hicolor/256x256/apps/rine32.png"
    install -m 0644 "$ICON_256_RINE_SOURCE" "$pkg_dir/usr/share/icons/hicolor/256x256/apps/rine-config.png"

    if [[ "$include_rine_dev_bin" == "yes" ]]; then
        install -m 0644 "$BRAND_ICON_SOURCE" "$pkg_dir/usr/share/icons/hicolor/scalable/apps/rine-dev.svg"
        install -m 0644 "$ICON_256_RINE_DEV_SOURCE" "$pkg_dir/usr/share/icons/hicolor/256x256/apps/rine-dev.png"
    fi

    cat > "$pkg_dir/usr/share/applications/rine.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=rine
Comment=Run Windows applications on Linux
Exec=rine %f
Icon=rine
Terminal=true
NoDisplay=true
MimeType=application/x-dosexec;application/x-ms-dos-executable;application/vnd.microsoft.portable-executable;
Categories=System;Emulator;
EOF

    cat > "$pkg_dir/usr/share/applications/rine32.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=rine32
Comment=Run 32-bit Windows applications on Linux
Exec=rine32 %f
Icon=rine32
Terminal=true
NoDisplay=true
Categories=System;Emulator;
EOF

    cat > "$pkg_dir/usr/share/applications/rine-config.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=rine Configuration Editor
Comment=Edit rine application settings
Exec=rine-config
Icon=rine-config
Terminal=false
Categories=Settings;Utility;
EOF

    if [[ "$include_rine_dev_bin" == "yes" ]]; then
        cat > "$pkg_dir/usr/share/applications/rine-dev.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=rine Developer Dashboard
Comment=Inspect and debug the rine runtime
Exec=rine-dev
Icon=rine-dev
Terminal=false
Categories=Development;Debugger;
EOF
    fi

    cat > "$pkg_dir/usr/share/mime/packages/rine-exe.xml" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
    <mime-type type="application/x-dosexec">
        <comment>Windows executable</comment>
        <glob pattern="*.exe"/>
    </mime-type>
    <mime-type type="application/x-ms-dos-executable">
        <comment>Windows executable</comment>
        <glob pattern="*.exe"/>
    </mime-type>
    <mime-type type="application/vnd.microsoft.portable-executable">
        <comment>Windows executable</comment>
        <glob pattern="*.exe"/>
    </mime-type>
</mime-info>
EOF

    if [[ "$package_name" == "rine" ]]; then
        cat > "$pkg_dir/usr/share/metainfo/rine.metainfo.xml" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
    <id>rine.desktop</id>
    <name>rine</name>
    <summary>Run Windows executables on Linux</summary>
    <metadata_license>CC0-1.0</metadata_license>
    <project_license>MIT</project_license>
    <launchable type="desktop-id">rine.desktop</launchable>
    <icon type="cached">rine</icon>
    <categories>
        <category>System</category>
        <category>Emulator</category>
    </categories>
    <description>
        <p>rine translates Windows NT behavior to Linux in userspace and runs Windows PE executables directly.</p>
        <p>It installs desktop integration for file associations and ships helper tools for 32-bit compatibility and configuration.</p>
    </description>
    <provides>
        <binary>rine</binary>
        <binary>rine32</binary>
        <binary>rine-config</binary>
    </provides>
    <content_rating type="oars-1.1"/>
</component>
EOF
    else
        cat > "$pkg_dir/usr/share/metainfo/rine-dev.metainfo.xml" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
    <id>rine-dev.desktop</id>
    <name>rine Developer Dashboard</name>
    <summary>Inspect and debug rine runtime state</summary>
    <metadata_license>CC0-1.0</metadata_license>
    <project_license>MIT</project_license>
    <launchable type="desktop-id">rine-dev.desktop</launchable>
    <icon type="cached">rine-dev</icon>
    <categories>
        <category>Development</category>
        <category>Debugger</category>
    </categories>
    <description>
        <p>Developer-focused dashboard for inspecting and debugging rine runtime behavior while running Windows executables on Linux.</p>
        <p>This package variant includes both the runtime and troubleshooting-oriented developer tooling.</p>
    </description>
    <provides>
        <binary>rine</binary>
        <binary>rine-dev</binary>
        <binary>rine32</binary>
        <binary>rine-config</binary>
    </provides>
    <content_rating type="oars-1.1"/>
</component>
EOF
    fi

        cat > "$pkg_dir/usr/share/kio/servicemenus/rine.desktop" <<EOF
[Desktop Entry]
Type=Service
MimeType=application/x-dosexec;application/x-ms-dos-executable;application/vnd.microsoft.portable-executable;
    Actions=${dolphin_actions}
X-KDE-Submenu=rine

[Desktop Action configure]
Name=Configure
Icon=preferences-system
Exec=/usr/bin/rine --config %f
EOF

        if [[ "$include_rine_dev_bin" == "yes" ]]; then
                cat >> "$pkg_dir/usr/share/kio/servicemenus/rine.desktop" <<'EOF'

[Desktop Action dev]
Name=Dev dashboard
Icon=utilities-terminal
Exec=/usr/bin/rine --dev %f
EOF
        fi
}

write_maintainer_scripts() {
    local debian_dir="$1"

    cat > "$debian_dir/postinst" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

if command -v update-mime-database >/dev/null 2>&1; then
    update-mime-database /usr/share/mime || true
fi

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
fi

if command -v kbuildsycoca6 >/dev/null 2>&1; then
    kbuildsycoca6 >/dev/null 2>&1 || true
elif command -v kbuildsycoca5 >/dev/null 2>&1; then
    kbuildsycoca5 >/dev/null 2>&1 || true
fi

if [[ "${1:-}" == "configure" ]] && [[ -x /usr/bin/rine ]]; then
    /usr/bin/rine --install-binfmt >/dev/null 2>&1 || true
fi
EOF

    cat > "$debian_dir/prerm" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" == "remove" || "${1:-}" == "purge" ]] && [[ -x /usr/bin/rine ]]; then
    /usr/bin/rine --uninstall-binfmt >/dev/null 2>&1 || true
fi
EOF

    cat > "$debian_dir/postrm" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

if command -v update-mime-database >/dev/null 2>&1; then
    update-mime-database /usr/share/mime || true
fi

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
fi

if command -v kbuildsycoca6 >/dev/null 2>&1; then
    kbuildsycoca6 >/dev/null 2>&1 || true
elif command -v kbuildsycoca5 >/dev/null 2>&1; then
    kbuildsycoca5 >/dev/null 2>&1 || true
fi
EOF

    chmod 0755 "$debian_dir/postinst" "$debian_dir/prerm" "$debian_dir/postrm"
}

build_package() {
    local package_name="$1"
    local rine_bin="$2"
    local include_rine_dev_bin="$3"

    local pkg_dir="$OUT_DIR/${package_name}_${DEB_VERSION}_${DEB_ARCH}"
    local debian_dir="$pkg_dir/DEBIAN"

    rm -rf "$pkg_dir"
    mkdir -p \
        "$debian_dir" \
        "$pkg_dir/usr/bin" \
        "$pkg_dir/usr/lib/rine/bin" \
        "$pkg_dir/usr/lib/rine/platform/x64" \
        "$pkg_dir/usr/lib/rine/platform/x86" \
        "$pkg_dir/usr/share/applications" \
        "$pkg_dir/usr/share/icons/hicolor/scalable/apps" \
        "$pkg_dir/usr/share/icons/hicolor/256x256/apps" \
        "$pkg_dir/usr/share/metainfo" \
        "$pkg_dir/usr/share/kio/servicemenus" \
        "$pkg_dir/usr/share/mime/packages" \
        "$pkg_dir/usr/share/doc/$package_name"

    install -m 0755 "$rine_bin" "$pkg_dir/usr/lib/rine/bin/rine"
    install -m 0755 "$BIN_RINE_CONFIG" "$pkg_dir/usr/bin/rine-config"
    install -m 0755 "$BIN_RINE32" "$pkg_dir/usr/lib/rine/bin/rine32"
    install -m 0644 "$REPO_ROOT/README.md" "$pkg_dir/usr/share/doc/$package_name/README.md"

    for provider_lib in "${HOST_PROVIDER_LIBS[@]}"; do
        install -m 0755 "$REPO_ROOT/target/release/$provider_lib" "$pkg_dir/usr/lib/rine/platform/x64/$provider_lib"
    done

    for provider_lib in "${TARGET32_PROVIDER_LIBS[@]}"; do
        install -m 0755 "$REPO_ROOT/target/$TARGET_32/release/$provider_lib" "$pkg_dir/usr/lib/rine/platform/x86/$provider_lib"
    done

    cat > "$pkg_dir/usr/bin/rine" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

export RINE_PLUGIN_DIR="/usr/lib/rine/platform/x64"
exec /usr/lib/rine/bin/rine "$@"
EOF
    chmod 0755 "$pkg_dir/usr/bin/rine"

    cat > "$pkg_dir/usr/bin/rine32" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

export RINE_PLUGIN_DIR="/usr/lib/rine/platform/x86"
exec /usr/lib/rine/bin/rine32 "$@"
EOF
    chmod 0755 "$pkg_dir/usr/bin/rine32"

    if [[ "$include_rine_dev_bin" == "yes" ]]; then
        install -m 0755 "$BIN_RINE_DEV_DASH" "$pkg_dir/usr/bin/rine-dev"
    fi

    write_desktop_and_mime_assets "$pkg_dir" "$package_name" "$include_rine_dev_bin"

    if [[ "$package_name" == "rine" ]]; then
        cat > "$debian_dir/control" <<EOF
Package: rine
Version: $DEB_VERSION
Section: utils
Priority: optional
Architecture: $DEB_ARCH
Maintainer: rine contributors <noreply@rine.dev>
Conflicts: rine-dev
Replaces: rine-dev
Depends: libc6 (>= 2.31), libstdc++6, libgtk-3-0, libglib2.0-0, libwebkit2gtk-4.1-0 | libwebkit2gtk-4.0-37, libayatana-appindicator3-1 | libappindicator3-1
Description: Windows PE executable loader for Linux
 rine runs Windows PE applications on Linux.
 .
 Includes the main runtime, the 32-bit helper runtime, and the
 configuration editor.
EOF
    else
        cat > "$debian_dir/control" <<EOF
Package: rine-dev
Version: $DEB_VERSION
Section: utils
Priority: optional
Architecture: $DEB_ARCH
Maintainer: rine contributors <noreply@rine.dev>
Conflicts: rine
Replaces: rine
Depends: libc6 (>= 2.31), libstdc++6, libgtk-3-0, libglib2.0-0, libwebkit2gtk-4.1-0 | libwebkit2gtk-4.0-37, libayatana-appindicator3-1 | libappindicator3-1
Description: Windows PE executable loader for Linux with developer dashboard
 rine runs Windows PE applications on Linux with developer mode enabled.
 .
 Includes the developer dashboard, the 32-bit helper runtime, and the
 configuration editor.
EOF
    fi

    write_maintainer_scripts "$debian_dir"

    dpkg-deb --build "$pkg_dir" >/dev/null
    echo "$pkg_dir.deb"
}

echo "==> Packaging Debian artifacts"
RINE_DEB="$(build_package "rine" "$BIN_RINE_NODEV" "no")"
RINE_DEV_DEB="$(build_package "rine-dev" "$BIN_RINE_DEV_FEATURE" "yes")"

echo
echo "Debian packages created:"
echo "  - $RINE_DEB"
echo "  - $RINE_DEV_DEB"
echo "Install one package variant with: sudo dpkg -i <package>.deb"
