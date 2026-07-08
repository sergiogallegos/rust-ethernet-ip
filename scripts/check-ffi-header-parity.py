#!/usr/bin/env python3
"""Check that include/rust_ethernet_ip.h matches the exported C ABI."""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path


RAW_POINTER_COMPAT = {
    "eip_get_udt_definition",
    "eip_get_tag_attributes",
    "eip_discover_tags_detailed",
}


def strip_comments(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    return re.sub(r"//.*", "", text)


def header_symbols(header: Path) -> set[str]:
    text = strip_comments(header.read_text(encoding="utf-8"))
    symbols = set(re.findall(r"\b(eip_[A-Za-z0-9_]+)\s*\(", text))
    return symbols


def run_tool(args: list[str]) -> str | None:
    try:
        completed = subprocess.run(
            args,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return None
    return completed.stdout


def read_u16(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 2], "little")


def read_u32(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 4], "little")


def pe_exports(library: Path) -> set[str]:
    data = library.read_bytes()
    if data[:2] != b"MZ":
        return set()
    pe_offset = read_u32(data, 0x3C)
    if data[pe_offset : pe_offset + 4] != b"PE\0\0":
        return set()

    coff = pe_offset + 4
    section_count = read_u16(data, coff + 2)
    optional_size = read_u16(data, coff + 16)
    optional = coff + 20
    magic = read_u16(data, optional)
    data_directory = optional + (112 if magic == 0x20B else 96)
    export_rva = read_u32(data, data_directory)
    if export_rva == 0:
        return set()

    sections = []
    section_table = optional + optional_size
    for index in range(section_count):
        section = section_table + index * 40
        virtual_size = read_u32(data, section + 8)
        virtual_address = read_u32(data, section + 12)
        raw_size = read_u32(data, section + 16)
        raw_pointer = read_u32(data, section + 20)
        sections.append((virtual_address, max(virtual_size, raw_size), raw_pointer))

    def rva_to_offset(rva: int) -> int:
        for virtual_address, size, raw_pointer in sections:
            if virtual_address <= rva < virtual_address + size:
                return raw_pointer + (rva - virtual_address)
        raise ValueError(f"RVA 0x{rva:x} is outside PE sections")

    export_offset = rva_to_offset(export_rva)
    name_count = read_u32(data, export_offset + 24)
    names_rva = read_u32(data, export_offset + 32)
    names_offset = rva_to_offset(names_rva)
    symbols = set()
    for index in range(name_count):
        name_rva = read_u32(data, names_offset + index * 4)
        name_offset = rva_to_offset(name_rva)
        end = data.index(b"\0", name_offset)
        name = data[name_offset:end].decode("ascii", errors="replace")
        if name.startswith("eip_"):
            symbols.add(name)
    return symbols


def exported_symbols(library: Path) -> set[str]:
    if library.suffix.lower() == ".dll":
        symbols = pe_exports(library)
        if symbols:
            return symbols

    candidates: list[list[str]] = []
    if os.name == "nt":
        dumpbin = shutil.which("dumpbin")
        if dumpbin:
            candidates.append([dumpbin, "/exports", str(library)])
    nm = shutil.which("nm")
    if nm:
        if sys.platform == "darwin":
            candidates.append([nm, "-gU", str(library)])
        else:
            candidates.append([nm, "-D", "--defined-only", str(library)])
    llvm_nm = shutil.which("llvm-nm")
    if llvm_nm:
        candidates.append([llvm_nm, "--defined-only", str(library)])

    output = None
    for command in candidates:
        output = run_tool(command)
        if output:
            break
    if not output:
        raise SystemExit(
            "could not read dynamic-library export table; install nm, llvm-nm, or dumpbin"
        )

    symbols = set()
    for match in re.findall(r"\b_?(eip_[A-Za-z0-9_]+)\b", output):
        symbols.add(match)
    return symbols


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--header", default="include/rust_ethernet_ip.h")
    parser.add_argument("--library", required=True)
    args = parser.parse_args()

    header = Path(args.header)
    library = Path(args.library)
    declared = header_symbols(header)
    exported = exported_symbols(library)

    missing_from_header = sorted(exported - declared)
    declared_but_not_exported = sorted((declared - exported) - RAW_POINTER_COMPAT)
    raw_pointer_leaks = sorted(declared & RAW_POINTER_COMPAT)

    if missing_from_header or declared_but_not_exported or raw_pointer_leaks:
        if missing_from_header:
            print("exported symbols missing from header:")
            for symbol in missing_from_header:
                print(f"  {symbol}")
        if declared_but_not_exported:
            print("header declarations not exported by library:")
            for symbol in declared_but_not_exported:
                print(f"  {symbol}")
        if raw_pointer_leaks:
            print("header must not advertise raw-pointer compatibility functions:")
            for symbol in raw_pointer_leaks:
                print(f"  {symbol}")
        return 1

    print(f"FFI header parity OK: {len(exported)} exported eip_* symbols")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
