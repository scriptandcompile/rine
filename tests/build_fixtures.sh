#!/usr/bin/env bash
# Compile all C test fixtures into Windows PE64 executables using MinGW.
# Usage: ./build_fixtures.sh
#
# Produces .exe files in tests/fixtures/bin/ from sources in tests/fixtures/src/.
# These are checked into the repo so CI doesn't need MinGW installed.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC_DIR="$SCRIPT_DIR/fixtures/src"
BIN_DIR="$SCRIPT_DIR/fixtures/bin"
CC="${MINGW_CC:-x86_64-w64-mingw32-gcc}"

if ! command -v "$CC" &>/dev/null; then
    echo "error: $CC not found. Install mingw-w64:" >&2
    echo "  apt install gcc-mingw-w64-x86-64" >&2
    exit 1
fi

mkdir -p "$BIN_DIR"

count=0
failed=0
for src in "$SRC_DIR"/*.c; do
    name="$(basename "$src" .c)"
    exe="$BIN_DIR/${name}.exe"
    echo "  CC  $name.c -> $name.exe"
    if "$CC" -o "$exe" "$src" -O1 -static -mconsole 2>&1; then
        count=$((count + 1))
    else
        echo "  FAIL: $name.c" >&2
        failed=$((failed + 1))
    fi
done

echo ""
echo "Built $count fixture(s), $failed failure(s)."
[ "$failed" -eq 0 ]
