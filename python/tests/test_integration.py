import unittest

from rust_ethernet_ip import BatchWriteItem, Client, RoutePath
from rust_ethernet_ip.exceptions import BatchReadError

try:
    from .sim_harness import SimulatorHarness
except ImportError:
    from sim_harness import SimulatorHarness

class SimulatorIntegrationTests(unittest.TestCase):
    def test_connect_read_write_and_health(self) -> None:
        with SimulatorHarness() as address:
            with Client(address) as plc:
                self.assertTrue(plc.check_health())

                self.assertEqual(plc.read_tag("DINT_TAG"), 1234)
                self.assertIs(plc.read_tag("BOOL_TAG"), True)
                self.assertEqual(plc.read_tag("STRING_TAG"), "Hello PLC")

                plc.write_tag("DINT_TAG", 4321)
                plc.write_tag("BOOL_TAG", False)
                plc.write_tag("STRING_TAG", "Updated")

                self.assertEqual(plc.read_tag("DINT_TAG"), 4321)
                self.assertIs(plc.read_tag("BOOL_TAG"), False)
                self.assertEqual(plc.read_tag("STRING_TAG"), "Updated")

    def test_batch_read_and_write(self) -> None:
        with SimulatorHarness() as address:
            with Client(address) as plc:
                initial = plc.read_tags(["DINT_TAG", "REAL_TAG", "BOOL_TAG", "STRING_TAG"])
                self.assertEqual(initial["DINT_TAG"], 1234)
                self.assertEqual(initial["REAL_TAG"], 3.0)
                self.assertIs(initial["BOOL_TAG"], True)
                self.assertEqual(initial["STRING_TAG"], "Hello PLC")

                results = plc.write_tags(
                    [
                        BatchWriteItem("DINT_TAG", 2468),
                        BatchWriteItem("REAL_TAG", 6.5),
                        BatchWriteItem("BOOL_TAG", False),
                        BatchWriteItem("STRING_TAG", "Batch Updated"),
                    ]
                )

                self.assertTrue(all(item.success for item in results.values()))

                updated = plc.read_tags(["DINT_TAG", "REAL_TAG", "BOOL_TAG", "STRING_TAG"])
                self.assertEqual(updated["DINT_TAG"], 2468)
                self.assertEqual(updated["REAL_TAG"], 6.5)
                self.assertIs(updated["BOOL_TAG"], False)
                self.assertEqual(updated["STRING_TAG"], "Batch Updated")

    def test_native_batch_partial_failure_and_array_result_correlation(self) -> None:
        with SimulatorHarness() as address:
            with Client(address) as plc:
                int_before = plc.read_tag("INT_TAG")
                results = plc.write_tags(
                    [
                        BatchWriteItem("DINT_TAG", 1357),
                        BatchWriteItem("INT_TAG", 99_999, value_type="INT"),
                        BatchWriteItem("DINT_ARRAY[1]", 2468),
                    ]
                )

                self.assertTrue(results["DINT_TAG"].success)
                self.assertFalse(results["INT_TAG"].success)
                self.assertTrue(results["DINT_ARRAY[1]"].success)
                self.assertEqual(plc.read_tag("DINT_TAG"), 1357)
                self.assertEqual(plc.read_tag("INT_TAG"), int_before)
                self.assertEqual(plc.read_tag("DINT_ARRAY[1]"), 2468)

    def test_duplicate_batch_names_execute_sequentially(self) -> None:
        with SimulatorHarness() as address:
            with Client(address) as plc:
                results = plc.write_tags(
                    [
                        BatchWriteItem("DINT_TAG", 1111),
                        BatchWriteItem("DINT_TAG", 2222),
                    ]
                )

                self.assertTrue(results["DINT_TAG"].success)
                self.assertEqual(plc.read_tag("DINT_TAG"), 2222)

    def test_bool_array_element_write_uses_typed_path(self) -> None:
        with SimulatorHarness() as address:
            with Client(address) as plc:
                plc.write_tag("BOOL_ARRAY[5]", True)
                self.assertIs(plc.read_tag("BOOL_ARRAY[5]"), True)

                plc.write_tag("BOOL_ARRAY[5]", False)
                self.assertIs(plc.read_tag("BOOL_ARRAY[5]"), False)

    def test_grouped_bool_array_writes_use_safe_fallback_above_dword_zero(self) -> None:
        with SimulatorHarness() as address:
            with Client(address) as plc:
                results = plc.write_tags(
                    [
                        BatchWriteItem("BOOL_ARRAY[5]", True),
                        BatchWriteItem("BOOL_ARRAY[40]", True),
                    ]
                )
                self.assertTrue(all(result.success for result in results.values()))
                self.assertIs(plc.read_tag("BOOL_ARRAY[5]"), True)
                self.assertIs(plc.read_tag("BOOL_ARRAY[40]"), True)

                restore = plc.write_tags(
                    [
                        BatchWriteItem("BOOL_ARRAY[5]", False),
                        BatchWriteItem("BOOL_ARRAY[40]", False),
                    ]
                )
                self.assertTrue(all(result.success for result in restore.values()))

    def test_typed_write_roundtrip_all_numeric_types(self) -> None:
        cases = [
            ("SINT_TAG", -7, "SINT"),
            ("INT_TAG", -123, "INT"),
            ("DINT_TAG", 4567, "DINT"),
            ("LINT_TAG", -9876543210, "LINT"),
            ("USINT_TAG", 7, "USINT"),
            ("UINT_TAG", 123, "UINT"),
            ("UDINT_TAG", 456789, "UDINT"),
            ("ULINT_TAG", 9876543210, "ULINT"),
            ("REAL_TAG", 8.25, "REAL"),
            ("LREAL_TAG", 9.5, "LREAL"),
        ]

        with SimulatorHarness() as address:
            with Client(address) as plc:
                for tag_name, value, value_type in cases:
                    with self.subTest(tag_name=tag_name):
                        plc.write_tag(tag_name, value, value_type=value_type)
                        self.assertEqual(plc.read_tag(tag_name), value)

    def test_out_of_range_write_raises_value_error_without_changing_tag(self) -> None:
        with SimulatorHarness() as address:
            with Client(address) as plc:
                before = plc.read_tag("INT_TAG")
                with self.assertRaises(ValueError):
                    plc.write_tag("INT_TAG", 99_999, value_type="INT")
                self.assertEqual(plc.read_tag("INT_TAG"), before)

    def test_connect_with_route_path(self) -> None:
        with SimulatorHarness() as address:
            with Client(address, route_path=RoutePath(slots=[0])) as plc:
                self.assertTrue(plc.check_health())
                self.assertEqual(plc.read_tag("DINT_TAG"), 1234)

    def test_batch_read_partial_failure_preserves_successful_values(self) -> None:
        with SimulatorHarness() as address:
            with Client(address) as plc:
                with self.assertRaises(BatchReadError) as ctx:
                    plc.read_tags(["DINT_TAG", "THIS_TAG_DOES_NOT_EXIST"])

                self.assertEqual(ctx.exception.partial_values["DINT_TAG"], 1234)
                self.assertIn("THIS_TAG_DOES_NOT_EXIST", ctx.exception.errors)

    def test_diagnostics_snapshot_maps_native_payload(self) -> None:
        with SimulatorHarness() as address:
            with Client(address) as plc:
                snapshot = plc.get_diagnostics_snapshot(detailed=True)

                self.assertIsNotNone(snapshot.captured_at_unix_ms)
                self.assertGreaterEqual(snapshot.connections.active_connections, 1)
                self.assertIn(snapshot.health.overall_health, {"Healthy", "Warning", "Critical", "Unknown"})
                self.assertIn(snapshot.health.health_mode, {"Passive", "Verified"})
                generation = snapshot.schema_cache.generation
                plc.refresh_schema()
                refreshed = plc.get_diagnostics_snapshot()
                self.assertEqual(refreshed.schema_cache.generation, generation + 1)
                self.assertEqual(refreshed.schema_cache.refreshes, snapshot.schema_cache.refreshes + 1)


if __name__ == "__main__":
    unittest.main()
