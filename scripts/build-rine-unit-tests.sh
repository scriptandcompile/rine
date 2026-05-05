#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

"$SCRIPT_DIR/build-rine.sh"

cd "$REPO_ROOT"

echo "==> Running unit tests across workspace"
cargo test --workspace --lib --bins \
	--exclude rine32 \
	--exclude rine32-advapi32 \
	--exclude rine32-kernel32 \
	--exclude rine32-comdlg32 \
	--exclude rine32-shell32 \
	--exclude rine32-gdi32 \
	--exclude rine32-msvcrt \
	--exclude rine32-ntdll \
	--exclude rine32-user32

echo "Unit test run completed successfully."