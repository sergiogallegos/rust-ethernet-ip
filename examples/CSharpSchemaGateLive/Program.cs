using RustEtherNetIp;

// Live companion runner for docs/validation/SCHEMA_CHANGE_GATE.md (C# leg).
//
// Automates the repeatable, non-editing steps of the schema-change
// validation procedure against a real controller: baseline capture,
// post-edit read/recovery observation, explicit RefreshSchema(),
// rediscovery, and restore-safe write verification. Every Studio 5000
// action stays manual and maintainer-controlled — this tool only pauses on
// stdin between phases and never issues a schema edit itself. Mirrors
// examples/schema_change_gate_live.rs (the Rust companion) phase for phase.
internal static class Program
{
    private static readonly int[] Indices = [5, 40];

    private sealed record Options(
        string Address,
        byte Slot,
        string ProgramName,
        string Tag,
        bool AllowWrites,
        bool DryRun);

    private static Options ParseOptions(string[] args)
    {
        string address = Environment.GetEnvironmentVariable("TEST_PLC_ADDRESS") ?? "192.168.0.1:44818";
        byte slot = byte.TryParse(Environment.GetEnvironmentVariable("TEST_PLC_SLOT"), out byte parsedSlot)
            ? parsedSlot
            : (byte)0;
        string program = Environment.GetEnvironmentVariable("TEST_PLC_PROGRAM") ?? "TestProgram";
        string tag = "gSchemaSwap";
        bool allowWrites = false;
        bool dryRun = false;

        for (int i = 0; i < args.Length; i++)
        {
            switch (args[i])
            {
                case "--plc-address": address = args[++i]; break;
                case "--plc-slot": slot = byte.Parse(args[++i]); break;
                case "--program": program = args[++i]; break;
                case "--tag": tag = args[++i]; break;
                case "--allow-writes": allowWrites = true; break;
                case "--dry-run": dryRun = true; break;
                default: throw new ArgumentException($"Unknown argument: {args[i]}");
            }
        }
        return new Options(address, slot, program, tag, allowWrites, dryRun);
    }

    private static void PauseForStudio5000(string message)
    {
        Console.WriteLine();
        Console.WriteLine("=== MAINTAINER ACTION REQUIRED ===");
        Console.WriteLine(message);
        Console.WriteLine("This tool never edits controller schema. Perform the Studio 5000 action now.");
        Console.Write("Press Enter once the change is downloaded and online: ");
        Console.ReadLine();
    }

    private static string Describe(PlcValue value) => value.Type switch
    {
        PlcValueType.Dint => $"Dint({value.As<int>()})",
        PlcValueType.Bool => $"Bool({value.As<bool>()})",
        PlcValueType.Real => $"Real({value.As<float>()})",
        _ => $"{value.Type}({value.Value})",
    };

    private static bool ValuesEqual(PlcValue a, PlcValue b) =>
        a.Type == b.Type && Equals(a.Value, b.Value);

    private static PlcValue ReadElement(EtherNetIpClient plc, string path)
    {
        TagReadResult result = plc.ReadTagWithDetails(path);
        if (!result.Success || result.Value is null)
            throw new InvalidOperationException($"{path}: read failed: {result.ErrorMessage}");
        return result.Value;
    }

    // Produces a distinguishable probe value of the same type, for a
    // restore-safe write/read-back check. Only the two shapes this gate
    // swaps between (DINT[] and packed BOOL[]) are supported.
    private static PlcValue Exercise(PlcValue value) => value.Type switch
    {
        PlcValueType.Dint => PlcValue.Dint(value.As<int>() == 123_456_789 ? 123_456_788 : 123_456_789),
        PlcValueType.Bool => PlcValue.Bool(!value.As<bool>()),
        _ => throw new InvalidOperationException(
            $"unsupported schema-swap element type for a write probe: {value.Type}"),
    };

    private static void WriteValue(EtherNetIpClient plc, string path, PlcValue value)
    {
        switch (value.Type)
        {
            case PlcValueType.Dint: plc.WriteDint(path, value.As<int>()); break;
            case PlcValueType.Bool: plc.WriteBool(path, value.As<bool>()); break;
            default: throw new InvalidOperationException(
                $"unsupported schema-swap element type for a write: {value.Type}");
        }
    }

