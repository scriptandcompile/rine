#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_32="i686-unknown-linux-gnu"
HOST_DIR="$REPO_ROOT/target/debug"
HOST_RINE="$HOST_DIR/rine"
TARGET_RINE32="$REPO_ROOT/target/$TARGET_32/debug/rine32"
HOST_RINE32="$HOST_DIR/rine32"

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

echo "==> Building default host workspace members"
cargo build

echo "==> Building 32-bit runtime (rine32 + win32 DLL crates)"
cargo build -p rine32 --target "$TARGET_32"

if [[ ! -x "$TARGET_RINE32" ]]; then
    echo "error: expected 32-bit runtime not found: $TARGET_RINE32" >&2
    exit 1
fi

if [[ ! -x "$HOST_RINE" ]]; then
    echo "error: expected host runtime not found: $HOST_RINE" >&2
    exit 1
fi

echo "==> Staging 32-bit runtime next to host rine"
install -m 0755 "$TARGET_RINE32" "$HOST_RINE32"

echo "Build complete."
echo "  host: target/debug/"
echo "  32-bit: target/$TARGET_32/debug/rine32"
echo "  staged helper: target/debug/rine32"