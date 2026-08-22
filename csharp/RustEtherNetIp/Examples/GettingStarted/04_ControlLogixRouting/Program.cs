using RustEtherNetIp;

string moduleAddress = Environment.GetEnvironmentVariable("PLC_ADDRESS")
    ?? "192.168.0.20:44818";
byte cpuSlot = byte.Parse(Environment.GetEnvironmentVariable("PLC_SLOT") ?? "0");

var route = new RoutePath().AddSlot(cpuSlot);

using var plc = new EtherNetIpClient();
if (!plc.ConnectWithRoute(moduleAddress, route))
    throw new InvalidOperationException(plc.LastConnectError);

Console.WriteLine($"Connected through {moduleAddress} to CPU slot {cpuSlot}");
Console.WriteLine($"ProductionCount = {plc.ReadDint("ProductionCount")}");

// Ordered multi-hop example:
// var multiHop = new RoutePath()
//     .AddBackplane(port: 1, slot: 3)
//     .AddEthernet(port: 2, address: "192.168.10.20")
//     .AddBackplane(port: 1, slot: 0);
