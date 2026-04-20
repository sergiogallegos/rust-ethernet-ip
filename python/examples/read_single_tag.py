from rust_ethernet_ip import Client


def main() -> None:
    with Client("192.168.0.10:44818") as plc:
        value = plc.read_tag("Program:Main.Counter")
        print(f"Program:Main.Counter = {value!r}")


if __name__ == "__main__":
    main()
