"""Verify the project version is consistent across every manifest.

Compares the base semver (MAJOR.MINOR.PATCH) across:
  - VERSION
  - Cargo.toml ([package].version)
  - python/pyproject.toml ([project].version)
  - csharp/RustEtherNetIp/RustEtherNetIp.csproj (<Version>, <AssemblyVersion>, <FileVersion>)

PEP 440 pre-release/dev suffixes on the python version (`.dev0`, `.rc1`, etc.)
and the 4th .NET assembly-version component are stripped before comparison so
those manifests can carry release-channel metadata without breaking the check.
"""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]


def base_version(raw: str) -> str:
    """Strip PEP 440 suffixes and the 4th .NET assembly component."""
    trimmed = re.split(r"[+-]", raw, maxsplit=1)[0]
    trimmed = re.sub(r"\.(dev|a|b|rc|post)\d+", "", trimmed)
    parts = trimmed.split(".")
    return ".".join(parts[:3])


def collect_sources() -> dict[str, str]:
    sources: dict[str, str] = {}

    sources["VERSION"] = (REPO_ROOT / "VERSION").read_text(encoding="utf-8").strip()

    cargo = tomllib.loads((REPO_ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    sources["Cargo.toml [package].version"] = cargo["package"]["version"]

    pyproject = tomllib.loads(
        (REPO_ROOT / "python" / "pyproject.toml").read_text(encoding="utf-8")
    )
    sources["python/pyproject.toml"] = pyproject["project"]["version"]

    csproj_text = (
        REPO_ROOT / "csharp" / "RustEtherNetIp" / "RustEtherNetIp.csproj"
    ).read_text(encoding="utf-8")
    for tag in ("Version", "AssemblyVersion", "FileVersion"):
        match = re.search(rf"<{tag}>([^<]+)</{tag}>", csproj_text)
        if match:
            sources[f"csproj <{tag}>"] = match.group(1)

    return sources


def main() -> int:
    sources = collect_sources()
    bases = {label: base_version(v) for label, v in sources.items()}

    print("Detected versions:")
    width = max(len(k) for k in sources)
    for label, raw in sources.items():
        print(f"  {label:<{width}}  {raw!r:<22} base={bases[label]}")

    unique = set(bases.values())
    if len(unique) != 1:
        print()
        print("ERROR: base versions do not match across manifests.")
        return 1

    print()
    print(f"OK: all base versions agree on {unique.pop()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
