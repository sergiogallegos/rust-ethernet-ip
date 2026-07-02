using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text.Json;
using RustEtherNetIp;

namespace CSharpFullCoverage;

internal enum Kind { Dint, Int, Real, Bool, String, Udt }
internal enum WriteMode
{
    Writeable,
    ReadOnly,
    FirmwareBlockedUdtStringMember,
    FirmwareBlockedUdtArrayElementMember,
    ServiceLayerWriteable,
}

internal readonly record struct Tag(string Name, string Category, Kind Kind, WriteMode Mode);

internal sealed class CatStats
{
    public int ReadOk, ReadFail, WriteOk, WriteFail, VerifyOk, VerifyFail, BlockedAsExpected, BlockedUnexpected;
}

internal static class Program
{
    private static List<Tag> BuildTags(string manifestPath)
    {
        using var document = JsonDocument.Parse(File.ReadAllText(manifestPath));
        var tags = new List<Tag>(2400);
        foreach (var category in document.RootElement.GetProperty("categories").EnumerateArray())
        {
            ExpandCategory(category, tags);
        }
        return tags;
    }

    private static void ExpandCategory(JsonElement category, List<Tag> tags)
    {
        var name = category.GetProperty("name").GetString()!;
        var pattern = category.GetProperty("pattern").GetString()!;
        if (category.TryGetProperty("members", out var members))
        {
            foreach (var i in RangeOrOnce(category, "indices"))
            {
                foreach (var member in members.EnumerateObject())
                {
                    var spec = member.Value;
                    tags.Add(new Tag(
                        RenderPattern(pattern, i, member.Name, null, null),
                        name,
                        ParseKind(spec.GetProperty("kind").GetString()!),
                        ParseWriteMode(spec.GetProperty("writeability").GetString()!)));
                }
            }
            return;
        }
        if (category.TryGetProperty("inner", out var inner))
        {
            foreach (var i in RangeOrOnce(category, "outer_indices"))
            {
                foreach (var field in inner.EnumerateObject())
                {
                    var spec = field.Value;
                    foreach (var j in Range(spec.GetProperty("range")))
                    {
                        tags.Add(new Tag(
                            RenderPattern(pattern, i, null, field.Name, j),
                            name,
                            ParseKind(spec.GetProperty("kind").GetString()!),
                            ParseWriteMode(spec.GetProperty("writeability").GetString()!)));
                    }
                }
            }
            return;
        }
        foreach (var i in RangeOrOnce(category, "indices"))
        {
            tags.Add(new Tag(
                RenderPattern(pattern, i, null, null, null),
                name,
                ParseKind(category.GetProperty("kind").GetString()!),
                ParseWriteMode(category.GetProperty("writeability").GetString()!)));
        }
    }

    private static IEnumerable<int> RangeOrOnce(JsonElement category, string property)
    {
        if (!category.TryGetProperty(property, out var indices)) return new[] { 0 };
        return Range(indices.GetProperty("range"));
    }

    private static IEnumerable<int> Range(JsonElement range)
    {
        var start = range[0].GetInt32();
        var end = range[1].GetInt32();
        return Enumerable.Range(start, end - start);
    }

    private static string RenderPattern(string pattern, int? i, string? member, string? field, int? j)
    {
        var output = pattern;
        if (i.HasValue) output = output.Replace("{i}", i.Value.ToString());
        if (member is not null) output = output.Replace("{member}", member);
        if (field is not null) output = output.Replace("{field}", field);
        if (j.HasValue) output = output.Replace("{j}", j.Value.ToString());
        return output;
    }

    private static Kind ParseKind(string value) => value switch
    {
        "Dint" => Kind.Dint,
        "Int" => Kind.Int,
        "Real" => Kind.Real,
        "Bool" => Kind.Bool,
        "String" => Kind.String,
        "Udt" => Kind.Udt,
        _ => throw new ArgumentException($"unknown kind: {value}"),
    };

    private static WriteMode ParseWriteMode(string value) => value switch
    {
        "writeable" => WriteMode.Writeable,
        "read_only" => WriteMode.ReadOnly,
        "firmware_blocked_udt_string_member" => WriteMode.FirmwareBlockedUdtStringMember,
        "firmware_blocked_udt_array_element_member" => WriteMode.FirmwareBlockedUdtArrayElementMember,
        "service_layer_writeable" => WriteMode.ServiceLayerWriteable,
        _ => throw new ArgumentException($"unknown writeability: {value}"),
    };

