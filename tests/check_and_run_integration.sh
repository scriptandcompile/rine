#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SRC_DIR="$REPO_ROOT/tests/fixtures/src"
BIN_DIR="$REPO_ROOT/tests/fixtures/bin"

STATUS_ONLY=0
AUTO_REBUILD=1
X86_MODE="dispatch"

usage() {
    cat <<'EOF'
Usage: ./tests/check_and_run_integration.sh [options]

Checks whether x64/x86 fixture binaries are up to date with tests/fixtures/src,
rebuilds stale/missing fixtures (unless disabled), then runs integration tests
for both architectures.

Options:
  --status-only   Only report fixture status; do not build or run tests
  --no-rebuild    Fail if fixtures are stale/missing instead of rebuilding
  --x86-full      Run the full x86 integration suite instead of dispatch::x86_
  -h, --help      Show this help
EOF
}

for arg in "$@"; do
    case "$arg" in
        --status-only)
            STATUS_ONLY=1
            ;;
        --no-rebuild)
            AUTO_REBUILD=0
            ;;
        --x86-full)
            X86_MODE="full"
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "error: unknown option '$arg'" >&2
            usage >&2
            exit 2
            ;;
    esac
done

check_arch_status() {
    local arch="$1"
    local out_dir="$BIN_DIR/$arch"
    local total=0
    local missing=0
    local stale=0

    while IFS= read -r -d '' src; do
        local name exe
        name="$(basename "$src" .c)"
        exe="$out_dir/${name}.exe"
        total=$((total + 1))

        if [[ ! -f "$exe" ]]; then
            missing=$((missing + 1))
            continue
        fi

        if [[ "$src" -nt "$exe" ]]; then
            stale=$((stale + 1))
        fi
    done < <(find "$SRC_DIR" -type f -name '*.c' -print0 | sort -z)

    if [[ "$missing" -eq 0 && "$stale" -eq 0 ]]; then
        echo "[$arch] up to date ($total fixtures)"
        return 0
    fi

    echo "[$arch] out of date: missing=$missing stale=$stale total=$total"
    return 1
}

check_all_status() {
    local ok=0
    check_arch_status x64 || ok=1
    check_arch_status x86 || ok=1
    return "$ok"
}

cd "$REPO_ROOT"

echo "Checking fixture status..."
if check_all_status; then
    echo "Fixture status: all up to date."
else
    echo "Fixture status: stale or missing artifacts detected."

    if [[ "$STATUS_ONLY" -eq 1 ]]; then
        exit 1
    fi

    if [[ "$AUTO_REBUILD" -eq 0 ]]; then
        echo "error: fixtures are stale/missing and --no-rebuild was set" >&2
        exit 1
    fi

    echo "Rebuilding fixtures for x64 and x86..."
    "$REPO_ROOT/tests/build_fixtures.sh" x64
    "$REPO_ROOT/tests/build_fixtures.sh" x86

    echo "Re-checking fixture status..."
    check_all_status
fi

if [[ "$STATUS_ONLY" -eq 1 ]]; then
    exit 0
fi

echo "Running integration tests with x64 fixtures..."
RINE_FIXTURE_ARCH=x64 cargo test -p rine --test integration

if [[ "$X86_MODE" == "full" ]]; then
    echo "Running full integration tests with x86 fixtures..."
    RINE_FIXTURE_ARCH=x86 cargo test -p rine --test integration
else
    echo "Running x86 dispatch integration tests (dispatch::x86_)..."
    RINE_FIXTURE_ARCH=x86 cargo test -p rine --test integration dispatch::x86_
fi

echo "Integration matrix completed successfully (x64 + x86)."
