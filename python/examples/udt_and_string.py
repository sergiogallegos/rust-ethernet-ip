import os

from rust_ethernet_ip import Client


def main() -> None:
    address = os.getenv("RUST_ETHERNET_IP_PLC_ADDRESS", "192.168.0.10:44818")
    with Client(address) as plc:
        # Read the whole structure for one logical snapshot. The result may be
        # decoded members or a {symbol_id, data} raw representation.
        print("Mixer snapshot:", plc.read_tag("Mixer"))

        # Prefer full member paths for normal reads, commands, and setpoints.
        print("Speed:", plc.read_tag("Mixer.SpeedFeedback"))
        print("Description:", plc.read_string("Mixer.Description"))
        plc.write_tag("Mixer.SpeedSetpoint", 60.0)
        plc.write_tag("Mixer.Enabled", True)
        plc.write_tag("Mixer.Description", "Primary mixer")

        # Whole array-element reads work. Write its members individually;
        # writing Motors[0] as one binary structure is unsupported in 1.2.0.
        print("Motor snapshot:", plc.read_tag("Motors[0]"))
        plc.write_tag("Motors[0].CommandSpeed", 1250)
        plc.write_tag("Motors[0].Description", "Infeed conveyor")

        # Built-in STRING capacity is DATA[82] = 82 UTF-8 bytes. Custom
        # string types use their configured DATA[N] capacity.


if __name__ == "__main__":
    main()
