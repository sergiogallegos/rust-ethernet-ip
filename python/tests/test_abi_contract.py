from __future__ import annotations

import ctypes
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import rust_ethernet_ip
import rust_ethernet_ip.bindings as bindings
from rust_ethernet_ip.exceptions import NativeLibraryLoadError


class FakeFunction:
    def __init__(self, value):
        self.value = value
        self.argtypes = None
        self.restype = None

    def __call__(self, *args):
        return self.value


class FakeNativeLibrary:
    def __init__(self, *, abi_version: int = 1) -> None:
        self.eip_abi_version = FakeFunction(abi_version)
        self.eip_library_version = FakeFunction(b"1.1.0")
        self.eip_capabilities = FakeFunction(0x0F)

    def __getattr__(self, name: str) -> FakeFunction:
        function = FakeFunction(0)
        setattr(self, name, function)
        return function


class AbiContractTests(unittest.TestCase):
    def test_load_native_library_populates_abi_metadata(self) -> None:
        fake_lib = FakeNativeLibrary()
        with tempfile.TemporaryDirectory() as tmp:
            candidate = Path(tmp) / "librust_ethernet_ip.so"
            candidate.write_bytes(b"fake")

            with patch.object(bindings, "_candidate_paths", return_value=[candidate]):
                with patch.object(ctypes, "CDLL", return_value=fake_lib):
                    loaded = bindings.load_native_library()

        self.assertIs(loaded, fake_lib)
        self.assertEqual(bindings.ABI_VERSION, 1)
        self.assertEqual(bindings.LIBRARY_VERSION, "1.1.0")
        self.assertEqual(bindings.CAPABILITIES, 0x0F)
        self.assertEqual(rust_ethernet_ip.ABI_VERSION, 1)
        self.assertEqual(rust_ethernet_ip.LIBRARY_VERSION, "1.1.0")
        self.assertEqual(rust_ethernet_ip.CAPABILITIES, 0x0F)

    def test_load_native_library_rejects_abi_mismatch(self) -> None:
        fake_lib = FakeNativeLibrary(abi_version=2)
        with tempfile.TemporaryDirectory() as tmp:
            candidate = Path(tmp) / "librust_ethernet_ip.so"
            candidate.write_bytes(b"fake")

            with patch.object(bindings, "_candidate_paths", return_value=[candidate]):
                with patch.object(ctypes, "CDLL", return_value=fake_lib):
                    with self.assertRaises(NativeLibraryLoadError) as ctx:
                        bindings.load_native_library()

        self.assertIn("native library ABI version 2, wrapper expects 1", str(ctx.exception))


if __name__ == "__main__":
    unittest.main()
