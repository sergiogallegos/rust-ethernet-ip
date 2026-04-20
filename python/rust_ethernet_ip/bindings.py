from __future__ import annotations

import ctypes
import os
import sys
from pathlib import Path

from .exceptions import NativeLibraryLoadError


RESULT_BUFFER_SIZE = 131072
READ_BUFFER_SIZE = 65536


def _native_file_names() -> list[str]:
    if sys.platform == "darwin":
        return ["librust_ethernet_ip.dylib"]
    if os.name == "nt":
        return ["rust_ethernet_ip.dll"]
    return ["librust_ethernet_ip.so"]


def _candidate_paths() -> list[Path]:
    env_path = os.environ.get("RUST_ETHERNET_IP_NATIVE_LIB")
    candidates: list[Path] = []
    if env_path:
        candidates.append(Path(env_path))

    repo_root = Path(__file__).resolve().parents[2]
    names = _native_file_names()
    for name in names:
        candidates.extend(
            [
                repo_root / "target" / "debug" / name,
                repo_root / "target" / "release" / name,
                repo_root / "csharp" / "RustEtherNetIp" / "bin" / "Release" / "net10.0" / name,
                repo_root / "csharp" / "RustEtherNetIp" / name,
            ]
        )

    return candidates


def _configure_function_signatures(lib: ctypes.CDLL) -> ctypes.CDLL:
    c_char_p_p = ctypes.POINTER(ctypes.c_char_p)

    lib.eip_connect.argtypes = [ctypes.c_char_p]
    lib.eip_connect.restype = ctypes.c_int

    lib.eip_disconnect.argtypes = [ctypes.c_int]
    lib.eip_disconnect.restype = ctypes.c_int

    lib.eip_read_tag.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_int]
    lib.eip_read_tag.restype = ctypes.c_int

    lib.eip_read_tags_batch.argtypes = [ctypes.c_int, c_char_p_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_int]
    lib.eip_read_tags_batch.restype = ctypes.c_int

    lib.eip_write_tags_batch.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_int]
    lib.eip_write_tags_batch.restype = ctypes.c_int

    lib.eip_check_health.argtypes = [ctypes.c_int, ctypes.POINTER(ctypes.c_int)]
    lib.eip_check_health.restype = ctypes.c_int

    lib.eip_get_diagnostics_json.argtypes = [
        ctypes.c_int,
        ctypes.c_int,
        ctypes.POINTER(ctypes.c_void_p),
    ]
    lib.eip_get_diagnostics_json.restype = ctypes.c_int

    lib.eip_free_string.argtypes = [ctypes.c_void_p]
    lib.eip_free_string.restype = None

    return lib


def load_native_library() -> ctypes.CDLL:
    errors: list[str] = []
    for path in _candidate_paths():
        if not path.exists():
            continue
        try:
            return _configure_function_signatures(ctypes.CDLL(str(path)))
        except OSError as exc:
            errors.append(f"{path}: {exc}")

    tried = "\n".join(str(path) for path in _candidate_paths())
    details = "\n".join(errors)
    raise NativeLibraryLoadError(
        "Could not load rust_ethernet_ip native library.\n"
        f"Tried:\n{tried}\n"
        f"{details}"
    )
