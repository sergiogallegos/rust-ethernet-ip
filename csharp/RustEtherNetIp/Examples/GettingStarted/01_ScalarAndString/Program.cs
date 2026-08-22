using RustEtherNetIp;

string address = Environment.GetEnvironmentVariable("PLC_ADDRESS")
    ?? "192.168.0.10:44818";

using var plc = new EtherNetIpClient();
if (!plc.Connect(address))
    throw new InvalidOperationException(plc.LastConnectError);

try
{
    Console.WriteLine($"ProductionCount = {plc.ReadDint("ProductionCount")}");
    Console.WriteLine($"TankTemperature = {plc.ReadReal("TankTemperature")}");
    Console.WriteLine($"MachineRunning = {plc.ReadBool("MachineRunning")}");
    Console.WriteLine($"RecipeName = {plc.ReadString("RecipeName")}");

    // Use dedicated test tags before enabling writes on production equipment.
    plc.WriteDint("ProductionSetpoint", 1250);
    plc.WriteReal("TemperatureSetpoint", 72.5f);
    plc.WriteBool("EnableCommand", true);
    plc.WriteString("RecipeName", "PRODUCT_A");

    // 1.2.0 also supports built-in/custom STRING members by full tag path.
    plc.WriteString("Mixer.Description", "Primary mixer");
    plc.WriteString("Motors[0].Description", "Infeed conveyor");
}
catch (PlcException ex)
{
    Console.Error.WriteLine(ex.Message);
    Console.Error.WriteLine($"Native detail: {ex.NativeError}");
    throw;
}
