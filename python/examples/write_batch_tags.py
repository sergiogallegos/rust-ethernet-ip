import os

from rust_ethernet_ip import BatchWriteItem, Client


def main() -> None:
    address = os.getenv("RUST_ETHERNET_IP_PLC_ADDRESS", "192.168.0.10:44818")
    with Client(address) as plc:
        results = plc.write_tags([
            BatchWriteItem("ProductionSetpoint", 1250),
            BatchWriteItem("TemperatureSetpoint", 72.5),
            BatchWriteItem("EnableCommand", True),
            BatchWriteItem("RecipeName", "PRODUCT_A"),
            BatchWriteItem("SmallCounter", 123, value_type="INT"),
        ])

        for tag, result in results.items():
            print(f"{tag}: {'ok' if result.success else result.error}")


if __name__ == "__main__":
    main()
