#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_32="i686-unknown-linux-gnu"
HOST_DEBUG_DIR="$REPO_ROOT/target/debug"
HOST_RELEASE_DIR="$REPO_ROOT/target/release"

HOST_PROVIDER_LIBS=(
    "librine64_kernel32.so"
    "librine64_msvcrt.so"
    "librine64_ntdll.so"
    "librine64_advapi32.so"
    "librine64_gdi32.so"
    "librine64_comdlg32.so"
    "librine64_shell32.so"
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
    "rine32-shell32"
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
    "librine32_shell32.so"
    "librine32_user32.so"
    "librine32_ws2_32.so"
)

cd "$REPO_ROOT"

if ! command -v rustup >/dev/null 2>&1; then
    echo "error: rustup not found; cannot manage Rust targets automatically" >&2
    echo "hint: install rustup, then run: rustup target add $TARGET_32" >&2
    exit 1
fi

if ! rustup target list --installed | grep -qx "$TARGET_32"; then
    echo "Installing missing Rust target: $TARGET_32"
    rustup target add "$TARGET_32"
fi

echo "==> Building default host workspace members (debug)"
cargo build

echo "==> Building default host workspace members (release)"
cargo build --release

echo "==> Building 32-bit runtime (debug)"
cargo build -p rine32 --target "$TARGET_32"

echo "==> Building 32-bit dynamic providers (debug)"
for package in "${TARGET32_PROVIDER_PACKAGES[@]}"; do
    cargo build --target "$TARGET_32" -p "$package"
done

echo "==> Building 32-bit runtime (release)"
cargo build --release -p rine32 --target "$TARGET_32"

echo "==> Building 32-bit dynamic providers (release)"
for package in "${TARGET32_PROVIDER_PACKAGES[@]}"; do
    cargo build --release --target "$TARGET_32" -p "$package"
done

for profile in debug release; do
    src="$REPO_ROOT/target/$TARGET_32/$profile/rine32"
    dst="$REPO_ROOT/target/$profile/rine32"
    if [[ ! -x "$src" ]]; then
        echo "error: expected 32-bit runtime not found: $src" >&2
        exit 1
    fi
    echo "==> Staging $profile rine32 next to $profile rine"
    install -m 0755 "$src" "$dst"

    for provider_lib in "${HOST_PROVIDER_LIBS[@]}"; do
        provider_src="$REPO_ROOT/target/$profile/$provider_lib"
        if [[ ! -f "$provider_src" ]]; then
            echo "error: expected dynamic provider not found: $provider_src" >&2
            exit 1
        fi
        echo "==> Verified $profile dynamic provider: $provider_src"
    done

    for provider_lib in "${TARGET32_PROVIDER_LIBS[@]}"; do
        provider32_src="$REPO_ROOT/target/$TARGET_32/$profile/$provider_lib"
        provider32_dst="$REPO_ROOT/target/$profile/$provider_lib"
        if [[ ! -f "$provider32_src" ]]; then
            echo "error: expected 32-bit dynamic provider not found: $provider32_src" >&2
            exit 1
        fi
        echo "==> Staging $profile 32-bit provider next to rine32: $provider_lib"
        install -m 0755 "$provider32_src" "$provider32_dst"
    done
done

echo "Build complete."
echo "  debug:   target/debug/rine + target/debug/rine32 + dynamic provider .so files"
echo "  release: target/release/rine + target/release/rine32 + dynamic provider .so files"