using RustEtherNetIp;

string address = Environment.GetEnvironmentVariable("PLC_ADDRESS")
    ?? "192.168.0.10:44818";

using var plc = new EtherNetIpClient();
if (!plc.Connect(address))
    throw new InvalidOperationException(plc.LastConnectError);

_ = plc.ReadDint("ProductionCount");
bool healthy = plc.CheckHealth();
DiagnosticsSnapshot snapshot = plc.GetDiagnosticsSnapshotDetailed();

Console.WriteLine($"Healthy: {healthy}");
Console.WriteLine($"Reads: {snapshot.Operations.TotalReads}");
Console.WriteLine($"Failed reads: {snapshot.Operations.FailedReads}");
Console.WriteLine($"Average read latency: {snapshot.Performance.AvgReadLatencyMs:F2} ms");
Console.WriteLine($"Maximum read latency: {snapshot.Performance.MaxReadLatencyMs:F2} ms");
Console.WriteLine($"Last error category: {snapshot.Errors.LastErrorCategory}");
Console.WriteLine($"Last error: {snapshot.Errors.LastErrorMessage}");
