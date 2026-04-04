#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_32="i686-unknown-linux-gnu"

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

case "$(dpkg --print-architecture)" in
    amd64|arm64|armhf|i386)
        DEB_ARCH="$(dpkg --print-architecture)"
        ;;
    *)
        echo "error: unsupported Debian architecture: $(dpkg --print-architecture)" >&2
        exit 1
        ;;
esac

echo "==> Building release binaries"
cargo build --release -p rine -p rine-dev -p rine-config

echo "==> Building 32-bit helper runtime"
cargo build --release -p rine32 --target "$TARGET_32"

BIN_RINE="$REPO_ROOT/target/release/rine"
BIN_RINE_DEV="$REPO_ROOT/target/release/rine-dev"
BIN_RINE_CONFIG="$REPO_ROOT/target/release/rine-config"
BIN_RINE32="$REPO_ROOT/target/$TARGET_32/release/rine32"

for bin in "$BIN_RINE" "$BIN_RINE_DEV" "$BIN_RINE_CONFIG" "$BIN_RINE32"; do
    if [[ ! -x "$bin" ]]; then
        echo "error: expected binary not found: $bin" >&2
        exit 1
    fi
done

OUT_DIR="$REPO_ROOT/target/debian"
PKG_DIR="$OUT_DIR/rine_${VERSION}_${DEB_ARCH}"
DEBIAN_DIR="$PKG_DIR/DEBIAN"

rm -rf "$PKG_DIR"
mkdir -p \
    "$DEBIAN_DIR" \
    "$PKG_DIR/usr/bin" \
    "$PKG_DIR/usr/share/applications" \
    "$PKG_DIR/usr/share/mime/packages" \
    "$PKG_DIR/usr/share/doc/rine"

install -m 0755 "$BIN_RINE" "$PKG_DIR/usr/bin/rine"
install -m 0755 "$BIN_RINE_DEV" "$PKG_DIR/usr/bin/rine-dev"
install -m 0755 "$BIN_RINE_CONFIG" "$PKG_DIR/usr/bin/rine-config"
install -m 0755 "$BIN_RINE32" "$PKG_DIR/usr/bin/rine32"
install -m 0644 "$REPO_ROOT/README.md" "$PKG_DIR/usr/share/doc/rine/README.md"

cat > "$PKG_DIR/usr/share/applications/rine.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=rine
Comment=Run Windows executables on Linux
Exec=rine %f
Terminal=true
NoDisplay=true
MimeType=application/x-dosexec;application/x-ms-dos-executable;
Categories=System;Emulator;
EOF

cat > "$PKG_DIR/usr/share/mime/packages/rine-exe.xml" <<'EOF'
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
</mime-info>
EOF

cat > "$DEBIAN_DIR/control" <<EOF
Package: rine
Version: $VERSION
Section: utils
Priority: optional
Architecture: $DEB_ARCH
Maintainer: rine contributors <noreply@rine.dev>
Depends: libc6 (>= 2.31), libstdc++6, libgtk-3-0, libglib2.0-0, libwebkit2gtk-4.1-0 | libwebkit2gtk-4.0-37, libayatana-appindicator3-1 | libappindicator3-1
Description: Windows PE executable loader for Linux
 rine translates Windows NT behavior to Linux in userspace and runs
 Windows .exe binaries directly.
 .
 This package includes all user-facing rine components:
  - rine (main CLI runtime)
  - rine32 (x86 helper runtime)
  - rine-dev (developer dashboard)
  - rine-config (configuration editor)
EOF

cat > "$DEBIAN_DIR/postinst" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

for_each_desktop_user() {
    getent passwd | while IFS=: read -r user _ uid _ _ home shell; do
        if [[ "$uid" -lt 1000 ]]; then
            continue
        fi
        if [[ ! -d "$home" ]]; then
            continue
        fi
        case "$shell" in
            */nologin|*/false)
                continue
                ;;
        esac
        printf '%s:%s\n' "$user" "$home"
    done
}

