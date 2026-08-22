using RustEtherNetIp;

string address = Environment.GetEnvironmentVariable("PLC_ADDRESS")
    ?? "192.168.0.10:44818";

using var plc = new EtherNetIpClient();
if (!plc.Connect(address))
    throw new InvalidOperationException(plc.LastConnectError);

var reads = plc.ReadTagsBatch(new[]
{
    "ProductionCount",
    "TankTemperature",
    "Program:MainProgram.MachineRunning"
});

foreach (var (tag, result) in reads)
{
    if (result.Success)
        Console.WriteLine($"READ  {tag} = {result.Value}");
    else
        Console.Error.WriteLine($"READ  {tag}: {result.ErrorMessage}");
}

var writes = plc.WriteTagsBatch(new Dictionary<string, object>
{
    ["ProductionSetpoint"] = 1250,
    ["TemperatureSetpoint"] = 72.5f,
    ["EnableCommand"] = true,
    ["RecipeName"] = "PRODUCT_A"
});

foreach (var (tag, result) in writes)
    Console.WriteLine($"WRITE {tag}: {(result.Success ? "ok" : result.ErrorMessage)}");

var mixed = plc.ExecuteBatch(new[]
{
    BatchOperation.Read("ProductionCount"),
    BatchOperation.Write("ProductionSetpoint", 1300),
    BatchOperation.Read("Program:MainProgram.MachineRunning")
});

foreach (var result in mixed)
{
    string kind = result.IsWrite ? "WRITE" : "READ";
    Console.WriteLine($"{kind} {result.TagName}: " +
        (result.Success ? result.Value ?? "ok" : result.ErrorMessage));
}
