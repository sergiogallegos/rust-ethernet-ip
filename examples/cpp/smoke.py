#!/usr/bin/env python3
from __future__ import annotations

import os
import subprocess
import sys
import time
from pathlib import Path


def read_address(process: subprocess.Popen[str]) -> str:
    deadline = time.time() + 60
    while time.time() < deadline:
        line = process.stdout.readline() if process.stdout else ""
        if not line:
            break
        marker = "PLC simulator listening on "
        if marker in line:
            return line.split(marker, 1)[1].strip()
    stderr = process.stderr.read() if process.stderr else ""
    raise RuntimeError(f"plc_sim did not report a listening address. stderr: {stderr}")


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: smoke.py PATH_TO_CPP_SMOKE_DEMO", file=sys.stderr)
        return 2

    demo = Path(sys.argv[1]).resolve()
    repo_root = Path(os.environ.get("RUST_ETHERNET_IP_REPO_ROOT", Path.cwd())).resolve()

    sim = subprocess.Popen(
        ["cargo", "run", "--quiet", "--bin", "plc_sim"],
        cwd=repo_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        address = read_address(sim)
        env = os.environ.copy()
        lib_dir = str(demo.parent)
        if sys.platform == "win32":
            env["PATH"] = lib_dir + os.pathsep + env.get("PATH", "")
        elif sys.platform == "darwin":
            env["DYLD_LIBRARY_PATH"] = lib_dir + os.pathsep + env.get("DYLD_LIBRARY_PATH", "")
        else:
            env["LD_LIBRARY_PATH"] = lib_dir + os.pathsep + env.get("LD_LIBRARY_PATH", "")
        subprocess.run([str(demo), address], check=True, env=env)
    finally:
        sim.terminate()
        try:
            sim.wait(timeout=5)
        except subprocess.TimeoutExpired:
            sim.kill()
            sim.wait(timeout=5)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