install_context_menu_for_user() {
    local user="$1"
    local home="$2"

    if command -v runuser >/dev/null 2>&1; then
        runuser -u "$user" -- env HOME="$home" XDG_DATA_HOME="$home/.local/share" \
            /usr/bin/rine --install-context-menu >/dev/null 2>&1 || true
    else
        su -s /bin/sh "$user" -c \
            "HOME='$home' XDG_DATA_HOME='$home/.local/share' /usr/bin/rine --install-context-menu" \
            >/dev/null 2>&1 || true
    fi
}

install_desktop_for_user() {
    local user="$1"
    local home="$2"

    if command -v runuser >/dev/null 2>&1; then
        runuser -u "$user" -- env HOME="$home" XDG_DATA_HOME="$home/.local/share" XDG_CONFIG_HOME="$home/.config" \
            /usr/bin/rine --install-desktop >/dev/null 2>&1 || true
    else
        su -s /bin/sh "$user" -c \
            "HOME='$home' XDG_DATA_HOME='$home/.local/share' XDG_CONFIG_HOME='$home/.config' /usr/bin/rine --install-desktop" \
            >/dev/null 2>&1 || true
    fi
}

if command -v update-mime-database >/dev/null 2>&1; then
    update-mime-database /usr/share/mime || true
fi

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications || true
fi

if [[ "${1:-}" == "configure" ]] && [[ -x /usr/bin/rine ]]; then
    /usr/bin/rine --install-binfmt >/dev/null 2>&1 || true

    while IFS=: read -r user home; do
        install_desktop_for_user "$user" "$home"
        install_context_menu_for_user "$user" "$home"
    done < <(for_each_desktop_user)
fi
EOF

cat > "$DEBIAN_DIR/prerm" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

for_each_desktop_user() {
    getent passwd | while IFS=: read -r user _ uid _ _ home shell; do
        if [[ "$uid" -lt 1000 ]]; then
            continue
        fi
        if [[ ! -d "$home" ]]; then
            continue
        fi
        case "$shell" in
            */nologin|*/false)
                continue
                ;;
        esac
        printf '%s:%s\n' "$user" "$home"
    done
}

uninstall_context_menu_for_user() {
    local user="$1"
    local home="$2"

    if command -v runuser >/dev/null 2>&1; then
        runuser -u "$user" -- env HOME="$home" XDG_DATA_HOME="$home/.local/share" \
            /usr/bin/rine --uninstall-context-menu >/dev/null 2>&1 || true
    else
        su -s /bin/sh "$user" -c \
            "HOME='$home' XDG_DATA_HOME='$home/.local/share' /usr/bin/rine --uninstall-context-menu" \
            >/dev/null 2>&1 || true
    fi
}

uninstall_desktop_for_user() {
    local user="$1"
    local home="$2"

    if command -v runuser >/dev/null 2>&1; then
        runuser -u "$user" -- env HOME="$home" XDG_DATA_HOME="$home/.local/share" XDG_CONFIG_HOME="$home/.config" \
            /usr/bin/rine --uninstall-desktop >/dev/null 2>&1 || true
    else
        su -s /bin/sh "$user" -c \
            "HOME='$home' XDG_DATA_HOME='$home/.local/share' XDG_CONFIG_HOME='$home/.config' /usr/bin/rine --uninstall-desktop" \
            >/dev/null 2>&1 || true
    fi
}

if [[ "${1:-}" == "remove" || "${1:-}" == "purge" ]] && [[ -x /usr/bin/rine ]]; then
    while IFS=: read -r user home; do
        uninstall_desktop_for_user "$user" "$home"
        uninstall_context_menu_for_user "$user" "$home"
    done < <(for_each_desktop_user)

    /usr/bin/rine --uninstall-binfmt >/dev/null 2>&1 || true
fi
EOF

cat > "$DEBIAN_DIR/postrm" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

if command -v update-mime-database >/dev/null 2>&1; then
    update-mime-database /usr/share/mime || true
fi

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications || true
fi
EOF

chmod 0755 "$DEBIAN_DIR/postinst" "$DEBIAN_DIR/prerm" "$DEBIAN_DIR/postrm"

mkdir -p "$OUT_DIR"
dpkg-deb --build "$PKG_DIR"

echo
echo "Debian package created: $PKG_DIR.deb"
echo "Install with: sudo dpkg -i $PKG_DIR.deb"
