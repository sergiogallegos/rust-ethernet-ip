from __future__ import annotations

import os
import signal
import shutil
import subprocess
import sys
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

        cargo = shutil.which("cargo")
        if cargo is None:
            raise unittest.SkipTest("cargo is not available to launch the in-repo simulator")

        repo_root = Path(__file__).resolve().parents[2]
        self._proc = subprocess.Popen(
            [cargo, "run", "--quiet", "--example", "python_test_simulator"],
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


import unittest
