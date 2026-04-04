#!/usr/bin/env python3

from __future__ import annotations

import json
import re
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
DOCS_DATA_PATH = REPO_ROOT / "docs" / "data" / "dll-support.json"

ARCH_ROOTS = [
    ("x64", REPO_ROOT / "crates" / "platform" / "win64-dll"),
    ("x86", REPO_ROOT / "crates" / "platform" / "win32-dll"),
]


@dataclass
class ExportRow:
    dll: str
    function: str
    arch: str
    status: str
    symbol: str
    source: str | None


def read_text(file_path: Path) -> str:
    return file_path.read_text(encoding="utf-8")


def list_crate_dirs(root_dir: Path) -> list[Path]:
    if not root_dir.exists():
        return []

    dirs: list[Path] = []
    for entry in root_dir.iterdir():
        if entry.is_dir() and (entry / "src" / "lib.rs").exists():
            dirs.append(entry)
    return dirs


def parse_dll_name(lib_source: str) -> str:
    match = re.search(r'&\[\s*"([^"]+\.dll)"', lib_source, re.IGNORECASE)
    return match.group(1).lower() if match else "unknown.dll"


def parse_win32_stub_names(lib_source: str) -> set[str]:
    return set(re.findall(r"win32_stub!\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*,", lib_source))


def parse_exports(lib_source: str) -> list[tuple[str, str]]:
    pattern = re.compile(
        r'Export::Func\(\s*"([^"]+)"\s*,\s*as_win_api!\(([^)]+)\)\s*,?\s*\)',
        re.MULTILINE | re.DOTALL,
    )
    return [(m.group(1).strip(), m.group(2).strip()) for m in pattern.finditer(lib_source)]


def find_symbol_source_file(crate_src_dir: Path, symbol_path: str) -> Path | None:
    if "::" in symbol_path:
        parts = symbol_path.split("::")
        module_path = "/".join(parts[:-1])
        module_file = crate_src_dir / f"{module_path}.rs"
        if module_file.exists():
            return module_file

    for file_path in crate_src_dir.rglob("*.rs"):
        text = read_text(file_path)
        if f"fn {symbol_path}" in text:
            return file_path

    return None


def find_function_body(file_source: str, symbol_name: str) -> str | None:
    escaped = re.escape(symbol_name)
    signature_re = re.compile(
        rf'(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+{escaped}\b',
        re.MULTILINE,
    )

    match = signature_re.search(file_source)
    if not match:
        return None

    start = match.start()
    open_brace = file_source.find("{", start)
    if open_brace == -1:
        return None

    depth = 0
    for index in range(open_brace, len(file_source)):
        ch = file_source[index]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return file_source[open_brace : index + 1]

    return None


def infer_status(
    *,
    arch: str,
    export_name: str,
    symbol_path: str,
    source_text: str | None,
    stub_names: set[str],
) -> str:
    if arch == "x86" and export_name in stub_names:
        return "stubbed"

    symbol_name = symbol_path.split("::")[-1]
    body = find_function_body(source_text, symbol_name) if source_text else None
    if not body:
        return "unimplemented"

    if re.search(r"\b(todo!|unimplemented!)\s*\(", body):
        return "unimplemented"

    if re.search(r"\bstub\b", body, re.IGNORECASE) or re.search(
        r"\bnot\s+implemented\b", body, re.IGNORECASE
    ):
        return "stubbed"

    return "implemented"


def to_source_label(dll_name: str, crate_src_dir: Path, abs_path: Path | None) -> str | None:
    if abs_path is None:
        return None

    dll_base = dll_name.lower().removesuffix(".dll")
    relative_source = abs_path.relative_to(crate_src_dir).as_posix()
    return f"{dll_base} - {relative_source}"


def collect_arch_data(arch: str, arch_root: Path) -> list[ExportRow]:
    rows: list[ExportRow] = []

    for crate_dir in list_crate_dirs(arch_root):
        src_dir = crate_dir / "src"
        lib_path = src_dir / "lib.rs"
        lib_source = read_text(lib_path)
        dll_name = parse_dll_name(lib_source)
        exports = parse_exports(lib_source)
        stub_names = parse_win32_stub_names(lib_source)

        for export_name, symbol_path in exports:
            source_file = find_symbol_source_file(src_dir, symbol_path)
            source_text = read_text(source_file) if source_file else None
            status = infer_status(
                arch=arch,
                export_name=export_name,
                symbol_path=symbol_path,
                source_text=source_text,
                stub_names=stub_names,
            )

            rows.append(
                ExportRow(
                    dll=dll_name,
                    function=export_name,
                    arch=arch,
                    status=status,
                    symbol=symbol_path,
                    source=to_source_label(dll_name, src_dir, source_file),
                )
            )

    return rows


def build_dataset() -> dict:
    combined: list[ExportRow] = []
    for arch, arch_root in ARCH_ROOTS:
        combined.extend(collect_arch_data(arch, arch_root))

    dll_map: dict[str, dict[str, dict]] = {}
    for row in combined:
        dll_map.setdefault(row.dll, {})
        dll_map[row.dll].setdefault(row.function, {"name": row.function, "x64": None, "x86": None})
        dll_map[row.dll][row.function][row.arch] = {
            "status": row.status,
            "symbol": row.symbol,
            "source": row.source,
        }

    dlls = []
    for dll_name in sorted(dll_map.keys()):
        functions = []
        for function_name in sorted(dll_map[dll_name].keys()):
            fn = dll_map[dll_name][function_name]
            functions.append(
                {
                    "name": fn["name"],
                    "x64": fn["x64"]
                    or {"status": "unimplemented", "symbol": None, "source": None},
                    "x86": fn["x86"]
                    or {"status": "unimplemented", "symbol": None, "source": None},
                }
            )

        dlls.append({"name": dll_name, "functionCount": len(functions), "functions": functions})

    totals = {
        "functions": 0,
        "x64": {"implemented": 0, "stubbed": 0, "unimplemented": 0},
        "x86": {"implemented": 0, "stubbed": 0, "unimplemented": 0},
    }

    for dll in dlls:
        totals["functions"] += dll["functionCount"]
        for fn in dll["functions"]:
            totals["x64"][fn["x64"]["status"]] += 1
            totals["x86"][fn["x86"]["status"]] += 1

    return {
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/generate-dll-support.py",
        "statusOrder": ["implemented", "stubbed", "unimplemented"],
        "dlls": dlls,
        "totals": totals,
    }


def main() -> None:
    data = build_dataset()
    DOCS_DATA_PATH.parent.mkdir(parents=True, exist_ok=True)
    DOCS_DATA_PATH.write_text(f"{json.dumps(data, indent=2)}\n", encoding="utf-8")

    print(f"Wrote {DOCS_DATA_PATH}")
    print(f"DLLs: {len(data['dlls'])}, functions: {data['totals']['functions']}")
    print(
        "x64 -> implemented "
        f"{data['totals']['x64']['implemented']}, "
        f"stubbed {data['totals']['x64']['stubbed']}, "
        f"unimplemented {data['totals']['x64']['unimplemented']}"
    )
    print(
        "x86 -> implemented "
        f"{data['totals']['x86']['implemented']}, "
        f"stubbed {data['totals']['x86']['stubbed']}, "
        f"unimplemented {data['totals']['x86']['unimplemented']}"
    )


if __name__ == "__main__":
    main()
