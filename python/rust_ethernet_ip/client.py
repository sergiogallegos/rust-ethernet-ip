from __future__ import annotations

import json
from ctypes import (
    POINTER,
    byref,
    c_char_p,
    c_double,
    c_int,
    c_int8,
    c_int16,
    c_int64,
    c_uint8,
    c_uint16,
    c_uint32,
    c_uint64,
    c_void_p,
    cast,
    create_string_buffer,
)

from .bindings import READ_BUFFER_SIZE, RESULT_BUFFER_SIZE, load_native_library
from .exceptions import BatchReadError, PlcConnectionError, PlcOperationError
from .types import (
    BatchWriteItem,
    DiagnosticsConnectionMetrics,
    DiagnosticsErrorMetrics,
    DiagnosticsHealthMetrics,
    DiagnosticsOperationMetrics,
    DiagnosticsPerformanceMetrics,
    DiagnosticsSnapshot,
    RoutePath,
    WriteResult,
)


_PLC_VARIANTS = {
    "Bool",
    "Sint",
    "Int",
    "Dint",
    "Lint",
    "Usint",
    "Uint",
    "Udint",
    "Ulint",
    "Real",
    "Lreal",
    "String",
    "Udt",
}


_INTEGER_RANGES = {
    "SINT": (-(2**7), 2**7 - 1),
    "INT": (-(2**15), 2**15 - 1),
    "DINT": (-(2**31), 2**31 - 1),
    "LINT": (-(2**63), 2**63 - 1),
    "USINT": (0, 2**8 - 1),
    "UINT": (0, 2**16 - 1),
    "UDINT": (0, 2**32 - 1),
    "ULINT": (0, 2**64 - 1),
}


_INTEGER_CTYPES = {
    "SINT": c_int8,
    "INT": c_int16,
    "DINT": c_int,
    "LINT": c_int64,
    "USINT": c_uint8,
    "UINT": c_uint16,
    "UDINT": c_uint32,
    "ULINT": c_uint64,
}


def _decode_logix_string_payload(payload: object) -> str | None:
    if not isinstance(payload, dict):
        return None
    if set(payload.keys()) != {"symbol_id", "data"}:
        return None

    data = payload.get("data")
    if not isinstance(data, list) or any(not isinstance(byte, int) for byte in data):
        return None

    buffer = bytes(data)
    for header_size in (2, 0):
        if header_size == 2:
            if len(buffer) < 6 or buffer[:2] != b"\xCE\x0F":
                continue
        elif len(buffer) < 4:
            continue

        length_offset = header_size
        length = int.from_bytes(
            buffer[length_offset : length_offset + 4],
            byteorder="little",
            signed=False,
        )
        string_data = buffer[length_offset + 4 :]
        if length > 82 or length > len(string_data):
            continue

        return string_data[:length].decode("utf-8", errors="replace")

    return None


def _decode_plc_value(value: object) -> object:
    if isinstance(value, dict) and len(value) == 1:
        key, payload = next(iter(value.items()))
        if key in _PLC_VARIANTS:
            if key == "Udt":
                return _decode_udt(payload)
            return payload

    if isinstance(value, list):
        return [_decode_plc_value(item) for item in value]

    if isinstance(value, dict):
        return {name: _decode_plc_value(item) for name, item in value.items()}

    return value


def _decode_udt(payload: object) -> object:
    if isinstance(payload, dict):
        decoded_string = _decode_logix_string_payload(payload)
        if decoded_string is not None:
            return decoded_string
        return {name: _decode_plc_value(value) for name, value in payload.items()}
    return payload


def _infer_value_type(value: object) -> str:
    if isinstance(value, bool):
        return "BOOL"
    if isinstance(value, str):
        return "STRING"
    if isinstance(value, float):
        return "REAL"
    if isinstance(value, int):
        if -(2**31) <= value <= (2**31 - 1):
            return "DINT"
        if -(2**63) <= value <= (2**63 - 1):
            return "LINT"
        raise ValueError("Integer is outside supported LINT range")
    if isinstance(value, dict) and "symbol_id" in value and "data" in value:
        return "UDT"

    raise ValueError(
        "Could not infer PLC value type. Pass value_type explicitly for this value."
    )


def _parse_diagnostics_snapshot(payload: dict[str, object]) -> DiagnosticsSnapshot:
    return DiagnosticsSnapshot(
        captured_at_unix_ms=payload.get("captured_at_unix_ms"),
        system_metrics_are_placeholders=bool(payload.get("system_metrics_are_placeholders", True)),
        connections=DiagnosticsConnectionMetrics(**payload["connections"]),
        operations=DiagnosticsOperationMetrics(**payload["operations"]),
        performance=DiagnosticsPerformanceMetrics(**payload["performance"]),
        errors=DiagnosticsErrorMetrics(**payload["errors"]),
        health=DiagnosticsHealthMetrics(**payload["health"]),
    )


