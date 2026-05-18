from __future__ import annotations

import os
import signal
import shutil
import subprocess
import sys
import unittest
from pathlib import Path


class SimulatorHarness:
    def __init__(self) -> None:
        self.address = os.environ.get("SIM_PLC_ADDRESS")
        self._proc: subprocess.Popen[str] | None = None

    def __enter__(self) -> str:
        if self.address:
            return self.address

        if os.environ.get("RUST_ETHERNET_IP_START_SIM") != "1":
            raise unittest.SkipTest(
                "SIM_PLC_ADDRESS is not configured and RUST_ETHERNET_IP_START_SIM is not enabled"
            )

        repo_root = Path(__file__).resolve().parents[2]
        simulator = self._resolve_simulator_binary(repo_root)
        self._proc = subprocess.Popen(
            [str(simulator)],
            cwd=repo_root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )

        assert self._proc.stdout is not None
        line = self._proc.stdout.readline().strip()
        if not line:
            stderr = ""
            if self._proc.stderr is not None:
                stderr = self._proc.stderr.read().strip()
            self._terminate()
            raise RuntimeError(f"Failed to launch simulator example. {stderr}".strip())

        self.address = line
        return self.address

    def _resolve_simulator_binary(self, repo_root: Path) -> Path:
        configured = os.environ.get("RUST_ETHERNET_IP_SIM_BIN")
        if configured:
            path = Path(configured)
            if path.exists():
                return path
            raise unittest.SkipTest(f"Configured simulator binary does not exist: {path}")

        binary_name = "python_test_simulator.exe" if sys.platform == "win32" else "python_test_simulator"
        default_path = repo_root / "target" / "debug" / "examples" / binary_name
        if default_path.exists():
            return default_path

        cargo = shutil.which("cargo")
        if cargo is None:
            raise unittest.SkipTest("cargo is not available to build the in-repo simulator")

        build = subprocess.run(
            [cargo, "build", "--quiet", "--example", "python_test_simulator"],
            cwd=repo_root,
            capture_output=True,
            text=True,
            timeout=120,
        )
        if build.returncode != 0:
            details = (build.stderr or build.stdout).strip()
            raise unittest.SkipTest(f"Could not build in-repo simulator. {details}".strip())
        if not default_path.exists():
            raise unittest.SkipTest(f"Simulator binary was not produced at {default_path}")
        return default_path

    def __exit__(self, exc_type, exc, tb) -> None:
        self._terminate()

    def _terminate(self) -> None:
        if self._proc is None:
            return

        proc = self._proc
        self._proc = None
        if proc.poll() is None:
            if sys.platform == "win32":
                proc.terminate()
            else:
                proc.send_signal(signal.SIGINT)
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=5)

        if proc.stdout is not None:
            proc.stdout.close()
        if proc.stderr is not None:
            proc.stderr.close()
