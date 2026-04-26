#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "==> [1/10] Cargo clippy for rine"
cargo clippy -p rine --target x86_64-unknown-linux-gnu --all-targets -- -D warnings

echo "==> [2/10] Cargo clippy for rine32"
cargo clippy -p rine32 --target i686-unknown-linux-gnu --all-targets -- -D warnings

echo "==> [3/10] Building integration test fixtures (x64 + x86)"
"$SCRIPT_DIR/build-integration-prereqs.sh"

echo "==> [4/10] Running x64 unit tests"
"$SCRIPT_DIR/build-rine-unit-tests.sh"

echo "==> [5/10] Running x86 unit tests"
"$SCRIPT_DIR/build-rine32-unit-tests.sh"

echo "==> [6/10] Running integration tests"
cd "$REPO_ROOT"
cargo test --test integration -p rine

echo "==> [7/10] Enforcing attribute-only DLL metadata"
python3 "$SCRIPT_DIR/check-attribute-metadata.py"

echo "==> [8/10] Updating DLL support data"
python3 "$SCRIPT_DIR/generate-dll-support.py"

echo "==> [9/9] Staging debug and release binaries"
"$SCRIPT_DIR/build-rine.sh"

echo "==> [10/10] Building Debian package"
"$SCRIPT_DIR/build-rine-deb.sh"

echo "Full build completed successfully."
