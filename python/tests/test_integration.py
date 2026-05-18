import os
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


if __name__ == "__main__":
    unittest.main()
