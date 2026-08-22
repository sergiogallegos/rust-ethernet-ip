using RustEtherNetIp;

internal static class Program
{
    private static readonly string[] BatchReadTags =
    [
        "gTestArray_DINT[0]",
        "gTestArray_INT[0]",
        "gTestArray_REAL[0]",
        "gTestArray_BOOL[0]",
        "gTest_STRING",
        "gTestUDT.Member1_DINT",
        "Program:TestProgram.gTestArray_DINT[0]",
        "Program:TestProgram.gTestUDT.Member1_DINT",
        "Program:TestProgram.gTestArray_REAL[0]",
        "Program:TestProgram.gTestArray_BOOL[0]",
    ];

    private static readonly string[] BatchWriteTags =
    [
        "gTestArray_DINT[50]",
        "gTestArray_DINT[51]",
        "Program:TestProgram.gTestArray_DINT[50]",
        "Program:TestProgram.gTestArray_DINT[51]",
    ];

    private static readonly string[] WholeUdtTags =
    [
        "gTestUDT",
        "gTestUDT_Array[0]",
        "Program:TestProgram.gTestUDT",
        "Program:TestProgram.gTestUDT_Array[0]",
    ];

    private sealed record Options(string Address, byte Slot, string ProgramName, bool DryRun, bool AllowWrites);

    private static Options ParseOptions(string[] args)
    {
        string address = Environment.GetEnvironmentVariable("TEST_PLC_ADDRESS") ?? "192.168.0.1:44818";
        byte slot = byte.TryParse(Environment.GetEnvironmentVariable("TEST_PLC_SLOT"), out byte parsedSlot)
            ? parsedSlot
            : (byte)0;
        string program = Environment.GetEnvironmentVariable("TEST_PLC_PROGRAM") ?? "TestProgram";
        bool dryRun = false;
        bool allowWrites = false;

        for (int i = 0; i < args.Length; i++)
        {
            switch (args[i])
            {
                case "--plc-address": address = args[++i]; break;
                case "--plc-slot": slot = byte.Parse(args[++i]); break;
                case "--program": program = args[++i]; break;
                case "--dry-run": dryRun = true; break;
                case "--allow-writes": allowWrites = true; break;
                default: throw new ArgumentException($"Unknown argument: {args[i]}");
            }
        }
        return new Options(address, slot, program, dryRun, allowWrites);
    }

    private static string WithProgram(string tag, string program) => tag.Replace("TestProgram", program);

    private static int ExerciseValue(int original) => original == 123_456_789 ? 123_456_788 : 123_456_789;

    private static void RequireBatchWrites(
        EtherNetIpClient plc,
        IReadOnlyDictionary<string, int> values)
    {
        var request = values.ToDictionary(item => item.Key, item => (object)item.Value);
        var results = plc.WriteTagsBatch(request);
        var failures = results.Values.Where(result => !result.Success).ToArray();
        if (failures.Length != 0)
        {
            throw new InvalidOperationException("Batch write failures: " + string.Join(
                "; ", failures.Select(result => $"{result.TagName}: {result.ErrorMessage}")));
        }
    }

    private static void VerifyDints(EtherNetIpClient plc, IReadOnlyDictionary<string, int> expected)
    {
        foreach (var (tag, value) in expected)
        {
            int actual = plc.ReadDint(tag);
            if (actual != value)
                throw new InvalidOperationException($"{tag}: expected {value}, received {actual}");
        }
    }

