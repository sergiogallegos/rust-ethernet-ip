import os
import unittest

from rust_ethernet_ip import BatchWriteItem, Client

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


if __name__ == "__main__":
    unittest.main()
