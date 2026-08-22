from __future__ import annotations

import ctypes
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import rust_ethernet_ip.bindings as bindings
from rust_ethernet_ip.exceptions import NativeLibraryLoadError


class NativeBindingLoaderTests(unittest.TestCase):
    def test_load_native_library_reports_paths_and_symbol_errors(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            candidate = Path(tmp) / "rust_ethernet_ip.dll"
            candidate.write_bytes(b"not a real dll")

            with patch.object(bindings, "_candidate_paths", return_value=[candidate]):
                with patch.object(ctypes, "CDLL", side_effect=AttributeError("missing eip_connect")):
                    with self.assertRaises(NativeLibraryLoadError) as ctx:
                        bindings.load_native_library()

            message = str(ctx.exception)
            self.assertIn(str(candidate), message)
            self.assertIn("missing eip_connect", message)

    def test_native_file_names_match_platform_family(self) -> None:
        names = bindings._native_file_names()
        self.assertTrue(any("rust_ethernet_ip" in name for name in names))
        self.assertTrue(all(name.endswith((".dll", ".so", ".dylib")) for name in names))

    def test_source_checkout_prefers_release_native_library(self) -> None:
        with patch.dict(os.environ, {"RUST_ETHERNET_IP_NATIVE_LIB": ""}):
            candidates = bindings._candidate_paths()

        release_index = next(
            index for index, path in enumerate(candidates)
            if path.parent.name == "release" and path.parent.parent.name == "target"
        )
        debug_index = next(
            index for index, path in enumerate(candidates)
            if path.parent.name == "debug" and path.parent.parent.name == "target"
        )
        self.assertLess(release_index, debug_index)


if __name__ == "__main__":
    unittest.main()