    private static void WriteAndVerify(EtherNetIpClient plc, string path, PlcValue value)
    {
        WriteValue(plc, path, value);
        PlcValue readBack = ReadElement(plc, path);
        if (!ValuesEqual(readBack, value))
        {
            throw new InvalidOperationException(
                $"{path}: wrote {Describe(value)}, read back {Describe(readBack)}");
        }
    }

    private static void PrintMetricsDelta(string label, DiagnosticsSchemaCacheMetrics before, DiagnosticsSchemaCacheMetrics after)
    {
        Console.WriteLine($"  {label}:");
        Console.WriteLine($"    generation: {before.Generation} -> {after.Generation} ({(long)after.Generation - (long)before.Generation:+0;-0;+0})");
        Console.WriteLine($"    refreshes: {before.Refreshes} -> {after.Refreshes} ({(long)after.Refreshes - (long)before.Refreshes:+0;-0;+0})");
        Console.WriteLine(
            $"    array classification hits/misses/evictions: {before.ArrayClassificationHits}/{before.ArrayClassificationMisses}/{before.ArrayClassificationEvictions} -> " +
            $"{after.ArrayClassificationHits}/{after.ArrayClassificationMisses}/{after.ArrayClassificationEvictions}");
        Console.WriteLine($"    datatype contradictions: {before.DatatypeContradictions} -> {after.DatatypeContradictions} ({(long)after.DatatypeContradictions - (long)before.DatatypeContradictions:+0;-0;+0})");
        Console.WriteLine(
            $"    read recoveries succeeded/failed: {before.SuccessfulReadRecoveries}/{before.FailedReadRecoveries} -> " +
            $"{after.SuccessfulReadRecoveries}/{after.FailedReadRecoveries}");
    }

