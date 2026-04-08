using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text.Json;
using RustEtherNetIp;

namespace CSharpWrapperBenchmark
{
    internal record Metric(
        string Name,
        int Iterations,
        int LogicalOps,
        double ElapsedMs,
        double AvgCallMs,
        double OpsPerSec
    );

    internal record PerfRun(
        string Address,
        int Iterations,
        List<Metric> Metrics
    );

    internal static class Program
    {
        private static Metric BuildMetric(string name, int iterations, int logicalOps, Stopwatch stopwatch)
        {
            var elapsedMs = stopwatch.Elapsed.TotalMilliseconds;
            return new Metric(
                name,
                iterations,
                logicalOps,
                elapsedMs,
                elapsedMs / iterations,
                logicalOps / stopwatch.Elapsed.TotalSeconds
            );
        }

        private static int ParseIterations(string[] args)
        {
            for (int i = 0; i < args.Length - 1; i++)
            {
                if (args[i] == "--iterations" && int.TryParse(args[i + 1], out int parsed))
                    return Math.Max(parsed, 1);
            }
            return 100;
        }

        private static void Main(string[] args)
        {
            var iterations = ParseIterations(args);
            var address = Environment.GetEnvironmentVariable("TEST_PLC_ADDRESS") ?? "192.168.0.1:44818";
            var slot = byte.TryParse(Environment.GetEnvironmentVariable("TEST_PLC_SLOT"), out var parsedSlot)
                ? parsedSlot
                : (byte)0;
            var route = new RoutePath().AddSlot(slot);

            using var client = new EtherNetIpClient();
            if (!client.ConnectWithRoute(address, route))
                throw new Exception($"Failed to connect to PLC at {address}");

            var singleReadTag = "gTestArray_DINT[0]";
            var batchReadTags = new[]
            {
                "gTestArray_DINT[0]",
                "gTestArray_DINT[1]",
                "gTestArray_DINT[2]",
                "gTestArray_DINT[3]",
                "gTestArray_DINT[4]",
                "gTestArray_REAL[0]",
                "gTestArray_REAL[1]",
                "gTestArray_BOOL[0]",
                "gTestArray_INT[0]",
                "gTestUDT.Member1_DINT"
            };

            var batchWriteTags = new[]
            {
                "gTestArray_DINT[5]",
                "gTestArray_DINT[6]",
                "gTestArray_DINT[7]"
            };

            var originalValues = batchWriteTags.ToDictionary(tag => tag, tag => client.ReadDint(tag));
            try
            {
                _ = client.ReadDint(singleReadTag);
                _ = client.ReadTagsBatch(batchReadTags);

                var singleReadSw = Stopwatch.StartNew();
                for (int i = 0; i < iterations; i++)
                    _ = client.ReadDint(singleReadTag);
                singleReadSw.Stop();
                var singleRead = BuildMetric("single_read", iterations, iterations, singleReadSw);

                var singleWriteSw = Stopwatch.StartNew();
                for (int i = 0; i < iterations; i++)
                    client.WriteDint(batchWriteTags[0], 10_000 + i);
                singleWriteSw.Stop();
                var singleWrite = BuildMetric("single_write", iterations, iterations, singleWriteSw);

                var batchReadSw = Stopwatch.StartNew();
                for (int i = 0; i < iterations; i++)
                {
                    var results = client.ReadTagsBatch(batchReadTags);
                    if (results.Count != batchReadTags.Length)
                        throw new Exception("Batch read returned incomplete results");
                }
                batchReadSw.Stop();
                var batchRead = BuildMetric("batch_read", iterations, iterations * batchReadTags.Length, batchReadSw);

                var batchWriteSw = Stopwatch.StartNew();
                for (int i = 0; i < iterations; i++)
                {
                    var writes = new Dictionary<string, object>
                    {
                        [batchWriteTags[0]] = 20_000 + i,
                        [batchWriteTags[1]] = 30_000 + i,
                        [batchWriteTags[2]] = 40_000 + i
                    };
                    var results = client.WriteTagsBatch(writes);
                    if (results.Values.Any(r => !r.Success))
                        throw new Exception("Batch write returned a failed result");
                }
                batchWriteSw.Stop();
                var batchWrite = BuildMetric("batch_write", iterations, iterations * batchWriteTags.Length, batchWriteSw);

                var mixedOperations = new[]
                {
                    BatchOperation.Read("gTestArray_DINT[0]"),
                    BatchOperation.Write("gTestArray_DINT[5]", 50_000),
                    BatchOperation.Read("gTestArray_REAL[0]"),
                    BatchOperation.Read("gTestUDT.Member1_DINT")
                };
                var mixedSw = Stopwatch.StartNew();
                for (int i = 0; i < iterations; i++)
                {
                    var results = client.ExecuteBatch(mixedOperations);
                    if (results.Any(r => !r.Success))
                        throw new Exception("Mixed execute returned a failed result");
                }
                mixedSw.Stop();
                var mixedExecute = BuildMetric("mixed_execute", iterations, iterations * mixedOperations.Length, mixedSw);

                var run = new PerfRun(
                    address,
                    iterations,
                    new List<Metric> { singleRead, singleWrite, batchRead, batchWrite, mixedExecute }
                );

                Console.WriteLine(JsonSerializer.Serialize(run));
            }
            finally
            {
                client.WriteTagsBatch(originalValues.ToDictionary(kvp => kvp.Key, kvp => (object)kvp.Value));
            }
        }
    }
}
