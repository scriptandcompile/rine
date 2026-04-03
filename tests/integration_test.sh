#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SRC_DIR="$REPO_ROOT/tests/fixtures/src"
BIN_DIR="$REPO_ROOT/tests/fixtures/bin"

STATUS_ONLY=0
AUTO_REBUILD=1
X86_MODE="dispatch"
ARCH="all"

assert_unique_fixture_names() {
    local duplicates
    duplicates="$({ find "$SRC_DIR" -type f -name '*.c' -print0 | xargs -0 -n1 basename | sed 's/\.c$//' | sort | uniq -d; } || true)"

    if [[ -n "$duplicates" ]]; then
        echo "error: duplicate fixture source basenames detected under $SRC_DIR" >&2
        echo "these would overwrite each other in tests/fixtures/bin:" >&2
        while IFS= read -r name; do
            [[ -n "$name" ]] && echo "  - ${name}.c" >&2
        done <<< "$duplicates"
        exit 1
    fi
}

usage() {
    cat <<'EOF'
Usage: ./tests/integration_test.sh [all|x64|x86] [options]

Checks whether x64/x86 fixture binaries are up to date with tests/fixtures/src,
rebuilds stale/missing fixtures (unless disabled), then runs integration tests
for both architectures.

Options:
    all|x64|x86   Restrict status/build/test work to specific fixture arch (default: all)
  --status-only   Only report fixture status; do not build or run tests
  --no-rebuild    Fail if fixtures are stale/missing instead of rebuilding
  --x86-full      Run the full x86 integration suite instead of dispatch::x86_
  -h, --help      Show this help
EOF
}

for arg in "$@"; do
    case "$arg" in
                all|x64|x86)
                        ARCH="$arg"
                        ;;
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

check_selected_status() {
    local ok=0

    case "$ARCH" in
        x64)
            check_arch_status x64 || ok=1
            ;;
        x86)
            check_arch_status x86 || ok=1
            ;;
        all)
            check_arch_status x64 || ok=1
            check_arch_status x86 || ok=1
            ;;
        *)
            echo "error: unknown arch '$ARCH' (expected: x64, x86, all)" >&2
            return 2
            ;;
    esac

    return "$ok"
}

run_selected_tests() {
    case "$ARCH" in
        x64)
            echo "Running integration tests with x64 fixtures..."
            RINE_FIXTURE_ARCH=x64 cargo test -p rine --test integration
            ;;
        x86)
            if [[ "$X86_MODE" == "full" ]]; then
                echo "Running full integration tests with x86 fixtures..."
                RINE_FIXTURE_ARCH=x86 cargo test -p rine --test integration
            else
                echo "Running x86 dispatch integration tests (dispatch::x86_)..."
                RINE_FIXTURE_ARCH=x86 cargo test -p rine --test integration dispatch::x86_
            fi
            ;;
        all)
            echo "Running integration tests with x64 fixtures..."
            RINE_FIXTURE_ARCH=x64 cargo test -p rine --test integration

            if [[ "$X86_MODE" == "full" ]]; then
                echo "Running full integration tests with x86 fixtures..."
                RINE_FIXTURE_ARCH=x86 cargo test -p rine --test integration
            else
                echo "Running x86 dispatch integration tests (dispatch::x86_)..."
                RINE_FIXTURE_ARCH=x86 cargo test -p rine --test integration dispatch::x86_
            fi
            ;;
        *)
            echo "error: unknown arch '$ARCH' (expected: x64, x86, all)" >&2
            return 2
            ;;
    esac
}

cd "$REPO_ROOT"
assert_unique_fixture_names

echo "Checking fixture status..."
if check_selected_status; then
    echo "Fixture status: selected fixtures are up to date."
else
    echo "Fixture status: stale or missing artifacts detected."

    if [[ "$STATUS_ONLY" -eq 1 ]]; then
        exit 1
    fi

    if [[ "$AUTO_REBUILD" -eq 0 ]]; then
        echo "error: fixtures are stale/missing and --no-rebuild was set" >&2
        exit 1
    fi

    echo "Rebuilding fixtures for arch='$ARCH'..."
    "$REPO_ROOT/tests/build_fixtures.sh" "$ARCH"

    echo "Re-checking fixture status..."
    check_selected_status
fi

if [[ "$STATUS_ONLY" -eq 1 ]]; then
    exit 0
fi

run_selected_tests

echo "Integration run completed successfully (arch=$ARCH)."
