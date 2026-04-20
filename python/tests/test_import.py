import unittest

from rust_ethernet_ip import Client


class ImportTests(unittest.TestCase):
    def test_client_symbol_is_exported(self) -> None:
        self.assertIsNotNone(Client)


if __name__ == "__main__":
    unittest.main()
