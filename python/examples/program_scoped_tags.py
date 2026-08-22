import os

from rust_ethernet_ip import Client


def main() -> None:
    address = os.getenv("RUST_ETHERNET_IP_PLC_ADDRESS", "192.168.0.10:44818")
    program = os.getenv("RUST_ETHERNET_IP_PLC_PROGRAM", "MainProgram")
    count_tag = f"Program:{program}.ProductionCount"
    setpoint_tag = f"Program:{program}.ProductionSetpoint"

    with Client(address) as plc:
        print(f"{count_tag} = {plc.read_tag(count_tag)!r}")
        plc.write_tag(setpoint_tag, 1250)

    # Python 1.2.0 accesses known program paths but does not enumerate them.


if __name__ == "__main__":
    main()
