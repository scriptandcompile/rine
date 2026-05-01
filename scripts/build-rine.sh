#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_32="i686-unknown-linux-gnu"
HOST_DEBUG_DIR="$REPO_ROOT/target/debug"
HOST_RELEASE_DIR="$REPO_ROOT/target/release"
NTDLL_PROVIDER_LIB="librine64_ntdll.so"

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

echo "==> Building 32-bit runtime (release)"
cargo build --release -p rine32 --target "$TARGET_32"

for profile in debug release; do
    src="$REPO_ROOT/target/$TARGET_32/$profile/rine32"
    dst="$REPO_ROOT/target/$profile/rine32"
    if [[ ! -x "$src" ]]; then
        echo "error: expected 32-bit runtime not found: $src" >&2
        exit 1
    fi
    echo "==> Staging $profile rine32 next to $profile rine"
    install -m 0755 "$src" "$dst"

    provider_src="$REPO_ROOT/target/$profile/$NTDLL_PROVIDER_LIB"
    if [[ ! -f "$provider_src" ]]; then
        echo "error: expected dynamic provider not found: $provider_src" >&2
        exit 1
    fi
    echo "==> Verified $profile dynamic provider: $provider_src"
done

echo "Build complete."
echo "  debug:   target/debug/rine  +  target/debug/rine32  +  target/debug/$NTDLL_PROVIDER_LIB"
echo "  release: target/release/rine  +  target/release/rine32  +  target/release/$NTDLL_PROVIDER_LIB"