    private static int Main(string[] args)
    {
        try
        {
            Options options = ParseOptions(args);
            Console.WriteLine("C# companion feature gate");
            Console.WriteLine($"target={options.Address} slot={options.Slot} program={options.ProgramName}");
            Console.WriteLine("writes are limited to four DINT test elements and are restored");

            if (options.DryRun)
            {
                Console.WriteLine(
                    $"would-test binding=csharp batch_read={BatchReadTags.Length} " +
                    $"batch_write={BatchWriteTags.Length} whole_udt={WholeUdtTags.Length} " +
                    "controller_discovery=yes program_discovery=N/A restore=yes");
                return 0;
            }
            if (!options.AllowWrites)
            {
                throw new InvalidOperationException(
                    "Live mode requires --allow-writes; four dedicated DINT test elements will be changed and restored");
            }

            using var plc = new EtherNetIpClient();
            if (!plc.ConnectWithRoute(options.Address, new RoutePath().AddSlot(options.Slot)))
                throw new InvalidOperationException(plc.LastConnectError ?? "Connection failed");

            Console.WriteLine("Phase 1 — controller discovery");
            var discovered = plc.DiscoverTagsDetailed();
            if (!discovered.Any(tag => tag.Name == "gTestUDT"))
                throw new InvalidOperationException("Controller discovery did not return gTestUDT");
            Console.WriteLine($"  PASS: {discovered.Count} controller tags");
            Console.WriteLine("Phase 2 — program discovery: N/A (not exposed by the C# 1.2.x wrapper)");

            Console.WriteLine("Phase 3 — whole UDT reads");
            foreach (string template in WholeUdtTags)
            {
                string tag = WithProgram(template, options.ProgramName);
                PlcValue value = plc.ReadUdtChunked(tag);
                int payloadBytes = value.UdtData?.Data.Length ?? 0;
                if (!value.IsUdt || payloadBytes == 0)
                    throw new InvalidOperationException($"{tag}: empty or non-UDT result");
                Console.WriteLine($"  PASS: {tag} ({payloadBytes} payload bytes)");
            }

            Console.WriteLine("Phase 4 — mixed-type batch read");
            string[] readTags = BatchReadTags.Select(tag => WithProgram(tag, options.ProgramName)).ToArray();
            var reads = plc.ReadTagsBatch(readTags);
            var readFailures = reads.Values.Where(result => !result.Success).ToArray();
            if (readFailures.Length != 0)
            {
                throw new InvalidOperationException("Batch read failures: " + string.Join(
                    "; ", readFailures.Select(result => $"{result.TagName}: {result.ErrorMessage}")));
            }
            Console.WriteLine($"  PASS: ReadTagsBatch {reads.Count}/{readTags.Length}");

            // ReadTagsBatch may retry individual failures for consumer resilience.
            // ExecuteBatch proves the native multi-service path independently.
            var nativeReads = plc.ExecuteBatch(readTags.Select(BatchOperation.Read).ToArray());
            var nativeReadFailures = nativeReads.Where(result => !result.Success).ToArray();
            if (nativeReadFailures.Length != 0)
            {
                throw new InvalidOperationException("Native execute-batch read failures: " + string.Join(
                    "; ", nativeReadFailures.Select(result => $"{result.TagName}: {result.ErrorMessage}")));
            }
            Console.WriteLine($"  PASS: native ExecuteBatch {nativeReads.Length}/{readTags.Length}");

            Console.WriteLine("Phase 5 — restore-safe batch write");
            string[] writeTags = BatchWriteTags.Select(tag => WithProgram(tag, options.ProgramName)).ToArray();
            var originals = writeTags.ToDictionary(tag => tag, plc.ReadDint);
            var exercise = originals.ToDictionary(item => item.Key, item => ExerciseValue(item.Value));
            Exception? exerciseError = null;
            try
            {
                RequireBatchWrites(plc, exercise);
                VerifyDints(plc, exercise);
            }
            catch (Exception error)
            {
                exerciseError = error;
            }

            try
            {
                RequireBatchWrites(plc, originals);
                VerifyDints(plc, originals);
                Console.WriteLine($"  RESTORED: {originals.Count}/{originals.Count}");
            }
            catch (Exception restoreError)
            {
                throw new InvalidOperationException($"RESTORE FAILED: {restoreError.Message}", restoreError);
            }

            if (exerciseError is not null)
                throw exerciseError;
            Console.WriteLine("  PASS: batch write and read-back");
            Console.WriteLine("PASS: C# companion gate completed with original write values restored");
            return 0;
        }
        catch (Exception error)
        {
            Console.Error.WriteLine($"FAIL: {error.Message}");
            return 1;
        }
    }
}
