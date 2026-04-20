import unittest

from rust_ethernet_ip.client import _decode_plc_value, _infer_value_type


class ClientValueMappingTests(unittest.TestCase):
    def test_decode_scalar_variants(self) -> None:
        self.assertIs(_decode_plc_value({"Bool": True}), True)
        self.assertEqual(_decode_plc_value({"Dint": 42}), 42)
        self.assertEqual(_decode_plc_value({"String": "hello"}), "hello")

    def test_decode_nested_dicts(self) -> None:
        value = {"outer": {"Dint": 7}, "inner": [{"Bool": False}, {"String": "x"}]}
        self.assertEqual(_decode_plc_value(value), {"outer": 7, "inner": [False, "x"]})

    def test_infer_value_type(self) -> None:
        self.assertEqual(_infer_value_type(True), "BOOL")
        self.assertEqual(_infer_value_type("abc"), "STRING")
        self.assertEqual(_infer_value_type(42), "DINT")
        self.assertEqual(_infer_value_type(3.14), "REAL")


if __name__ == "__main__":
    unittest.main()