def _normalize_write_request(item: BatchWriteItem | dict[str, object]) -> dict[str, object]:
    if isinstance(item, BatchWriteItem):
        tag_name = item.tag_name
        value = item.value
        value_type = item.value_type
    else:
        tag_name = str(item["tag_name"])
        value = item["value"]
        value_type = item.get("value_type")

    return {
        "tag_name": tag_name,
        "value": value,
        "value_type": value_type or _infer_value_type(value),
    }


def _validate_integer_value(kind: str, value: object) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise ValueError(f"{kind} writes require an integer value")

    low, high = _INTEGER_RANGES[kind]
    if not low <= value <= high:
        raise ValueError(f"{kind} value {value} is outside range {low}..{high}")
    return value


def _validate_float_value(kind: str, value: object) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f"{kind} writes require a numeric value")
    return float(value)


def _parse_write_results(payload: object) -> dict[str, WriteResult]:
    if not isinstance(payload, list):
        raise PlcOperationError("Invalid JSON returned for batch write")

    results: dict[str, WriteResult] = {}
    for item in payload:
        if not isinstance(item, dict):
            raise PlcOperationError("Invalid JSON returned for batch write")
        tag_name = item["tag_name"]
        results[tag_name] = WriteResult(
            tag_name=tag_name,
            success=bool(item["success"]),
            error=item.get("error"),
        )
    return results


