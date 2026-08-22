import os

from rust_ethernet_ip import Client


def main() -> None:
    address = os.getenv("RUST_ETHERNET_IP_PLC_ADDRESS", "192.168.0.10:44818")
    tag = os.getenv("RUST_ETHERNET_IP_TAG", "ProductionCount")
    with Client(address) as plc:
        value = plc.read_tag(tag)
        print(f"{tag} = {value!r}")


if __name__ == "__main__":
    main()
