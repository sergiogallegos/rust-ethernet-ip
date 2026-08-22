import os

from rust_ethernet_ip import BatchReadError, Client


def main() -> None:
    address = os.getenv("RUST_ETHERNET_IP_PLC_ADDRESS", "192.168.0.10:44818")
    tags = ["ProductionCount", "TankTemperature", "Program:MainProgram.MachineRunning"]
    with Client(address) as plc:
        try:
            values = plc.read_tags(tags)
        except BatchReadError as exc:
            print("Batch read returned partial failures:")
            print(exc.partial_values)
            print(exc.errors)
            raise

        for name, value in values.items():
            print(f"{name} = {value!r}")


if __name__ == "__main__":
    main()
