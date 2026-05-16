#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SRC_DIR="$REPO_ROOT/tests/fixtures/src"
BIN_DIR="$REPO_ROOT/tests/fixtures/bin"
TARGET_32="i686-unknown-linux-gnu"
CC_X64="${MINGW_CC_X64:-${MINGW_CC:-x86_64-w64-mingw32-gcc}}"
CC_X86="${MINGW_CC_X86:-i686-w64-mingw32-gcc}"
WINDRES_X64="${MINGW_WINDRES_X64:-x86_64-w64-mingw32-windres}"
WINDRES_X86="${MINGW_WINDRES_X86:-i686-w64-mingw32-windres}"

assert_unique_fixture_names() {
    local duplicates
    duplicates="$({ find "$SRC_DIR" -type f -name '*.c' -print0 | xargs -0 -n1 basename | sed 's/\.c$//' | sort | uniq -d; } || true)"

    if [[ -n "$duplicates" ]]; then
        echo "error: duplicate fixture source basenames detected under $SRC_DIR" >&2
        echo "these would overwrite each other in tests/fixtures/bin:" >&2
        while IFS= read -r name; do
            [[ -n "$name" ]] && echo "  - ${name}.c" >&2
        done <<< "$duplicates"
        return 1
    fi
}

build_arch() {
    local arch="$1"
    local cc="$2"
    local windres="$3"
    local out_dir="$BIN_DIR/$arch"

    if ! command -v "$cc" &>/dev/null; then
        echo "error: $cc not found. Install mingw-w64 compiler for $arch." >&2
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
        local name exe rc_src res_obj extra_obj
        name="$(basename "$src" .c)"
        exe="$out_dir/${name}.exe"
        rc_src="${src%.c}.rc"
        extra_obj=""

        if [[ -f "$rc_src" ]]; then
            res_obj="$out_dir/${name}.res.o"
            echo "  RC  [$arch] $name.rc -> $res_obj"
            "$windres" -I"$(dirname "$rc_src")" -o "$res_obj" "$rc_src"
            extra_obj="$res_obj"
        fi

        echo "  CC  [$arch] $name.c -> $exe"
        # shellcheck disable=SC2086
        "$cc" -o "$exe" "$src" $extra_obj -I"$SRC_DIR" -O1 -static -mconsole -lgdi32 -lcomdlg32
    done < <(find "$SRC_DIR" -type f -name '*.c' -print0 | sort -z)
}

cd "$REPO_ROOT"

echo "==> Building host runtime and 32-bit helper"
"$SCRIPT_DIR/build-rine.sh"

if ! rustup target list --installed | grep -qx "$TARGET_32"; then
    echo "Installing missing Rust target: $TARGET_32"
    rustup target add "$TARGET_32"
fi

echo "==> Building 32-bit test runtime"
cargo build -p rine32 --target "$TARGET_32"

echo "==> Building Windows fixture binaries (x64 + x86)"
assert_unique_fixture_names
mkdir -p "$BIN_DIR"
build_arch x64 "$CC_X64" "$WINDRES_X64"
build_arch x86 "$CC_X86" "$WINDRES_X86"

echo "Integration prerequisites completed successfully."
echo "  fixtures: tests/fixtures/bin/{x64,x86}"
echo "  helper runtime: target/$TARGET_32/debug/rine32"
