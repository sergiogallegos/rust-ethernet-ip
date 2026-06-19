"""Stage the prebuilt native library into the Python package before building a wheel.

Usage:
    # build the cdylib first
    cargo build --release --features ffi
    # then stage it next to the Python sources
    python python/scripts/stage_native_lib.py [--profile release|debug]

This copies the platform's native library (``librust_ethernet_ip.so`` /
``rust_ethernet_ip.dll`` / ``librust_ethernet_ip.dylib``) from ``target/<profile>``
into ``python/rust_ethernet_ip/`` so it is picked up by ``package-data`` and the
package loader (``bindings._candidate_paths``) when the wheel is installed.
"""

from __future__ import annotations

import argparse
import shutil
import sys
from pathlib import Path


def native_file_names() -> list[str]:
    if sys.platform == "darwin":
        return ["librust_ethernet_ip.dylib"]
    if sys.platform.startswith("win"):
        return ["rust_ethernet_ip.dll"]
    return ["librust_ethernet_ip.so"]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--profile",
        choices=["release", "debug"],
        default="release",
        help="cargo build profile to source the library from (default: release)",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[2]
    target_dir = repo_root / "target" / args.profile
    package_dir = repo_root / "python" / "rust_ethernet_ip"

    staged: list[str] = []
    for name in native_file_names():
        src = target_dir / name
        if not src.exists():
            print(f"error: {src} not found. Build it first with:", file=sys.stderr)
            print("  cargo build --features ffi" + (" --release" if args.profile == "release" else ""), file=sys.stderr)
            return 1
        dst = package_dir / name
        shutil.copy2(src, dst)
        staged.append(str(dst.relative_to(repo_root)))

    print("Staged native library:")
    for path in staged:
        print(f"  {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
