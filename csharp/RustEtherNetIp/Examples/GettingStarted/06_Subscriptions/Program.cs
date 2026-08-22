using RustEtherNetIp;

string address = Environment.GetEnvironmentVariable("PLC_ADDRESS")
    ?? "192.168.0.10:44818";

using var plc = new EtherNetIpClient();
if (!plc.Connect(address))
    throw new InvalidOperationException(plc.LastConnectError);

var subscription = plc.SubscribeToTag(
    "ProductionCount",
    new SubscriptionOptions(pollIntervalMs: 250));

subscription.ValueChanged += (_, change) =>
    Console.WriteLine($"{change.TagName}: {change.OldValue} -> {change.NewValue}");

plc.UpsertTagGroup(
    "line-status",
    new[] { "ProductionCount", "TankTemperature", "MachineRunning" },
    updateRateMs: 500);

TagGroupSnapshot snapshot = plc.ReadTagGroupOnce("line-status");
foreach (var (tag, value) in snapshot.Values)
    Console.WriteLine($"{tag} = {value}");
foreach (var (tag, error) in snapshot.Errors)
    Console.Error.WriteLine($"{tag}: {error}");

Console.WriteLine("Polling for 30 seconds. Press Ctrl+C to stop early.");
await Task.Delay(TimeSpan.FromSeconds(30));
plc.UnsubscribeFromTag("ProductionCount");
