import os

from rust_ethernet_ip import Client


def main() -> None:
    address = os.getenv("RUST_ETHERNET_IP_PLC_ADDRESS", "192.168.0.10:44818")
    with Client(address) as plc:
        # Replace these with dedicated writable test tags before running.
        plc.write_tag("ProductionSetpoint", 1250)
        plc.write_tag("TemperatureSetpoint", 72.5)
        plc.write_tag("EnableCommand", True)
        plc.write_tag("RecipeName", "PRODUCT_A")

        # Use value_type for a type Python cannot infer from the value alone.
        plc.write_tag("SmallCounter", 123, value_type="INT")

        # 1.2.0 supports built-in/custom STRING members by full tag path.
        plc.write_tag("Mixer.Description", "Primary mixer")
        print(plc.read_string("Mixer.Description"))


if __name__ == "__main__":
    main()