    private static bool IsWriteable(WriteMode mode) =>
        mode is WriteMode.Writeable or WriteMode.ServiceLayerWriteable;

    private static bool IsFirmwareBlocked(WriteMode mode) =>
        mode is WriteMode.FirmwareBlockedUdtStringMember
            or WriteMode.FirmwareBlockedUdtArrayElementMember;

    private static object? Rand(Kind k, Random rng) => k switch
    {
        Kind.Dint => rng.Next(1_000, 900_000),
        Kind.Int  => (short)rng.Next(100, 20_000),
        Kind.Real => (float)(rng.NextDouble() * 9000 + 1.0),
        Kind.Bool => rng.Next(2) == 1,
        _ => null,
    };

    private static object? Nines(Kind k) => k switch
    {
        Kind.Dint => 999_999,
        Kind.Int  => (short)9_999,
        Kind.Real => 99.99f,
        Kind.Bool => true,
        _ => null,
    };

    private static bool DoRead(EtherNetIpClient c, Tag t)
    {
        try
        {
            switch (t.Kind)
            {
                case Kind.Dint:   c.ReadDint(t.Name); return true;
                case Kind.Int:    c.ReadInt(t.Name); return true;
                case Kind.Real:   c.ReadReal(t.Name); return true;
                case Kind.Bool:   c.ReadBool(t.Name); return true;
                case Kind.String: c.ReadString(t.Name); return true;
                case Kind.Udt:    c.ReadUdt(t.Name); return true;
            }
        }
        catch { return false; }
        return false;
    }

    private static bool DoWrite(EtherNetIpClient c, Tag t, object v)
    {
        try
        {
            switch (t.Kind)
            {
                case Kind.Dint: c.WriteDint(t.Name, (int)v); return true;
                case Kind.Int:  c.WriteInt(t.Name, (short)v); return true;
                case Kind.Real: c.WriteReal(t.Name, (float)v); return true;
                case Kind.Bool: c.WriteBool(t.Name, (bool)v); return true;
            }
        }
        catch { return false; }
        return false;
    }

    private static bool VerifyRead(EtherNetIpClient c, Tag t, object expected)
    {
        try
        {
            return t.Kind switch
            {
                Kind.Dint => c.ReadDint(t.Name) == (int)expected,
                Kind.Int  => c.ReadInt(t.Name)  == (short)expected,
                Kind.Bool => c.ReadBool(t.Name) == (bool)expected,
                Kind.Real => Math.Abs(c.ReadReal(t.Name) - (float)expected) < 0.001f,
                _ => false,
            };
        }
        catch { return false; }
    }

    private static List<(string Category, Tag Tag, object Expected)> SettleSamples() => new()
    {
        ("ctrl.BOOL_array", new("gTestArray_BOOL[5]", "ctrl.BOOL_array", Kind.Bool, WriteMode.Writeable), true),
        ("ctrl.DINT_array", new("gTestArray_DINT[42]", "ctrl.DINT_array", Kind.Dint, WriteMode.Writeable), 999_999),
        ("ctrl.INT_array", new("gTestArray_INT[100]", "ctrl.INT_array", Kind.Int, WriteMode.Writeable), (short)9_999),
        ("ctrl.Large_DINT", new("gTestArray_Large[500]", "ctrl.Large_DINT", Kind.Dint, WriteMode.Writeable), 999_999),
        ("ctrl.REAL_array", new("gTestArray_REAL[10]", "ctrl.REAL_array", Kind.Real, WriteMode.Writeable), 99.99f),
        ("ctrl.UDT_members", new("gTestUDT.Member1_DINT", "ctrl.UDT_members", Kind.Dint, WriteMode.Writeable), 999_999),
        ("ctrl.UDT_nested", new("gTestUDT.Array_DINT[5]", "ctrl.UDT_nested", Kind.Dint, WriteMode.Writeable), 999_999),
        ("ctrl.UDTarr_elem_nested", new("gTestUDT_Array[2].Array_DINT[3]", "ctrl.UDTarr_elem_nested", Kind.Dint, WriteMode.Writeable), 999_999),
        ("prog.BOOL_array", new("Program:TestProgram.gTestArray_BOOL[5]", "prog.BOOL_array", Kind.Bool, WriteMode.Writeable), true),
        ("prog.DINT_array", new("Program:TestProgram.gTestArray_DINT[42]", "prog.DINT_array", Kind.Dint, WriteMode.Writeable), 999_999),
        ("prog.REAL_array", new("Program:TestProgram.gTestArray_REAL[10]", "prog.REAL_array", Kind.Real, WriteMode.Writeable), 99.99f),
        ("prog.UDT_members", new("Program:TestProgram.gTestUDT.Member1_DINT", "prog.UDT_members", Kind.Dint, WriteMode.Writeable), 999_999),
        ("prog.UDT_nested", new("Program:TestProgram.gTestUDT.Array_DINT[5]", "prog.UDT_nested", Kind.Dint, WriteMode.Writeable), 999_999),
        ("prog.UDTarr_elem_nested", new("Program:TestProgram.gTestUDT_Array[2].Array_DINT[3]", "prog.UDTarr_elem_nested", Kind.Dint, WriteMode.Writeable), 999_999),
    };

