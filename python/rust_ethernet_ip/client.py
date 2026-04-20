from __future__ import annotations

import json
from ctypes import byref, c_char_p, c_int, c_void_p, cast, create_string_buffer

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


class Client:
    def __init__(self, address: str, *, auto_connect: bool = True):
        self._address = address
        self._lib = load_native_library()
        self._client_id: int | None = None
        if auto_connect:
            self.connect()

    @property
    def address(self) -> str:
        return self._address

    @property
    def is_connected(self) -> bool:
        return self._client_id is not None and self._client_id >= 0

    def connect(self) -> None:
        if self.is_connected:
            return

        client_id = self._lib.eip_connect(self._address.encode("utf-8"))
        if client_id < 0:
            raise PlcConnectionError(f"Failed to connect to PLC at {self._address}")
        self._client_id = int(client_id)

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
        item = BatchWriteItem(tag_name=tag_name, value=value, value_type=value_type)
        results = self.write_tags([item])
        result = results[tag_name]
        if not result.success:
            raise PlcOperationError(result.error or f"Failed to write tag '{tag_name}'")

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

        client_id = self._require_client_id()
        normalized: list[dict[str, object]] = []
        for item in items:
            if isinstance(item, BatchWriteItem):
                tag_name = item.tag_name
                value = item.value
                value_type = item.value_type
            else:
                tag_name = str(item["tag_name"])
                value = item["value"]
                value_type = item.get("value_type")

            normalized.append(
                {
                    "tag_name": tag_name,
                    "value": value,
                    "value_type": value_type or _infer_value_type(value),
                }
            )

        payload = json.dumps(normalized).encode("utf-8")
        buffer = create_string_buffer(RESULT_BUFFER_SIZE)
        rc = self._lib.eip_write_tags_batch(client_id, payload, len(normalized), buffer, RESULT_BUFFER_SIZE)
        if rc != 0 and not buffer.value:
            raise PlcOperationError("Batch write call failed")

        try:
            result_items = json.loads(buffer.value.decode("utf-8"))
        except json.JSONDecodeError as exc:
            raise PlcOperationError("Invalid JSON returned for batch write") from exc

        results: dict[str, WriteResult] = {}
        for item in result_items:
            results[item["tag_name"]] = WriteResult(
                tag_name=item["tag_name"],
                success=bool(item["success"]),
                error=item.get("error"),
            )

        return results

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
