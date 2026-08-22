using RustEtherNetIp;

string address = Environment.GetEnvironmentVariable("PLC_ADDRESS")
    ?? "192.168.0.10:44818";

using var plc = new EtherNetIpClient();
if (!plc.Connect(address))
    throw new InvalidOperationException(plc.LastConnectError);

// Read the whole structure when the application needs one logical snapshot.
// ReadUdtChunked also handles a UDT that is larger than one CIP response.
PlcValue snapshot = plc.ReadUdtChunked("Mixer");
Console.WriteLine($"Whole Mixer value: {snapshot}");

// Prefer typed member paths for ordinary reads, commands, and setpoints.
Console.WriteLine($"Speed = {plc.ReadReal("Mixer.SpeedFeedback")}");
Console.WriteLine($"Description = {plc.ReadString("Mixer.Description")}");
plc.WriteReal("Mixer.SpeedSetpoint", 60.0f);
plc.WriteBool("Mixer.Enabled", true);
plc.WriteString("Mixer.Description", "Primary mixer");

// UDT array elements use the same complete symbolic path. Whole-element reads
// work, but 1.2.0 does not support writing Motors[0] as one binary structure.
PlcValue motor = plc.ReadUdtChunked("Motors[0]");
Console.WriteLine($"Whole motor value: {motor}");
plc.WriteDint("Motors[0].CommandSpeed", 1250);
plc.WriteString("Motors[0].Description", "Infeed conveyor");

// Built-in Logix STRING uses DATA[82]: 82 UTF-8 bytes. Custom string types use
// their declared DATA[N] capacity; the library discovers the correct handle.