    private static int Main()
    {
        var address = Environment.GetEnvironmentVariable("TEST_PLC_ADDRESS") ?? "192.168.0.1:44818";
        var slot = byte.TryParse(Environment.GetEnvironmentVariable("TEST_PLC_SLOT"), out var s) ? s : (byte)0;
        var manifestPath = Path.Combine(AppContext.BaseDirectory, "full_coverage_tags.json");
        var outDir = "examples/full_coverage_results";
        var dryRun = false;
        var skipPreflight = false;
        var argv = Environment.GetCommandLineArgs();
        for (int i = 1; i < argv.Length; i++)
        {
            switch (argv[i])
            {
                case "--plc-address": address = argv[++i]; break;
                case "--plc-slot": slot = byte.Parse(argv[++i]); break;
                case "--manifest": manifestPath = argv[++i]; break;
                case "--out-dir": outDir = argv[++i]; break;
                case "--dry-run": dryRun = true; break;
                case "--skip-preflight": skipPreflight = true; break;
            }
        }
        List<Tag> tags;
        try
        {
            tags = BuildTags(manifestPath);
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or JsonException or ArgumentException or KeyNotFoundException)
        {
            Console.Error.WriteLine($"manifest-error: failed to load {manifestPath}: {ex.Message}");
            return 2;
        }
        var rng = new Random();
        var stats = new SortedDictionary<string, CatStats>();
        var writtenValues = new List<(Tag, object)>(1900);
        int writeable = 0, blocked = 0, readonly_ = 0;
        foreach (var t in tags)
        {
            if (IsWriteable(t.Mode)) writeable++;
            else if (IsFirmwareBlocked(t.Mode)) blocked++;
            else readonly_++;
        }

        Console.WriteLine("C# wrapper — full-coverage exerciser");
        Console.WriteLine($"PLC: {address} (slot {slot})  total tags: {tags.Count}");
        Console.WriteLine($"  writeable: {writeable}   firmware-blocked: {blocked}   read-only: {readonly_}");
        Console.WriteLine();

        if (dryRun)
        {
            Console.WriteLine($"would-test binding=csharp tags={tags.Count} writeable={writeable} blocked={blocked} read_only={readonly_}");
            return 0;
        }

        using var client = new EtherNetIpClient();
        if (!client.ConnectWithRoute(address, new RoutePath().AddSlot(slot)))
        {
            Console.Error.WriteLine("Connect failed"); return 2;
        }

        CatStats S(string c) => stats.TryGetValue(c, out var v) ? v : (stats[c] = new CatStats());

        int preflightOk = 0, preflightFail = 0;
        if (!skipPreflight)
        {
            Console.WriteLine("Phase 0 — preflight tag inventory");
            var preflightSw = Stopwatch.StartNew();
            foreach (var t in tags)
            {
                if (DoRead(client, t)) preflightOk++;
                else
                {
                    preflightFail++;
                    Console.Error.WriteLine($"setup-error: tag {t.Name} failed preflight — verify the PLC project against docs/PLC_TEST_TAG_DEFINITIONS.md");
                }
            }
            preflightSw.Stop(); Console.WriteLine($"  done in {preflightSw.Elapsed.TotalSeconds:F1}s  preflight={preflightOk}/{preflightOk + preflightFail}");
            if (preflightFail > 0) return 2;
        }

        Console.WriteLine("Phase 1 — read every tag");
        var sw = Stopwatch.StartNew();
        foreach (var t in tags) { if (DoRead(client, t)) S(t.Category).ReadOk++; else S(t.Category).ReadFail++; }
        sw.Stop(); Console.WriteLine($"  done in {sw.Elapsed.TotalSeconds:F1}s");

        Console.WriteLine("Phase 2 — write random values to writeable tags");
        sw.Restart();
        foreach (var t in tags)
        {
            if (!IsWriteable(t.Mode)) continue;
            var v = Rand(t.Kind, rng); if (v is null) continue;
            if (DoWrite(client, t, v)) { S(t.Category).WriteOk++; writtenValues.Add((t, v)); }
            else { S(t.Category).WriteFail++; }
        }
        sw.Stop(); Console.WriteLine($"  done in {sw.Elapsed.TotalSeconds:F1}s");

        Console.WriteLine("Phase 3 — verify writes via read-back");
        sw.Restart();
        foreach (var (t, expected) in writtenValues)
        {
            if (VerifyRead(client, t, expected)) S(t.Category).VerifyOk++; else S(t.Category).VerifyFail++;
        }
        sw.Stop(); Console.WriteLine($"  done in {sw.Elapsed.TotalSeconds:F1}s");

        Console.WriteLine("Phase 4 — confirm firmware-blocked writes are still blocked");
        sw.Restart();
        foreach (var t in tags)
        {
            if (!IsFirmwareBlocked(t.Mode)) continue;
            var v = Rand(t.Kind, rng); if (v is null) continue;
            if (DoWrite(client, t, v)) S(t.Category).BlockedUnexpected++; else S(t.Category).BlockedAsExpected++;
        }
        sw.Stop(); Console.WriteLine($"  done in {sw.Elapsed.TotalSeconds:F1}s");

        Console.WriteLine("Phase 5 — settle writeable tags to terminal state");
        sw.Restart();
        int settleOk = 0, settleFail = 0;
        foreach (var t in tags)
        {
            if (!IsWriteable(t.Mode)) continue;
            var v = Nines(t.Kind); if (v is null) continue;
            if (DoWrite(client, t, v)) settleOk++; else settleFail++;
        }
        sw.Stop(); Console.WriteLine($"  done in {sw.Elapsed.TotalSeconds:F1}s  settle_ok={settleOk} settle_fail={settleFail}");
        Console.WriteLine();

        Console.WriteLine("Phase 6 — verify settle (sample read-back)");
        sw.Restart();
        int settleVerifyOk = 0, settleVerifyFail = 0;
        foreach (var (category, tag, expected) in SettleSamples())
        {
            if (VerifyRead(client, tag, expected))
            {
                settleVerifyOk++;
                Console.WriteLine($"  verify-settle  {category,-28} {tag.Name,-48} ✓");
            }
            else
            {
                settleVerifyFail++;
                Console.WriteLine($"  verify-settle  {category,-28} {tag.Name,-48} ✗ MISMATCH");
            }
        }
        sw.Stop(); Console.WriteLine($"  done in {sw.Elapsed.TotalSeconds:F1}s  settle_verify={settleVerifyOk}/{settleVerifyOk + settleVerifyFail}");
        Console.WriteLine();

        Console.WriteLine("Per-category results:");
        Console.WriteLine($"  {"category",-32} {"read+",9} {"read-",9} {"write+",9} {"write-",9} {"verify+",9} {"blocked+",9}");
        var T = new CatStats();
        foreach (var (cat, st) in stats)
        {
            Console.WriteLine($"  {cat,-32} {st.ReadOk,9} {st.ReadFail,9} {st.WriteOk,9} {st.WriteFail,9} {st.VerifyOk,9} {st.BlockedAsExpected,9}");
            T.ReadOk += st.ReadOk; T.ReadFail += st.ReadFail;
            T.WriteOk += st.WriteOk; T.WriteFail += st.WriteFail;
            T.VerifyOk += st.VerifyOk; T.VerifyFail += st.VerifyFail;
            T.BlockedAsExpected += st.BlockedAsExpected; T.BlockedUnexpected += st.BlockedUnexpected;
        }
        Console.WriteLine($"  {"TOTAL",-32} {T.ReadOk,9} {T.ReadFail,9} {T.WriteOk,9} {T.WriteFail,9} {T.VerifyOk,9} {T.BlockedAsExpected,9}");
        Console.WriteLine();

        var unexpected = T.ReadFail + T.WriteFail + T.VerifyFail + T.BlockedUnexpected + settleFail + settleVerifyFail;
        Console.WriteLine($"Summary: reads={T.ReadOk}/{T.ReadOk + T.ReadFail}  writes={T.WriteOk}/{T.WriteOk + T.WriteFail}  verify={T.VerifyOk}/{T.VerifyOk + T.VerifyFail}  blocked_as_expected={T.BlockedAsExpected}  unexpected_anomalies={unexpected}");
        Console.WriteLine($"binding=csharp tags={tags.Count} reads={T.ReadOk}/{T.ReadOk + T.ReadFail} writes={T.WriteOk}/{T.WriteOk + T.WriteFail} verify={T.VerifyOk}/{T.VerifyOk + T.VerifyFail} blocked={T.BlockedAsExpected} anomalies={unexpected} RESULT={(unexpected == 0 ? "PASS" : "FAIL")}");
        WriteJsonResult(outDir, address, slot, tags.Count, preflightOk, preflightFail, stats, T, settleOk, settleFail, settleVerifyOk, settleVerifyFail, unexpected);
        Console.WriteLine(unexpected == 0 ? "RESULT: PASS" : $"RESULT: FAIL ({unexpected} anomalies)");
        return unexpected == 0 ? 0 : 1;
    }

