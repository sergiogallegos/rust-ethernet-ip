class PlcError(Exception):
    """Base exception for Python wrapper errors."""


class NativeLibraryLoadError(PlcError):
    """Raised when the native rust_ethernet_ip library cannot be loaded."""


class PlcConnectionError(PlcError):
    """Raised when connecting to or disconnecting from a PLC fails."""


class PlcOperationError(PlcError):
    """Raised when a PLC read/write operation fails."""


class BatchReadError(PlcOperationError):
    """Raised when a batch read completes with one or more per-tag failures."""

    def __init__(self, message: str, errors: dict[str, str], partial_values: dict[str, object] | None = None):
        super().__init__(message)
        self.errors = errors
        self.partial_values = partial_values or {}
