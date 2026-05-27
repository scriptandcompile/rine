#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
EXAMPLES_DIR="$REPO_ROOT/tests/fixtures/examples"
OUT_DIR="$EXAMPLES_DIR/bin"

CC_X64="${MINGW_CC_X64:-${MINGW_CC:-x86_64-w64-mingw32-gcc}}"
WINDRES_X64="${MINGW_WINDRES_X64:-x86_64-w64-mingw32-windres}"

require_tool() {
    local tool="$1"
    local install_hint="$2"

    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: required tool not found: $tool" >&2
        [[ -n "$install_hint" ]] && echo "hint: $install_hint" >&2
        exit 1
    fi
}

build_basic_window_example() {
    local src_dir="$EXAMPLES_DIR/basic_window_c_app"
    local out_exe="$OUT_DIR/basic_window_c_app.exe"

    echo "  CC  basic_window_c_app -> $out_exe"
    "$CC_X64" -municode -O1 -o "$out_exe" "$src_dir/main.c" -lgdi32 -lcomctl32
}

build_modal_dialog_example() {
    local src_dir="$EXAMPLES_DIR/modal_dialog_c_app"
    local res_obj="$OUT_DIR/modal_dialog_c_app.res.o"
    local out_exe="$OUT_DIR/modal_dialog_c_app.exe"

    echo "  RC  modal_dialog_c_app/dialog.rc -> $res_obj"
    "$WINDRES_X64" -I"$src_dir" -o "$res_obj" "$src_dir/dialog.rc"

    echo "  CC  modal_dialog_c_app -> $out_exe"
    "$CC_X64" -municode -O1 -o "$out_exe" "$src_dir/main.c" "$res_obj" -luser32
}

echo "==> Building fixture examples (x64)"
require_tool "$CC_X64" "apt install gcc-mingw-w64-x86-64"
require_tool "$WINDRES_X64" "apt install binutils-mingw-w64-x86-64"

mkdir -p "$OUT_DIR"
build_basic_window_example
build_modal_dialog_example

echo "Build completed successfully."
echo "  outputs: $OUT_DIR/basic_window_c_app.exe"
echo "           $OUT_DIR/modal_dialog_c_app.exe"
