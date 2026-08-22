import os

from rust_ethernet_ip import Client, RoutePath


def main() -> None:
    module_address = os.getenv("RUST_ETHERNET_IP_PLC_ADDRESS", "192.168.0.20:44818")
    cpu_slot = int(os.getenv("RUST_ETHERNET_IP_PLC_SLOT", "0"))

    with Client(module_address, route_path=RoutePath(slots=[cpu_slot])) as plc:
        print(f"ProductionCount = {plc.read_tag('ProductionCount')!r}")


if __name__ == "__main__":
    main()
