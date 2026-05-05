#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_32="i686-unknown-linux-gnu"

"$SCRIPT_DIR/build-rine.sh"

cd "$REPO_ROOT"

echo "==> Running x86 unit tests"
cargo test \
    --target "$TARGET_32" \
    --lib --bins \
    -p rine32 \
    -p rine32-advapi32 \
    -p rine32-kernel32 \
    -p rine32-comdlg32 \
    -p rine32-shell32 \
    -p rine32-gdi32 \
    -p rine32-msvcrt \
    -p rine32-ntdll \
    -p rine32-user32

echo "x86 unit test run completed successfully."
