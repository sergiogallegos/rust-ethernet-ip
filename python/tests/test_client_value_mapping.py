import unittest
from unittest.mock import patch

import rust_ethernet_ip.client as client_module
from rust_ethernet_ip import BatchWriteItem, Client
from rust_ethernet_ip.client import _decode_plc_value, _infer_value_type


class RecordingNativeLibrary:
    def __init__(self) -> None:
        self.calls: list[tuple[str, str, object]] = []
        self.batch_calls = 0

    def eip_connect(self, address: bytes) -> int:
        return 11

    def eip_disconnect(self, client_id: int) -> int:
        return 0

    def eip_execute_batch(self, client_id: int, payload: bytes, operation_count: int, buffer, capacity: int) -> int:
        self.batch_calls += 1
        buffer.value = b'[{"tag_name":"UDT_TAG","success":true,"error":null}]'
        return 0

    def eip_write_tags_batch(self, *args) -> int:
        self.batch_calls += 1
        return 0

    def _record(self, name: str, tag_name: bytes, value: object) -> int:
        self.calls.append((name, tag_name.decode("utf-8"), getattr(value, "value", value)))
        return 0

    def eip_write_bool(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record("eip_write_bool", tag_name, value)

    def eip_write_sint(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record("eip_write_sint", tag_name, value)

    def eip_write_int(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record("eip_write_int", tag_name, value)

    def eip_write_dint(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record("eip_write_dint", tag_name, value)

    def eip_write_lint(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record("eip_write_lint", tag_name, value)

    def eip_write_usint(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record("eip_write_usint", tag_name, value)

    def eip_write_uint(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record("eip_write_uint", tag_name, value)

    def eip_write_udint(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record("eip_write_udint", tag_name, value)

    def eip_write_ulint(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record("eip_write_ulint", tag_name, value)

    def eip_write_real(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record("eip_write_real", tag_name, value)

    def eip_write_lreal(self, client_id: int, tag_name: bytes, value: object) -> int:
        return self._record("eip_write_lreal", tag_name, value)

    def eip_write_string(self, client_id: int, tag_name: bytes, value: bytes) -> int:
        return self._record("eip_write_string", tag_name, value.decode("utf-8"))


class ClientValueMappingTests(unittest.TestCase):
    def test_decode_scalar_variants(self) -> None:
        self.assertIs(_decode_plc_value({"Bool": True}), True)
        self.assertEqual(_decode_plc_value({"Dint": 42}), 42)
        self.assertEqual(_decode_plc_value({"String": "hello"}), "hello")

    def test_decode_nested_dicts(self) -> None:
        value = {"outer": {"Dint": 7}, "inner": [{"Bool": False}, {"String": "x"}]}
        self.assertEqual(_decode_plc_value(value), {"outer": 7, "inner": [False, "x"]})

    def test_decode_logix_string_payload_with_header(self) -> None:
        value = {
            "Udt": {
                "symbol_id": 0,
                "data": [0xCE, 0x0F, 5, 0, 0, 0, 72, 69, 76, 76, 79],
            }
        }
        self.assertEqual(_decode_plc_value(value), "HELLO")

    def test_preserve_generic_udt_payloads(self) -> None:
        value = {"Udt": {"symbol_id": 123, "data": [1, 2, 3, 4]}}
        self.assertEqual(_decode_plc_value(value), {"symbol_id": 123, "data": [1, 2, 3, 4]})

    def test_headerless_binary_udt_not_misdecoded_as_string(self) -> None:
        # Leading bytes look like a length-3 STRING ("ABC"), but the trailing
        # bytes are non-zero (not STRING padding), so a generic UDT must be
        # preserved rather than mis-decoded as a garbage string.
        value = {"Udt": {"symbol_id": 5, "data": [3, 0, 0, 0, 65, 66, 67, 1, 2, 3]}}
        self.assertEqual(
            _decode_plc_value(value),
            {"symbol_id": 5, "data": [3, 0, 0, 0, 65, 66, 67, 1, 2, 3]},
        )

    def test_headerless_string_with_zero_padding_decodes(self) -> None:
        # length-3 "ABC" with zero padding past the used length decodes as STRING.
        value = {"Udt": {"symbol_id": 5, "data": [3, 0, 0, 0, 65, 66, 67, 0, 0]}}
        self.assertEqual(_decode_plc_value(value), "ABC")

    def test_infer_value_type(self) -> None:
        self.assertEqual(_infer_value_type(True), "BOOL")
        self.assertEqual(_infer_value_type("abc"), "STRING")
        self.assertEqual(_infer_value_type(42), "DINT")
        self.assertEqual(_infer_value_type(3.14), "REAL")

    def test_write_tag_dispatches_to_typed_export(self) -> None:
        cases = [
            (True, None, "eip_write_bool", 1),
            (-12, "SINT", "eip_write_sint", -12),
            (-1234, "INT", "eip_write_int", -1234),
            (1234, None, "eip_write_dint", 1234),
            (12345678901, None, "eip_write_lint", 12345678901),
            (12, "USINT", "eip_write_usint", 12),
            (1234, "UINT", "eip_write_uint", 1234),
            (123456789, "UDINT", "eip_write_udint", 123456789),
            (12345678901, "ULINT", "eip_write_ulint", 12345678901),
            (1.25, None, "eip_write_real", 1.25),
            (2.5, "LREAL", "eip_write_lreal", 2.5),
        ]

        for value, value_type, expected_fn, expected_value in cases:
            with self.subTest(value=value, value_type=value_type):
                lib = RecordingNativeLibrary()
                with patch.object(client_module, "load_native_library", return_value=lib):
                    plc = Client("127.0.0.1:44818")
                    plc.write_tag("TAG", value, value_type=value_type)

                self.assertEqual(lib.calls, [(expected_fn, "TAG", expected_value)])
                self.assertEqual(lib.batch_calls, 0)

    def test_write_tag_string_uses_typed_export(self) -> None:
        lib = RecordingNativeLibrary()
        with patch.object(client_module, "load_native_library", return_value=lib):
            plc = Client("127.0.0.1:44818")
            plc.write_tag("STRING_TAG", "hello")

        self.assertEqual(lib.calls, [("eip_write_string", "STRING_TAG", "hello")])
        self.assertEqual(lib.batch_calls, 0)

    def test_write_tag_udt_still_uses_batch_path(self) -> None:
        lib = RecordingNativeLibrary()
        with patch.object(client_module, "load_native_library", return_value=lib):
            plc = Client("127.0.0.1:44818")
            plc.write_tag("UDT_TAG", {"symbol_id": 1, "data": [1, 2, 3]})

        self.assertEqual(lib.calls, [])
        self.assertEqual(lib.batch_calls, 1)

    def test_out_of_range_write_raises_value_error_before_ffi_call(self) -> None:
        lib = RecordingNativeLibrary()
        with patch.object(client_module, "load_native_library", return_value=lib):
            plc = Client("127.0.0.1:44818")
            with self.assertRaises(ValueError):
                plc.write_tag("INT_TAG", 99_999, value_type="INT")

        self.assertEqual(lib.calls, [])
        self.assertEqual(lib.batch_calls, 0)


if __name__ == "__main__":
    unittest.main()
