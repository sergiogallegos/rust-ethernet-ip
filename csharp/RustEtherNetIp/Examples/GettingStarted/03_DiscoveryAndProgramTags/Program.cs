using RustEtherNetIp;

string address = Environment.GetEnvironmentVariable("PLC_ADDRESS")
    ?? "192.168.0.10:44818";
string program = Environment.GetEnvironmentVariable("PLC_PROGRAM")
    ?? "MainProgram";

using var plc = new EtherNetIpClient();
if (!plc.Connect(address))
    throw new InvalidOperationException(plc.LastConnectError);

Console.WriteLine("Controller-scoped tags:");
foreach (var tag in plc.DiscoverTagsDetailed().Take(50))
    Console.WriteLine($"{tag.Name,-40} {tag.DataTypeName,-10} {tag.Size,6} bytes");

TagAttributes controllerAttributes = plc.GetTagAttributes("ProductionCount");
Console.WriteLine($"Controller tag: {controllerAttributes}");

// Program tags use their complete symbolic path. The 1.2.0 C# wrapper can
// read/write known program paths but does not expose program enumeration yet.
string programTag = $"Program:{program}.ProductionCount";
Console.WriteLine($"{programTag} = {plc.ReadDint(programTag)}");
Console.WriteLine($"Program tag attributes: {plc.GetTagAttributes(programTag)}");
