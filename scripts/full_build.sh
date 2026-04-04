#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "==> [1/6] Building integration test fixtures (x64 + x86)"
"$SCRIPT_DIR/build-integration-prereqs.sh"

echo "==> [2/6] Running x64 unit tests"
"$SCRIPT_DIR/build-rine-unit-tests.sh"

echo "==> [3/6] Running x86 unit tests"
"$SCRIPT_DIR/build-rine32-unit-tests.sh"

echo "==> [4/6] Running integration tests"
cd "$REPO_ROOT"
cargo test --test integration -p rine

echo "==> [5/6] Updating DLL support data"
python3 "$SCRIPT_DIR/generate-dll-support.py"

echo "==> [6/6] Building Debian package"
"$SCRIPT_DIR/build-rine-deb.sh"

echo "Full build completed successfully."
