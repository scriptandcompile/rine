#!/usr/bin/env python3

from __future__ import annotations

import re
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
PLUGIN_ROOTS = [
    REPO_ROOT / "crates" / "platform" / "win32-dll",
    REPO_ROOT / "crates" / "platform" / "win64-dll",
]

FORBIDDEN_METADATA_PATTERNS = [
    (re.compile(r"\bExport::Func\s*\("), "manual Export::Func metadata"),
    (re.compile(r"\bExport::Data\s*\("), "manual Export::Data metadata"),
    (re.compile(r"\bExport::Ordinal\s*\("), "manual Export::Ordinal metadata"),
    (re.compile(r"\bStubExport\s*\{"), "manual StubExport metadata"),
    (re.compile(r"\bPartialExport\s*\{"), "manual PartialExport metadata"),
]

REQUIRED_METHOD_INCLUDES = {
    "exports": "dll_plugin_generated.rs",
    "stubs": "dll_plugin_generated_stubs.rs",
    "partials": "dll_plugin_generated_partials.rs",
}

def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def list_plugin_crates() -> list[Path]:
    crates: list[Path] = []
    for root in PLUGIN_ROOTS:
        if not root.exists():
            continue
        for entry in sorted(root.iterdir()):
            if not entry.is_dir():
                continue
            if (entry / "src" / "lib.rs").exists():
                crates.append(entry)
    return crates


def extract_method_body(source: str, method_name: str) -> str | None:
    signature = re.search(
        rf"fn\s+{re.escape(method_name)}\s*\(\s*&self\s*\)\s*->\s*Vec<[^>]+>\s*\{{",
        source,
    )
    if not signature:
        return None

    start = signature.end()
    depth = 1
    for i in range(start, len(source)):
        ch = source[i]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return source[start:i]
    return None


def has_required_include(body: str, generated_file: str) -> bool:
    include_pattern = re.compile(
        rf"include!\s*\(\s*concat!\s*\(\s*env!\(\s*\"OUT_DIR\"\s*\)\s*,\s*\"/{re.escape(generated_file)}\"\s*\)\s*\)",
        re.DOTALL,
    )
    return include_pattern.search(body) is not None


def validate_plugin(crate_dir: Path) -> list[str]:
    errors: list[str] = []
    lib_path = crate_dir / "src" / "lib.rs"
    lib_source = read_text(lib_path)

    for pattern, label in FORBIDDEN_METADATA_PATTERNS:
        match = pattern.search(lib_source)
        if match:
            line = lib_source.count("\n", 0, match.start()) + 1
            errors.append(f"{lib_path.relative_to(REPO_ROOT)}:{line}: {label} is forbidden")

    for method_name, generated_file in REQUIRED_METHOD_INCLUDES.items():
        body = extract_method_body(lib_source, method_name)
        if body is None:
            errors.append(
                f"{lib_path.relative_to(REPO_ROOT)}: missing fn {method_name}(&self) metadata method"
            )
            continue
        if not has_required_include(body, generated_file):
            errors.append(
                f"{lib_path.relative_to(REPO_ROOT)}: fn {method_name} must include {generated_file} from OUT_DIR"
            )

    build_rs = crate_dir / "build.rs"
    if not build_rs.exists():
        errors.append(f"{build_rs.relative_to(REPO_ROOT)}: missing build.rs")
    else:
        build_source = read_text(build_rs)
        if "generate_metadata_code(" not in build_source:
            errors.append(
                f"{build_rs.relative_to(REPO_ROOT)}: build script must call rine_dll_build::generate_metadata_code()"
            )

    return errors


def main() -> int:
    crates = list_plugin_crates()
    if not crates:
        print("No DLL plugin crates found.")
        return 0

    failures: list[str] = []
    for crate_dir in crates:
        failures.extend(validate_plugin(crate_dir))

    if failures:
        print("Attribute metadata regression check failed.")
        print("All DLL plugin metadata must be generated from attributes only.")
        print()
        for failure in failures:
            print(f"- {failure}")
        return 1

    print("Attribute metadata regression check passed.")
    print(f"Checked {len(crates)} DLL plugin crates.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