class Client:
    def __init__(
        self,
        address: str,
        *,
        route_path: RoutePath | None = None,
        auto_connect: bool = True,
    ):
        self._address = address
        self._route_path = route_path
        self._lib = load_native_library()
        self._client_id: int | None = None
        if auto_connect:
            self.connect()

    @property
    def address(self) -> str:
        return self._address

    @property
    def route_path(self) -> RoutePath | None:
        return self._route_path

    @property
    def is_connected(self) -> bool:
        return self._client_id is not None and self._client_id >= 0

    def connect(self) -> None:
        if self.is_connected:
            return

        if self._route_path is None:
            client_id = self._lib.eip_connect(self._address.encode("utf-8"))
        else:
            client_id = self._connect_with_route(self._route_path)
        if client_id < 0:
            raise PlcConnectionError(f"Failed to connect to PLC at {self._address}")
        self._client_id = int(client_id)

    def _connect_with_route(self, route_path: RoutePath) -> int:
        hops = route_path.ordered_hops()
        hop_types = [1 if hop.kind == "backplane" else 2 for hop in hops]
        ports = [hop.port for hop in hops]
        slots = [hop.slot or 0 for hop in hops]
        address_bytes = [
            (hop.address or "").encode("utf-8") if hop.kind == "ethernet" else b""
            for hop in hops
        ]

        hop_type_array = (c_uint8 * len(hop_types))(*hop_types) if hop_types else None
        port_array = (c_uint8 * len(ports))(*ports) if ports else None
        slot_array = (c_uint8 * len(slots))(*slots) if slots else None
        address_array = (c_char_p * len(address_bytes))(*address_bytes) if address_bytes else None

        return self._lib.eip_connect_with_route_hops(
            self._address.encode("utf-8"),
            cast(hop_type_array, POINTER(c_uint8)) if hop_type_array is not None else None,
            cast(port_array, POINTER(c_uint8)) if port_array is not None else None,
            cast(slot_array, POINTER(c_uint8)) if slot_array is not None else None,
            cast(address_array, POINTER(c_char_p)) if address_array is not None else None,
            len(hops),
        )

    def disconnect(self) -> None:
        if not self.is_connected:
            return
        rc = self._lib.eip_disconnect(self._client_id)
        if rc != 0:
            raise PlcConnectionError("Failed to disconnect from PLC")
        self._client_id = None

    def close(self) -> None:
        self.disconnect()

    def __enter__(self) -> Client:
        if not self.is_connected:
            self.connect()
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        self.disconnect()

    def _require_client_id(self) -> int:
        if not self.is_connected or self._client_id is None:
            raise PlcConnectionError("Client is not connected")
        return self._client_id

    def read_tag(self, tag_name: str) -> object:
        client_id = self._require_client_id()
        buffer = create_string_buffer(READ_BUFFER_SIZE)
        rc = self._lib.eip_read_tag(client_id, tag_name.encode("utf-8"), buffer, READ_BUFFER_SIZE)
        if rc != 0:
            raise PlcOperationError(f"Failed to read tag '{tag_name}'")

        try:
            decoded = json.loads(buffer.value.decode("utf-8"))
        except json.JSONDecodeError as exc:
            raise PlcOperationError(f"Invalid JSON returned for tag '{tag_name}'") from exc

        return _decode_plc_value(decoded)

    def write_tag(self, tag_name: str, value: object, *, value_type: str | None = None) -> None:
        kind = (value_type or _infer_value_type(value)).upper()

        if kind == "UDT":
            # UDT writes need symbol-id/raw-data handling and remain on the batch path for now.
            item = _normalize_write_request(
                BatchWriteItem(tag_name=tag_name, value=value, value_type=kind)
            )
            results = self._execute_write_operations([item])
            result = results[tag_name]
            if not result.success:
                raise PlcOperationError(result.error or f"Failed to write tag '{tag_name}'")
            return

        client_id = self._require_client_id()
        encoded_tag = tag_name.encode("utf-8")

        if kind == "BOOL":
            if not isinstance(value, bool):
                raise ValueError("BOOL writes require a bool value")
            rc = self._lib.eip_write_bool(client_id, encoded_tag, 1 if value else 0)
        elif kind in _INTEGER_CTYPES:
            integer_value = _validate_integer_value(kind, value)
            rc = getattr(self._lib, f"eip_write_{kind.lower()}")(
                client_id,
                encoded_tag,
                _INTEGER_CTYPES[kind](integer_value),
            )
        elif kind == "REAL":
            rc = self._lib.eip_write_real(
                client_id,
                encoded_tag,
                c_double(_validate_float_value(kind, value)),
            )
        elif kind == "LREAL":
            rc = self._lib.eip_write_lreal(
                client_id,
                encoded_tag,
                c_double(_validate_float_value(kind, value)),
            )
        elif kind == "STRING":
            if not isinstance(value, str):
                raise ValueError("STRING writes require a string value")
            rc = self._lib.eip_write_string(client_id, encoded_tag, value.encode("utf-8"))
        else:
            raise ValueError(f"Unsupported PLC value type for single-tag write: {kind}")

        if rc != 0:
            raise PlcOperationError(
                f"Failed to write tag '{tag_name}' via {kind} typed FFI export (rc={rc})"
            )

    def read_tags(self, tag_names: list[str]) -> dict[str, object]:
        if not tag_names:
            return {}

        client_id = self._require_client_id()
        encoded = [name.encode("utf-8") for name in tag_names]
        array = (c_char_p * len(encoded))(*encoded)
        buffer = create_string_buffer(RESULT_BUFFER_SIZE)
        rc = self._lib.eip_read_tags_batch(client_id, array, len(encoded), buffer, RESULT_BUFFER_SIZE)
        if rc != 0:
            raise PlcOperationError("Batch read call failed")

        try:
            payload = json.loads(buffer.value.decode("utf-8"))
        except json.JSONDecodeError as exc:
            raise PlcOperationError("Invalid JSON returned for batch read") from exc

        values: dict[str, object] = {}
        errors: dict[str, str] = {}
        for item in payload:
            tag_name = item["tag_name"]
            if item["success"]:
                values[tag_name] = _decode_plc_value(item["value"])
            else:
                errors[tag_name] = item.get("error") or "Unknown error"

        if errors:
            raise BatchReadError("One or more batch read operations failed", errors, values)

        return values

    def write_tags(self, items: list[BatchWriteItem | dict[str, object]]) -> dict[str, WriteResult]:
        if not items:
            return {}

        normalized = [_normalize_write_request(item) for item in items]
        return self._execute_write_operations(normalized, sequential=True)

    def _execute_write_operations(
        self,
        normalized: list[dict[str, object]],
        *,
        sequential: bool = False,
    ) -> dict[str, WriteResult]:
        client_id = self._require_client_id()
        if sequential:
            results: dict[str, WriteResult] = {}
            for item in normalized:
                results.update(self._execute_write_operations([item], sequential=False))
            return results

        operations = [
            {
                "tag_name": item["tag_name"],
                "is_write": True,
                "value_type": item["value_type"],
                "value": item["value"],
            }
            for item in normalized
        ]

        payload = json.dumps(operations).encode("utf-8")
        buffer = create_string_buffer(RESULT_BUFFER_SIZE)
        rc = self._lib.eip_execute_batch(
            client_id,
            payload,
            len(operations),
            buffer,
            RESULT_BUFFER_SIZE,
        )
        if rc != 0 and not buffer.value:
            raise PlcOperationError("Batch write call failed")

        try:
            result_items = json.loads(buffer.value.decode("utf-8"))
        except json.JSONDecodeError as exc:
            raise PlcOperationError("Invalid JSON returned for batch write") from exc

        return _parse_write_results(result_items)

    def check_health(self) -> bool:
        client_id = self._require_client_id()
        is_healthy = c_int(0)
        rc = self._lib.eip_check_health(client_id, byref(is_healthy))
        return rc == 0 and is_healthy.value != 0

    def get_diagnostics_snapshot(self, *, detailed: bool = False) -> DiagnosticsSnapshot:
        client_id = self._require_client_id()
        result_ptr = c_void_p()
        rc = self._lib.eip_get_diagnostics_json(client_id, 1 if detailed else 0, byref(result_ptr))
        if rc != 0 or not result_ptr.value:
            raise PlcOperationError("Failed to retrieve diagnostics snapshot")

        try:
            raw = cast(result_ptr, c_char_p).value
            if not raw:
                raise PlcOperationError("Native diagnostics snapshot was empty")
            payload = json.loads(raw.decode("utf-8"))
        except json.JSONDecodeError as exc:
            raise PlcOperationError("Invalid JSON returned for diagnostics snapshot") from exc
        finally:
            self._lib.eip_free_string(result_ptr)

        return _parse_diagnostics_snapshot(payload)
