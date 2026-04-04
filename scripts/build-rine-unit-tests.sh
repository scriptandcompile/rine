#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

"$SCRIPT_DIR/build-rine.sh"

cd "$REPO_ROOT"

echo "==> Running unit tests across workspace"
cargo test --workspace --lib --bins

echo "Unit test run completed successfully."