    private static int Run(Options options)
    {
        Console.WriteLine("Schema-change live gate companion (C#)");
        Console.WriteLine(
            $"target={options.Address} slot={options.Slot} program={options.ProgramName} " +
            $"tag={options.Tag} allow_writes={options.AllowWrites}");
        Console.WriteLine("This tool never edits controller schema; every Studio 5000 action stays manual.");

        if (options.DryRun)
        {
            Console.WriteLine(
                $"would-test scopes=controller,program indices=[{string.Join(", ", Indices)}] allow_writes={options.AllowWrites}");
            return 0;
        }
        if (!options.AllowWrites)
        {
            throw new InvalidOperationException(
                "Live mode requires --allow-writes; dedicated gSchemaSwap elements will be changed and restored");
        }

        using var plc = new EtherNetIpClient();
        if (!plc.ConnectWithRoute(options.Address, new RoutePath().AddSlot(options.Slot)))
            throw new InvalidOperationException(plc.LastConnectError ?? "Connection failed");
        Console.WriteLine($"Phase 0 — connected; healthy={plc.CheckHealth()}");

        var scopes = new (string Name, string Base)[]
        {
            ("controller", options.Tag),
            ("program", $"Program:{options.ProgramName}.{options.Tag}"),
        };

        DiagnosticsSchemaCacheMetrics baselineMetrics = plc.GetDiagnosticsSnapshot().SchemaCache;
        Console.WriteLine(
            $"Phase 1 — baseline schema_cache_metrics: generation={baselineMetrics.Generation} refreshes={baselineMetrics.Refreshes}");

        Console.WriteLine("Phase 2 — pre-edit reads (twice, to warm classification cache)");
        var preEditValues = new List<(string Path, PlcValue Value)>();
        foreach (var (scopeName, basePath) in scopes)
        {
            foreach (int index in Indices)
            {
                string path = $"{basePath}[{index}]";
                PlcValue first = ReadElement(plc, path);
                PlcValue second = ReadElement(plc, path);
                if (!ValuesEqual(first, second))
                {
                    throw new InvalidOperationException(
                        $"{path}: unstable read before any edit: {Describe(first)} then {Describe(second)}");
                }
                Console.WriteLine($"  {scopeName} {path} = {Describe(second)}");
                preEditValues.Add((path, second));
            }
        }

        Console.WriteLine("Phase 3 — restore-safe pre-edit write smoke check");
        foreach (var (path, original) in preEditValues)
        {
            PlcValue probe = Exercise(original);
            WriteAndVerify(plc, path, probe);
            WriteAndVerify(plc, path, original);
            Console.WriteLine($"  {path}: exercised and restored to {Describe(original)}");
        }

        PauseForStudio5000(
            $"Move any test-only references off '{options.Tag}', delete the unused original, and rename " +
            $"the replacement to '{options.Tag}' — for both controller and program scope.");

        Console.WriteLine("Phase 4 — post-edit reads without calling RefreshSchema() first");
        DiagnosticsSchemaCacheMetrics preRefreshMetrics = plc.GetDiagnosticsSnapshot().SchemaCache;
        var postEditValues = new List<(string Path, PlcValue Value)>();
        foreach (var (scopeName, basePath) in scopes)
        {
            foreach (int index in Indices)
            {
                string path = $"{basePath}[{index}]";
                try
                {
                    PlcValue value = ReadElement(plc, path);
                    Console.WriteLine(
                        $"  {scopeName} {path} = {Describe(value)} (automatic recovery applies if the type changed)");
                    postEditValues.Add((path, value));
                }
                catch (Exception error)
                {
                    Console.WriteLine($"  {scopeName} {path}: read error before refresh: {error.Message}");
                }
            }
        }
        DiagnosticsSchemaCacheMetrics postReadMetrics = plc.GetDiagnosticsSnapshot().SchemaCache;
        PrintMetricsDelta("automatic recovery (no explicit refresh yet)", preRefreshMetrics, postReadMetrics);

        Console.WriteLine("Phase 5 — explicit RefreshSchema()");
        plc.RefreshSchema();
        DiagnosticsSchemaCacheMetrics postRefreshMetrics = plc.GetDiagnosticsSnapshot().SchemaCache;
        if (postRefreshMetrics.Generation != preRefreshMetrics.Generation + 1 ||
            postRefreshMetrics.Refreshes != preRefreshMetrics.Refreshes + 1)
        {
            throw new InvalidOperationException(
                "RefreshSchema() did not advance generation/refresh count by exactly one: " +
                $"before=(gen={preRefreshMetrics.Generation}, refreshes={preRefreshMetrics.Refreshes}) " +
                $"after=(gen={postRefreshMetrics.Generation}, refreshes={postRefreshMetrics.Refreshes})");
        }
        Console.WriteLine($"  generation now {postRefreshMetrics.Generation}");

        Console.WriteLine("Phase 6 — rediscovery");
        try
        {
            List<TagAttributes> tags = plc.DiscoverTagsDetailed();
            int matches = tags.Count(t => t.Name == options.Tag);
            Console.WriteLine($"  controller discovery: {tags.Count} tags, {matches} match '{options.Tag}'");
        }
        catch (Exception error)
        {
            Console.WriteLine($"  controller discovery failed (non-fatal): {error.Message}");
        }
        Console.WriteLine("  program discovery: N/A (not exposed by the C# 1.2.x wrapper)");

        Console.WriteLine("Phase 7 — post-refresh reads");
        var postRefreshValues = new List<(string Path, PlcValue Value)>();
        foreach (var (scopeName, basePath) in scopes)
        {
            foreach (int index in Indices)
            {
                string path = $"{basePath}[{index}]";
                PlcValue value = ReadElement(plc, path);
                Console.WriteLine($"  {scopeName} {path} = {Describe(value)}");
                postRefreshValues.Add((path, value));
            }
        }

        Console.WriteLine("Phase 8 — restore-safe post-refresh write/verify");
        foreach (var (path, current) in postRefreshValues)
        {
            PlcValue probe = Exercise(current);
            WriteAndVerify(plc, path, probe);
            WriteAndVerify(plc, path, current);
            Console.WriteLine($"  {path}: exercised the new addressing shape and restored to {Describe(current)}");
        }

        DiagnosticsSchemaCacheMetrics finalMetrics = plc.GetDiagnosticsSnapshot().SchemaCache;
        Console.WriteLine();
        Console.WriteLine("=== Paste into the dated validation record ===");
        Console.WriteLine(
            $"session survived: yes (single connection held for the entire run; healthy={plc.CheckHealth()})");
        PrintMetricsDelta("baseline -> final", baselineMetrics, finalMetrics);
        Console.WriteLine("C#: PASS");
        return 0;
    }

    private static int Main(string[] args)
    {
        try
        {
            return Run(ParseOptions(args));
        }
        catch (Exception error)
        {
            Console.Error.WriteLine($"FAIL: {error.Message}");
            return 1;
        }
    }
}
