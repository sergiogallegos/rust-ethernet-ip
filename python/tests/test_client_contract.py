from __future__ import annotations

import ctypes
import gc
import json
import unittest
from unittest.mock import patch

import rust_ethernet_ip.client as client_module
from rust_ethernet_ip import BatchReadError, BatchWriteItem, Client, PlcConnectionError, PlcOperationError, RoutePath
from rust_ethernet_ip.client import _normalize_write_request, _parse_write_results


class FakeNativeLibrary:
    def __init__(self) -> None:
        self.connected_addresses: list[str] = []
        self.route_calls: list[dict[str, object]] = []
        self.disconnected_ids: list[int] = []
        self.read_calls: list[str] = []
        self.executed_batches: list[list[dict[str, object]]] = []
        self._diagnostics_buffer: ctypes.Array[ctypes.c_char] | None = None
        self.connect_result = 7
        self.disconnect_result = 0
        self.read_tag_result = {"Dint": 1234}
        self.batch_read_payload: list[dict[str, object]] = []
        self.execute_batch_payload: list[dict[str, object]] | None = None
        self.typed_write_calls: list[tuple[str, str, object]] = []
        self.typed_write_result = 0
        self.health_result = 1
        self.read_tag_rc = 0
        self.last_error = b""

    def eip_connect(self, address: bytes) -> int:
        self.connected_addresses.append(address.decode("utf-8"))
        return self.connect_result

    def eip_connect_with_route_hops(
        self,
        address: bytes,
        hop_types,
        ports,
        slots,
        addresses,
        hop_count: int,
    ) -> int:
        hop_type_values = [int(hop_types[i]) for i in range(hop_count)] if hop_types else []
        slot_values = [int(slots[i]) for i in range(hop_count)] if slots else []
        port_values = [int(ports[i]) for i in range(hop_count)] if ports else []
        address_values = [
            ctypes.cast(addresses[i], ctypes.c_char_p).value.decode("utf-8")
            for i in range(hop_count)
        ] if addresses else []
        self.route_calls.append(
            {
                "address": address.decode("utf-8"),
                "hop_types": hop_type_values,
                "slots": slot_values,
                "ports": port_values,
                "addresses": address_values,
            }
        )
        return self.connect_result

    def eip_disconnect(self, client_id: int) -> int:
        self.disconnected_ids.append(client_id)
        return self.disconnect_result

    def eip_read_tag(self, client_id: int, tag_name: bytes, buffer, capacity: int) -> int:
        self.read_calls.append(tag_name.decode("utf-8"))
        if self.read_tag_rc != 0:
            return self.read_tag_rc
        payload = json.dumps(self.read_tag_result).encode("utf-8")
        if len(payload) + 1 > capacity:
            return -1
        buffer.value = payload
        return 0

    def eip_get_last_error(self, client_id: int, buffer, max_len: int) -> int:
        if not self.last_error:
            buffer.value = b""
            return 0
        if len(self.last_error) + 1 > max_len:
            return -1
        buffer.value = self.last_error
        return len(self.last_error)

    def eip_read_tags_batch(self, client_id: int, tag_names, tag_count: int, buffer, capacity: int) -> int:
        payload = json.dumps(self.batch_read_payload).encode("utf-8")
        if len(payload) + 1 > capacity:
            return -1
        buffer.value = payload
        return 0

    def eip_execute_batch(self, client_id: int, payload: bytes, operation_count: int, buffer, capacity: int) -> int:
        operations = json.loads(payload.decode("utf-8"))
        self.executed_batches.append(operations)
        result_payload = self.execute_batch_payload
        if result_payload is None:
            result_payload = [
                {
                    "tag_name": item["tag_name"],
                    "success": True,
                    "error": None,
                }
                for item in operations
            ]
        encoded = json.dumps(result_payload).encode("utf-8")
        if len(encoded) + 1 > capacity:
            return -1
        buffer.value = encoded
        return 0

    def _record_typed_write(self, function_name: str, tag_name: bytes, value: object) -> int:
        self.typed_write_calls.append((function_name, tag_name.decode("utf-8"), value))
        return self.typed_write_result

    def eip_write_bool(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record_typed_write("eip_write_bool", tag_name, value)

    def eip_write_sint(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record_typed_write("eip_write_sint", tag_name, value)

    def eip_write_int(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record_typed_write("eip_write_int", tag_name, value)

    def eip_write_dint(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record_typed_write("eip_write_dint", tag_name, value)

    def eip_write_lint(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record_typed_write("eip_write_lint", tag_name, value)

    def eip_write_usint(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record_typed_write("eip_write_usint", tag_name, value)

    def eip_write_uint(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record_typed_write("eip_write_uint", tag_name, value)

    def eip_write_udint(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record_typed_write("eip_write_udint", tag_name, value)

    def eip_write_ulint(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record_typed_write("eip_write_ulint", tag_name, value)

    def eip_write_real(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record_typed_write("eip_write_real", tag_name, value)

    def eip_write_lreal(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record_typed_write("eip_write_lreal", tag_name, value)

    def eip_write_string(self, client_id: int, tag_name: bytes, value: bytes) -> int:
        return self._record_typed_write("eip_write_string", tag_name, value.decode("utf-8"))

    def eip_check_health(self, client_id: int, is_healthy) -> int:
        is_healthy._obj.value = self.health_result
        return 0

    def eip_get_diagnostics_json(self, client_id: int, detailed: int, result_ptr) -> int:
        payload = {
            "captured_at_unix_ms": 1713550000000,
            "system_metrics_are_placeholders": True,
            "connections": {
                "active_connections": 1,
                "total_connections": 1,
                "failed_connections": 0,
                "connection_uptime_avg_seconds": 0.0,
                "last_connection_time_unix_ms": 1713550000000,
            },
            "operations": {
                "total_reads": 1,
                "total_writes": 0,
                "successful_reads": 1,
                "successful_writes": 0,
                "failed_reads": 0,
                "failed_writes": 0,
                "batch_operations": 0,
                "subscription_updates": 0,
                "partial_batch_failures": 0,
                "last_successful_read_time_unix_ms": None,
                "last_failed_read_time_unix_ms": None,
                "last_successful_write_time_unix_ms": None,
                "last_failed_write_time_unix_ms": None,
            },
            "performance": {
                "avg_read_latency_ms": 1.0,
                "avg_write_latency_ms": 0.0,
                "max_read_latency_ms": 1.0,
                "max_write_latency_ms": 0.0,
                "reads_per_second": 10.0,
                "writes_per_second": 0.0,
                "memory_usage_mb": 0.0,
                "cpu_usage_percent": 0.0,
            },
            "errors": {
                "network_errors": 0,
                "protocol_errors": 0,
                "timeout_errors": 0,
                "tag_not_found_errors": 0,
                "data_type_errors": 0,
                "session_errors": 0,
                "route_path_errors": 0,
                "embedded_service_errors": 0,
                "known_controller_limitation_errors": 0,
                "retriable_errors": 0,
                "non_retriable_errors": 0,
                "last_error_time_unix_ms": None,
                "last_error_message": None,
                "last_error_category": None,
                "last_retriable_error_time_unix_ms": None,
            },
            "health": {
                "overall_health": "Healthy",
                "health_mode": "Verified" if detailed else "Passive",
                "last_health_check_unix_ms": 1713550000000,
                "last_verified_health_check_unix_ms": 1713550000000 if detailed else None,
                "consecutive_failures": 0,
                "recovery_attempts": 0,
                "system_uptime_seconds": 10.0,
                "last_success_time_unix_ms": 1713550000000,
                "last_failure_time_unix_ms": None,
            },
        }
        encoded = json.dumps(payload).encode("utf-8")
        self._diagnostics_buffer = ctypes.create_string_buffer(encoded)
        result_ptr._obj.value = ctypes.addressof(self._diagnostics_buffer)
        return 0

    def eip_free_string(self, ptr) -> None:
        self._diagnostics_buffer = None


class ClientContractTests(unittest.TestCase):
    def test_connect_disconnect_and_context_manager_use_native_lifecycle(self) -> None:
        lib = FakeNativeLibrary()
        with patch.object(client_module, "load_native_library", return_value=lib):
            with Client("127.0.0.1:44818") as plc:
                self.assertTrue(plc.is_connected)
                self.assertEqual(plc.address, "127.0.0.1:44818")
                self.assertEqual(lib.connected_addresses, ["127.0.0.1:44818"])

            self.assertFalse(plc.is_connected)
            self.assertEqual(lib.disconnected_ids, [7])

    def test_connect_failure_raises_connection_error(self) -> None:
        lib = FakeNativeLibrary()
        lib.connect_result = -1
        with patch.object(client_module, "load_native_library", return_value=lib):
            with self.assertRaises(PlcConnectionError):
                Client("127.0.0.1:44818")

    def test_disconnect_failure_still_clears_local_state(self) -> None:
        lib = FakeNativeLibrary()
        lib.disconnect_result = -1
        with patch.object(client_module, "load_native_library", return_value=lib):
            plc = Client("127.0.0.1:44818")
            with self.assertRaises(PlcConnectionError):
                plc.disconnect()

            self.assertFalse(plc.is_connected)
            self.assertEqual(lib.disconnected_ids, [7])

    def test_finalizer_disconnects_native_handle_on_gc(self) -> None:
        lib = FakeNativeLibrary()
        with patch.object(client_module, "load_native_library", return_value=lib):
            plc = Client("127.0.0.1:44818")
            self.assertTrue(plc.is_connected)
            del plc
            gc.collect()

        self.assertEqual(lib.disconnected_ids, [7])

    def test_read_failure_includes_native_error(self) -> None:
        lib = FakeNativeLibrary()
        lib.read_tag_rc = -1
        lib.last_error = b"CIP 0x04: path segment error"
        with patch.object(client_module, "load_native_library", return_value=lib):
            plc = Client("127.0.0.1:44818")
            with self.assertRaises(PlcOperationError) as ctx:
                plc.read_tag("BadTag")
            self.assertIn("native: CIP 0x04: path segment error", str(ctx.exception))

    def test_read_failure_without_native_error_uses_plain_message(self) -> None:
        lib = FakeNativeLibrary()
        lib.read_tag_rc = -1
        with patch.object(client_module, "load_native_library", return_value=lib):
            plc = Client("127.0.0.1:44818")
            with self.assertRaises(PlcOperationError) as ctx:
                plc.read_tag("BadTag")
            self.assertNotIn("native:", str(ctx.exception))

    def test_connect_with_route_passes_ordered_hops(self) -> None:
        lib = FakeNativeLibrary()
        route = RoutePath(slots=[0, 1], ports=[2], addresses=["10.20.30.40"])
        with patch.object(client_module, "load_native_library", return_value=lib):
            plc = Client("127.0.0.1:44818", route_path=route)

        self.assertEqual(plc.route_path, route)
        self.assertEqual(
            lib.route_calls,
            [
                {
                    "address": "127.0.0.1:44818",
                    "hop_types": [1, 1, 2],
                    "slots": [0, 1, 0],
                    "ports": [1, 1, 2],
                    "addresses": ["", "", "10.20.30.40"],
                }
            ],
        )

    def test_read_tag_decodes_native_json(self) -> None:
        lib = FakeNativeLibrary()
        lib.read_tag_result = {"String": "hello"}
        with patch.object(client_module, "load_native_library", return_value=lib):
            plc = Client("127.0.0.1:44818")
            self.assertEqual(plc.read_tag("STRING_TAG"), "hello")

        self.assertEqual(lib.read_calls, ["STRING_TAG"])

    def test_read_tags_raises_batch_error_with_partial_values(self) -> None:
        lib = FakeNativeLibrary()
        lib.batch_read_payload = [
            {"tag_name": "DINT_TAG", "success": True, "value": {"Dint": 1234}},
            {"tag_name": "MISSING", "success": False, "error": "Tag not found"},
        ]
        with patch.object(client_module, "load_native_library", return_value=lib):
            plc = Client("127.0.0.1:44818")
            with self.assertRaises(BatchReadError) as ctx:
                plc.read_tags(["DINT_TAG", "MISSING"])

        self.assertEqual(ctx.exception.partial_values, {"DINT_TAG": 1234})
        self.assertEqual(ctx.exception.errors, {"MISSING": "Tag not found"})

    def test_write_tag_raises_operation_error_for_typed_failure(self) -> None:
        lib = FakeNativeLibrary()
        lib.typed_write_result = -1
        with patch.object(client_module, "load_native_library", return_value=lib):
            plc = Client("127.0.0.1:44818")
            with self.assertRaises(PlcOperationError) as ctx:
                plc.write_tag("DINT_TAG", 1)

        self.assertIn("DINT", str(ctx.exception))
        self.assertEqual(lib.typed_write_calls[0][0], "eip_write_dint")
        self.assertEqual(lib.executed_batches, [])

    def test_write_tags_executes_sequentially_to_preserve_per_tag_status(self) -> None:
        lib = FakeNativeLibrary()
        with patch.object(client_module, "load_native_library", return_value=lib):
            plc = Client("127.0.0.1:44818")
            results = plc.write_tags(
                [
                    BatchWriteItem("DINT_TAG", 1),
                    {"tag_name": "REAL_TAG", "value": 2.5},
                ]
            )

        self.assertEqual(set(results), {"DINT_TAG", "REAL_TAG"})
        self.assertTrue(all(result.success for result in results.values()))
        self.assertEqual(len(lib.executed_batches), 2)
        self.assertEqual(lib.executed_batches[0][0]["value_type"], "DINT")
        self.assertEqual(lib.executed_batches[1][0]["value_type"], "REAL")

    def test_check_health_and_diagnostics_use_native_outputs(self) -> None:
        lib = FakeNativeLibrary()
        with patch.object(client_module, "load_native_library", return_value=lib):
            plc = Client("127.0.0.1:44818")
            self.assertTrue(plc.check_health())
            snapshot = plc.get_diagnostics_snapshot(detailed=True)

        self.assertEqual(snapshot.connections.active_connections, 1)
        self.assertEqual(snapshot.operations.total_reads, 1)
        self.assertEqual(snapshot.health.health_mode, "Verified")

    def test_operations_require_connected_client(self) -> None:
        lib = FakeNativeLibrary()
        with patch.object(client_module, "load_native_library", return_value=lib):
            plc = Client("127.0.0.1:44818", auto_connect=False)
            with self.assertRaises(PlcConnectionError):
                plc.read_tag("DINT_TAG")
            with self.assertRaises(PlcConnectionError):
                plc.write_tag("DINT_TAG", 1)
            with self.assertRaises(PlcConnectionError):
                plc.check_health()


class ClientHelperContractTests(unittest.TestCase):
    def test_normalize_write_request_accepts_dataclass_and_dict(self) -> None:
        self.assertEqual(
            _normalize_write_request(BatchWriteItem("A", True)),
            {"tag_name": "A", "value": True, "value_type": "BOOL"},
        )
        self.assertEqual(
            _normalize_write_request({"tag_name": "B", "value": 123, "value_type": "LINT"}),
            {"tag_name": "B", "value": 123, "value_type": "LINT"},
        )

    def test_normalize_write_request_rejects_unknown_values(self) -> None:
        with self.assertRaises(ValueError):
            _normalize_write_request(BatchWriteItem("A", object()))

    def test_parse_write_results_validates_payload_shape(self) -> None:
        parsed = _parse_write_results([{"tag_name": "A", "success": True, "error": None}])
        self.assertTrue(parsed["A"].success)

        with self.assertRaises(PlcOperationError):
            _parse_write_results({"tag_name": "A"})
        with self.assertRaises(PlcOperationError):
            _parse_write_results(["not-a-dict"])


if __name__ == "__main__":
    unittest.main()