    private static void WriteJsonResult(string outDir, string address, byte slot, int tagCount, int preflightOk, int preflightFail, SortedDictionary<string, CatStats> stats, CatStats totals, int settleOk, int settleFail, int settleVerifyOk, int settleVerifyFail, int anomalies)
    {
        Directory.CreateDirectory(outDir);
        var path = Path.Combine(outDir, $"csharp_{DateTimeOffset.UtcNow:yyyyMMddTHHmmssZ}.json");
        var result = new
        {
            schema_version = 1,
            binding = "csharp",
            binding_version = typeof(EtherNetIpClient).Assembly.GetName().Version?.ToString() ?? "unknown",
            plc_address = address,
            plc_slot = slot,
            manifest_version = 1,
            tag_count = tagCount,
            result = anomalies == 0 ? "PASS" : "FAIL",
            anomalies,
            phases = new
            {
                preflight = new { ok = preflightOk, fail = preflightFail },
                phase1_read = new { ok = totals.ReadOk, fail = totals.ReadFail },
                phase2_write = new { ok = totals.WriteOk, fail = totals.WriteFail },
                phase3_verify = new { ok = totals.VerifyOk, fail = totals.VerifyFail },
                phase4_blocked = new { ok = totals.BlockedAsExpected, fail = totals.BlockedUnexpected, note = "expected firmware rejections" },
                phase5_settle = new { ok = settleOk, fail = settleFail },
                phase6_verify_settle = new { ok = settleVerifyOk, fail = settleVerifyFail },
            },
            categories = stats.ToDictionary(
                entry => entry.Key,
                entry => new
                {
                    read_ok = entry.Value.ReadOk,
                    read_fail = entry.Value.ReadFail,
                    write_ok = entry.Value.WriteOk,
                    write_fail = entry.Value.WriteFail,
                    verify_ok = entry.Value.VerifyOk,
                    verify_fail = entry.Value.VerifyFail,
                    blocked_as_expected = entry.Value.BlockedAsExpected,
                    blocked_unexpected_pass = entry.Value.BlockedUnexpected,
                })
        };
        File.WriteAllText(path, JsonSerializer.Serialize(result, new JsonSerializerOptions { WriteIndented = true }));
    }
}
