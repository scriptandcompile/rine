#!/usr/bin/env bash
# Compile C test fixtures into Windows PE executables for x64 and x86.
# Usage:
#   ./build_fixtures.sh           # build x64 and x86 (x86 optional if compiler missing)
#   ./build_fixtures.sh x64       # build only x64
#   ./build_fixtures.sh x86       # build only x86
#
# Produces .exe files in:
#   tests/fixtures/bin/x64/
#   tests/fixtures/bin/x86/

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC_DIR="$SCRIPT_DIR/fixtures/src"
BIN_DIR="$SCRIPT_DIR/fixtures/bin"
CC_X64="${MINGW_CC_X64:-${MINGW_CC:-x86_64-w64-mingw32-gcc}}"
CC_X86="${MINGW_CC_X86:-i686-w64-mingw32-gcc}"
ARCH="${1:-all}"

build_arch() {
    local arch="$1"
    local cc="$2"
    local out_dir="$BIN_DIR/$arch"
    local count=0
    local failed=0

    if ! command -v "$cc" &>/dev/null; then
        if [[ "$ARCH" == "all" && "$arch" == "x86" ]]; then
            echo "warning: $cc not found; skipping x86 fixtures" >&2
            return 0
        fi

        echo "error: $cc not found. Install mingw-w64 compiler for $arch:" >&2
        if [[ "$arch" == "x64" ]]; then
            echo "  apt install gcc-mingw-w64-x86-64" >&2
        else
            echo "  apt install gcc-mingw-w64-i686" >&2
        fi
        return 1
    fi

    mkdir -p "$out_dir"
    echo "Building fixtures for $arch with $cc"

    while IFS= read -r -d '' src; do
        name="$(basename "$src" .c)"
        exe="$out_dir/${name}.exe"
        echo "  CC  [$arch] $name.c -> $exe"
        if "$cc" -o "$exe" "$src" -O1 -static -mconsole -lgdi32 -lcomdlg32 2>&1; then
            count=$((count + 1))
        else
            echo "  FAIL: [$arch] $name.c" >&2
            failed=$((failed + 1))
        fi
    done < <(find "$SRC_DIR" -type f -name '*.c' -print0 | sort -z)

    echo "Built $count fixture(s) for $arch, $failed failure(s)."
    [[ "$failed" -eq 0 ]]
}

mkdir -p "$BIN_DIR"

case "$ARCH" in
    x64)
        build_arch x64 "$CC_X64"
        ;;
    x86)
        build_arch x86 "$CC_X86"
        ;;
    all)
        build_arch x64 "$CC_X64"
        build_arch x86 "$CC_X86"
        ;;
    *)
        echo "error: unknown arch '$ARCH' (expected: x64, x86, all)" >&2
        exit 2
        ;;
esac
