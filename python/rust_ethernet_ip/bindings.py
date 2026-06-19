from __future__ import annotations

import ctypes
import os
import sys
from pathlib import Path

from .exceptions import NativeLibraryLoadError


RESULT_BUFFER_SIZE = 131072
READ_BUFFER_SIZE = 65536
EXPECTED_ABI_VERSION = 1
CAP_ROUTE_PATH_ORDERED_HOPS = 0x0000_0000_0000_0001
CAP_BATCH_EXECUTE_V1 = 0x0000_0000_0000_0002
CAP_DIAGNOSTICS_JSON = 0x0000_0000_0000_0004
CAP_TAG_GROUP_SUBSCRIPTIONS = 0x0000_0000_0000_0008

ABI_VERSION: int | None = None
LIBRARY_VERSION: str | None = None
CAPABILITIES: int | None = None


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

    names = _native_file_names()

    # 1) Bundled inside the installed package (the pip/wheel install path). This
    #    must come before the repo-relative locations so an installed wheel works
    #    without a source checkout.
    package_dir = Path(__file__).resolve().parent
    for name in names:
        candidates.append(package_dir / name)
        candidates.append(package_dir / "_native" / name)

    # 2) Repo-relative locations, for running against a local source build.
    repo_root = Path(__file__).resolve().parents[2]
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
    c_uint8_p = ctypes.POINTER(ctypes.c_uint8)

    lib.eip_connect.argtypes = [ctypes.c_char_p]
    lib.eip_connect.restype = ctypes.c_int

    lib.eip_connect_with_route.argtypes = [
        ctypes.c_char_p,
        c_uint8_p,
        ctypes.c_int,
        c_uint8_p,
        ctypes.c_int,
        c_char_p_p,
        ctypes.c_int,
    ]
    lib.eip_connect_with_route.restype = ctypes.c_int
    lib.eip_connect_with_route_hops.argtypes = [
        ctypes.c_char_p,
        c_uint8_p,
        c_uint8_p,
        c_uint8_p,
        c_char_p_p,
        ctypes.c_int,
    ]
    lib.eip_connect_with_route_hops.restype = ctypes.c_int

    lib.eip_disconnect.argtypes = [ctypes.c_int]
    lib.eip_disconnect.restype = ctypes.c_int

    lib.eip_read_tag.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_int]
    lib.eip_read_tag.restype = ctypes.c_int

    lib.eip_read_bool.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.POINTER(ctypes.c_int)]
    lib.eip_read_bool.restype = ctypes.c_int
    lib.eip_write_bool.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int]
    lib.eip_write_bool.restype = ctypes.c_int

    lib.eip_read_sint.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.POINTER(ctypes.c_int8)]
    lib.eip_read_sint.restype = ctypes.c_int
    lib.eip_write_sint.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int8]
    lib.eip_write_sint.restype = ctypes.c_int

    lib.eip_read_int.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.POINTER(ctypes.c_int16)]
    lib.eip_read_int.restype = ctypes.c_int
    lib.eip_write_int.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int16]
    lib.eip_write_int.restype = ctypes.c_int

    lib.eip_read_dint.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.POINTER(ctypes.c_int)]
    lib.eip_read_dint.restype = ctypes.c_int
    lib.eip_write_dint.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int]
    lib.eip_write_dint.restype = ctypes.c_int

    lib.eip_read_lint.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.POINTER(ctypes.c_int64)]
    lib.eip_read_lint.restype = ctypes.c_int
    lib.eip_write_lint.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int64]
    lib.eip_write_lint.restype = ctypes.c_int

    lib.eip_read_usint.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.POINTER(ctypes.c_uint8)]
    lib.eip_read_usint.restype = ctypes.c_int
    lib.eip_write_usint.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_uint8]
    lib.eip_write_usint.restype = ctypes.c_int

    lib.eip_read_uint.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.POINTER(ctypes.c_uint16)]
    lib.eip_read_uint.restype = ctypes.c_int
    lib.eip_write_uint.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_uint16]
    lib.eip_write_uint.restype = ctypes.c_int

    lib.eip_read_udint.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.POINTER(ctypes.c_uint32)]
    lib.eip_read_udint.restype = ctypes.c_int
    lib.eip_write_udint.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_uint32]
    lib.eip_write_udint.restype = ctypes.c_int

    lib.eip_read_ulint.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.POINTER(ctypes.c_uint64)]
    lib.eip_read_ulint.restype = ctypes.c_int
    lib.eip_write_ulint.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_uint64]
    lib.eip_write_ulint.restype = ctypes.c_int

    lib.eip_read_real.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.POINTER(ctypes.c_double)]
    lib.eip_read_real.restype = ctypes.c_int
    lib.eip_write_real.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_double]
    lib.eip_write_real.restype = ctypes.c_int

    lib.eip_read_lreal.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.POINTER(ctypes.c_double)]
    lib.eip_read_lreal.restype = ctypes.c_int
    lib.eip_write_lreal.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_double]
    lib.eip_write_lreal.restype = ctypes.c_int

    lib.eip_read_string.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_int]
    lib.eip_read_string.restype = ctypes.c_int
    lib.eip_write_string.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_char_p]
    lib.eip_write_string.restype = ctypes.c_int

    lib.eip_read_tags_batch.argtypes = [ctypes.c_int, c_char_p_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_int]
    lib.eip_read_tags_batch.restype = ctypes.c_int

    lib.eip_write_tags_batch.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_int]
    lib.eip_write_tags_batch.restype = ctypes.c_int

    lib.eip_execute_batch.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_int]
    lib.eip_execute_batch.restype = ctypes.c_int

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


def _configure_abi_signatures(lib: ctypes.CDLL) -> None:
    lib.eip_abi_version.argtypes = []
    lib.eip_abi_version.restype = ctypes.c_uint32

    lib.eip_library_version.argtypes = []
    lib.eip_library_version.restype = ctypes.c_char_p

    lib.eip_capabilities.argtypes = []
    lib.eip_capabilities.restype = ctypes.c_uint64


def _verify_abi(lib: ctypes.CDLL) -> None:
    global ABI_VERSION, LIBRARY_VERSION, CAPABILITIES

    _configure_abi_signatures(lib)
    abi_version = int(lib.eip_abi_version())
    if abi_version != EXPECTED_ABI_VERSION:
        raise NativeLibraryLoadError(
            f"native library ABI version {abi_version}, wrapper expects {EXPECTED_ABI_VERSION}"
        )

    raw_version = lib.eip_library_version()
    library_version = raw_version.decode("utf-8") if raw_version else ""

    ABI_VERSION = abi_version
    LIBRARY_VERSION = library_version
    CAPABILITIES = int(lib.eip_capabilities())


def load_native_library() -> ctypes.CDLL:
    errors: list[str] = []
    for path in _candidate_paths():
        if not path.exists():
            continue
        try:
            lib = ctypes.CDLL(str(path))
            _verify_abi(lib)
            return _configure_function_signatures(lib)
        except (AttributeError, OSError) as exc:
            errors.append(f"{path}: {exc}")

    tried = "\n".join(str(path) for path in _candidate_paths())
    details = "\n".join(errors)
    raise NativeLibraryLoadError(
        "Could not load rust_ethernet_ip native library.\n"
        f"Tried:\n{tried}\n"
        f"{details}"
    )
