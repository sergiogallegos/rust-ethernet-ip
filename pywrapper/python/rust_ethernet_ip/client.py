import asyncio
from typing import Any, List, Tuple, Optional, Union

# Import the Rust extension module (must be built with maturin or setuptools-rust)
try:
    from .rust_ethernet_ip import (
        PyEipClient,
        PyPlcValue,
        PySubscriptionOptions,
        PyRoutePath,
        PyUdtData,
    )
except ImportError as e:
    raise ImportError("The Rust extension module 'rust_ethernet_ip' could not be imported. Build it with maturin or setuptools-rust.") from e

class EipClient:
    """
    Async EtherNet/IP client for Allen-Bradley PLCs (Python wrapper for Rust).
    """
    def __init__(self, inner: Any):
        self._inner = inner

    @classmethod
    async def connect(cls, address: str) -> 'EipClient':
        """Connect to a PLC (for CompactLogix with built-in Ethernet)."""
        loop = asyncio.get_running_loop()
        inner = PyEipClient()
        await loop.run_in_executor(None, lambda: inner.connect(address))
        return cls(inner)

    @classmethod
    async def connect_with_route(cls, address: str, route_path: 'RoutePath') -> 'EipClient':
        """Connect to a PLC with route path (for ControlLogix systems)."""
        loop = asyncio.get_running_loop()
        inner = PyEipClient()
        # Access the PyRoutePath from RoutePath wrapper
        py_route = route_path._inner
        result = await loop.run_in_executor(None, lambda: inner.connect_with_route(address, py_route))
        if not result:
            raise RuntimeError(f"Failed to connect to {address} with route path")
        return cls(inner)

    async def read_tag(self, tag_name: str) -> PyPlcValue:
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(None, lambda: self._inner.read_tag(tag_name))

    async def write_tag(self, tag_name: str, value: PyPlcValue) -> None:
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(None, lambda: self._inner.write_tag(tag_name, value))

    async def read_tags_batch(self, tag_names: List[str]) -> List[Tuple[str, Union[PyPlcValue, Exception]]]:
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(None, lambda: self._inner.read_tags_batch(tag_names))

    async def write_tags_batch(self, tag_values: List[Tuple[str, PyPlcValue]]) -> List[Tuple[str, Union[None, Exception]]]:
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(None, lambda: self._inner.write_tags_batch(tag_values))

    async def unregister_session(self) -> None:
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(None, self._inner.unregister_session)

    async def subscribe_to_tag(self, tag_name: str, options: Optional[PySubscriptionOptions] = None) -> None:
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(None, lambda: self._inner.subscribe_to_tag(tag_name, options))

    async def subscribe_to_tags(self, tags: List[Tuple[str, PySubscriptionOptions]]) -> None:
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(None, lambda: self._inner.subscribe_to_tags(tags))

    async def set_route_path(self, route_path: 'RoutePath') -> None:
        """Set the route path for an existing connection."""
        loop = asyncio.get_running_loop()
        # Access the PyRoutePath from RoutePath wrapper
        py_route = route_path._inner
        return await loop.run_in_executor(None, lambda: self._inner.set_route_path(py_route))

    async def read_udt_data(self, tag_name: str) -> 'UdtData':
        """Read a UDT tag and return UdtData (new generic format)."""
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(None, lambda: self._inner.read_udt_data(tag_name))

    async def write_udt_data(self, tag_name: str, udt_data: 'UdtData') -> None:
        """Write a UDT tag using UdtData format."""
        loop = asyncio.get_running_loop()
        # Access the PyUdtData from UdtData wrapper
        py_udt = udt_data._inner
        return await loop.run_in_executor(None, lambda: self._inner.write_udt_data(tag_name, py_udt))


class RoutePath:
    """Route path for PLC communication (for ControlLogix backplane routing)."""
    def __init__(self):
        self._inner = PyRoutePath()

    def add_slot(self, slot: int) -> 'RoutePath':
        """Add a backplane slot to the route path."""
        self._inner.add_slot(slot)
        return self

    def add_port(self, port: int) -> 'RoutePath':
        """Add a network port to the route path."""
        self._inner.add_port(port)
        return self

    def add_address(self, address: str) -> 'RoutePath':
        """Add a network address to the route path."""
        self._inner.add_address(address)
        return self

    def is_empty(self) -> bool:
        """Check if the route path is empty."""
        return self._inner.is_empty()


class UdtData:
    """Raw UDT (User Defined Type) data with symbol_id and raw bytes."""
    def __init__(self, symbol_id: int = 0, data: bytes = b''):
        self._inner = PyUdtData(symbol_id, list(data))

    @property
    def symbol_id(self) -> int:
        """Get the symbol_id (template instance ID)."""
        return self._inner.symbol_id

    @symbol_id.setter
    def symbol_id(self, value: int) -> None:
        """Set the symbol_id."""
        self._inner.set_symbol_id(value)

    @property
    def data(self) -> bytes:
        """Get the raw byte data."""
        return bytes(self._inner.data)

    @data.setter
    def data(self, value: bytes) -> None:
        """Set the raw byte data."""
        self._inner.set_data(list(value))


# Re-export types for convenience
PlcValue = PyPlcValue
SubscriptionOptions = PySubscriptionOptions

__all__ = [
    'EipClient',
    'PlcValue',
    'SubscriptionOptions',
    'RoutePath',
    'UdtData',
] 