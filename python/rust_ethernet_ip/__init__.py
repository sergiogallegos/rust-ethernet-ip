from . import bindings as _bindings
from .client import Client
from .exceptions import (
    BatchReadError,
    NativeLibraryLoadError,
    PlcConnectionError,
    PlcError,
    PlcOperationError,
)
from .types import BatchWriteItem, DiagnosticsSnapshot, RoutePath, WriteResult

__all__ = [
    "BatchReadError",
    "BatchWriteItem",
    "Client",
    "DiagnosticsSnapshot",
    "NativeLibraryLoadError",
    "PlcConnectionError",
    "PlcError",
    "PlcOperationError",
    "RoutePath",
    "WriteResult",
    "ABI_VERSION",
    "CAPABILITIES",
    "LIBRARY_VERSION",
]


def __getattr__(name: str):
    if name in {"ABI_VERSION", "CAPABILITIES", "LIBRARY_VERSION"}:
        return getattr(_bindings, name)
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
