#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "==> [1/11] Cargo fmt"
cargo fmt

echo "==> [2/11] Cargo clippy for rine"
cargo clippy -p rine --target x86_64-unknown-linux-gnu --all-targets -- -D warnings

echo "==> [3/11] Cargo clippy for rine32"
cargo clippy -p rine32 --target i686-unknown-linux-gnu --all-targets -- -D warnings

echo "==> [4/11] Building integration test fixtures (x64 + x86)"
"$SCRIPT_DIR/build-integration-prereqs.sh"

echo "==> [5/11] Running x64 unit tests"
"$SCRIPT_DIR/build-rine-unit-tests.sh"

echo "==> [6/11] Running x86 unit tests"
"$SCRIPT_DIR/build-rine32-unit-tests.sh"

echo "==> [7/11] Running integration tests"
cd "$REPO_ROOT"
cargo test --test integration -p rine

echo "==> [8/11] Enforcing attribute-only DLL metadata"
python3 "$SCRIPT_DIR/check-attribute-metadata.py"

echo "==> [9/11] Updating DLL support data"
python3 "$SCRIPT_DIR/generate-dll-support.py"

echo "==> [10/11] Staging debug and release binaries"
"$SCRIPT_DIR/build-rine.sh"

echo "==> [11/11] Building Debian package"
"$SCRIPT_DIR/build-rine-deb.sh"

echo "Full build completed successfully."
