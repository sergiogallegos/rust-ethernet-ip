using System;
using System.Collections.Generic;
using System.Diagnostics;
using RustEtherNetIp;

namespace CSharpFullCoverage;

internal enum Kind { Dint, Int, Real, Bool, String, Udt }
internal enum WriteMode { Writeable, FirmwareBlocked, ReadOnly }

internal readonly record struct Tag(string Name, string Category, Kind Kind, WriteMode Mode);

internal sealed class CatStats
{
    public int ReadOk, ReadFail, WriteOk, WriteFail, VerifyOk, VerifyFail, BlockedAsExpected, BlockedUnexpected;
}

internal static class Program
{
    private static List<Tag> BuildTags()
    {
        var t = new List<Tag>(2400);

        for (int i = 0; i < 100; i++)  t.Add(new($"gTestArray_DINT[{i}]",  "ctrl.DINT_array",  Kind.Dint, WriteMode.Writeable));
        for (int i = 0; i < 50; i++)   t.Add(new($"gTestArray_REAL[{i}]",  "ctrl.REAL_array",  Kind.Real, WriteMode.Writeable));
        for (int i = 0; i < 128; i++)  t.Add(new($"gTestArray_BOOL[{i}]",  "ctrl.BOOL_array",  Kind.Bool, WriteMode.Writeable));
        for (int i = 0; i < 200; i++)  t.Add(new($"gTestArray_INT[{i}]",   "ctrl.INT_array",   Kind.Int,  WriteMode.Writeable));
        for (int i = 0; i < 1000; i++) t.Add(new($"gTestArray_Large[{i}]", "ctrl.Large_DINT",  Kind.Dint, WriteMode.Writeable));

        t.Add(new("gTest_STRING", "ctrl.STRING", Kind.String, WriteMode.FirmwareBlocked));

        t.Add(new("gTestUDT", "ctrl.UDT_whole", Kind.Udt, WriteMode.ReadOnly));
        t.Add(new("gTestUDT.Member1_DINT",   "ctrl.UDT_members", Kind.Dint,   WriteMode.Writeable));
        t.Add(new("gTestUDT.Member2_REAL",   "ctrl.UDT_members", Kind.Real,   WriteMode.Writeable));
        t.Add(new("gTestUDT.Member3_BOOL",   "ctrl.UDT_members", Kind.Bool,   WriteMode.Writeable));
        t.Add(new("gTestUDT.Member4_INT",    "ctrl.UDT_members", Kind.Int,    WriteMode.Writeable));
        t.Add(new("gTestUDT.Member5_String", "ctrl.UDT_members", Kind.String, WriteMode.FirmwareBlocked));
        for (int i = 0; i < 10; i++) t.Add(new($"gTestUDT.Array_DINT[{i}]", "ctrl.UDT_nested", Kind.Dint, WriteMode.Writeable));
        for (int i = 0; i < 5; i++)  t.Add(new($"gTestUDT.Array_REAL[{i}]", "ctrl.UDT_nested", Kind.Real, WriteMode.Writeable));
        for (int i = 0; i < 20; i++) t.Add(new($"gTestUDT.Array_BOOL[{i}]", "ctrl.UDT_nested", Kind.Bool, WriteMode.Writeable));

        t.Add(new("gTestUDT_Array", "ctrl.UDTarr_whole", Kind.Udt, WriteMode.ReadOnly));
        for (int i = 0; i < 10; i++)
        {
            t.Add(new($"gTestUDT_Array[{i}]", "ctrl.UDTarr_element", Kind.Udt, WriteMode.ReadOnly));
            t.Add(new($"gTestUDT_Array[{i}].Member1_DINT",   "ctrl.UDTarr_elem_members", Kind.Dint,   WriteMode.FirmwareBlocked));
            t.Add(new($"gTestUDT_Array[{i}].Member2_REAL",   "ctrl.UDTarr_elem_members", Kind.Real,   WriteMode.FirmwareBlocked));
            t.Add(new($"gTestUDT_Array[{i}].Member3_BOOL",   "ctrl.UDTarr_elem_members", Kind.Bool,   WriteMode.FirmwareBlocked));
            t.Add(new($"gTestUDT_Array[{i}].Member4_INT",    "ctrl.UDTarr_elem_members", Kind.Int,    WriteMode.FirmwareBlocked));
            t.Add(new($"gTestUDT_Array[{i}].Member5_String", "ctrl.UDTarr_elem_members", Kind.String, WriteMode.FirmwareBlocked));
            for (int j = 0; j < 10; j++) t.Add(new($"gTestUDT_Array[{i}].Array_DINT[{j}]", "ctrl.UDTarr_elem_nested", Kind.Dint, WriteMode.Writeable));
            for (int j = 0; j < 5; j++)  t.Add(new($"gTestUDT_Array[{i}].Array_REAL[{j}]", "ctrl.UDTarr_elem_nested", Kind.Real, WriteMode.Writeable));
            for (int j = 0; j < 20; j++) t.Add(new($"gTestUDT_Array[{i}].Array_BOOL[{j}]", "ctrl.UDTarr_elem_nested", Kind.Bool, WriteMode.Writeable));
        }

        for (int i = 0; i < 100; i++) t.Add(new($"Program:TestProgram.gTestArray_DINT[{i}]", "prog.DINT_array", Kind.Dint, WriteMode.Writeable));
        for (int i = 0; i < 50; i++)  t.Add(new($"Program:TestProgram.gTestArray_REAL[{i}]", "prog.REAL_array", Kind.Real, WriteMode.Writeable));
        for (int i = 0; i < 100; i++) t.Add(new($"Program:TestProgram.gTestArray_BOOL[{i}]", "prog.BOOL_array", Kind.Bool, WriteMode.Writeable));
        t.Add(new("Program:TestProgram.gTest_STRING", "prog.STRING", Kind.String, WriteMode.FirmwareBlocked));

        t.Add(new("Program:TestProgram.gTestUDT", "prog.UDT_whole", Kind.Udt, WriteMode.ReadOnly));
        t.Add(new("Program:TestProgram.gTestUDT.Member1_DINT",   "prog.UDT_members", Kind.Dint,   WriteMode.Writeable));
        t.Add(new("Program:TestProgram.gTestUDT.Member2_REAL",   "prog.UDT_members", Kind.Real,   WriteMode.Writeable));
        t.Add(new("Program:TestProgram.gTestUDT.Member3_BOOL",   "prog.UDT_members", Kind.Bool,   WriteMode.Writeable));
        t.Add(new("Program:TestProgram.gTestUDT.Member4_INT",    "prog.UDT_members", Kind.Int,    WriteMode.Writeable));
        t.Add(new("Program:TestProgram.gTestUDT.Member5_String", "prog.UDT_members", Kind.String, WriteMode.FirmwareBlocked));
        for (int i = 0; i < 10; i++) t.Add(new($"Program:TestProgram.gTestUDT.Array_DINT[{i}]", "prog.UDT_nested", Kind.Dint, WriteMode.Writeable));
        for (int i = 0; i < 5; i++)  t.Add(new($"Program:TestProgram.gTestUDT.Array_REAL[{i}]", "prog.UDT_nested", Kind.Real, WriteMode.Writeable));
        for (int i = 0; i < 20; i++) t.Add(new($"Program:TestProgram.gTestUDT.Array_BOOL[{i}]", "prog.UDT_nested", Kind.Bool, WriteMode.Writeable));

        t.Add(new("Program:TestProgram.gTestUDT_Array", "prog.UDTarr_whole", Kind.Udt, WriteMode.ReadOnly));
        for (int i = 0; i < 5; i++)
        {
            t.Add(new($"Program:TestProgram.gTestUDT_Array[{i}]", "prog.UDTarr_element", Kind.Udt, WriteMode.ReadOnly));
            t.Add(new($"Program:TestProgram.gTestUDT_Array[{i}].Member1_DINT", "prog.UDTarr_elem_members", Kind.Dint, WriteMode.FirmwareBlocked));
            t.Add(new($"Program:TestProgram.gTestUDT_Array[{i}].Member2_REAL", "prog.UDTarr_elem_members", Kind.Real, WriteMode.FirmwareBlocked));
            t.Add(new($"Program:TestProgram.gTestUDT_Array[{i}].Member3_BOOL", "prog.UDTarr_elem_members", Kind.Bool, WriteMode.FirmwareBlocked));
            t.Add(new($"Program:TestProgram.gTestUDT_Array[{i}].Member4_INT",  "prog.UDTarr_elem_members", Kind.Int,  WriteMode.FirmwareBlocked));
            for (int j = 0; j < 10; j++) t.Add(new($"Program:TestProgram.gTestUDT_Array[{i}].Array_DINT[{j}]", "prog.UDTarr_elem_nested", Kind.Dint, WriteMode.Writeable));
        }

        return t;
    }

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

    private static int Main()
    {
        var address = Environment.GetEnvironmentVariable("TEST_PLC_ADDRESS") ?? "192.168.0.1:44818";
        var slot = byte.TryParse(Environment.GetEnvironmentVariable("TEST_PLC_SLOT"), out var s) ? s : (byte)0;
        var tags = BuildTags();
        var rng = new Random();
        var stats = new SortedDictionary<string, CatStats>();
        var writtenValues = new List<(Tag, object)>(1900);
        int writeable = 0, blocked = 0, readonly_ = 0;
        foreach (var t in tags)
        {
            switch (t.Mode) { case WriteMode.Writeable: writeable++; break; case WriteMode.FirmwareBlocked: blocked++; break; case WriteMode.ReadOnly: readonly_++; break; }
        }

        Console.WriteLine("C# wrapper — full-coverage exerciser");
        Console.WriteLine($"PLC: {address} (slot {slot})  total tags: {tags.Count}");
        Console.WriteLine($"  writeable: {writeable}   firmware-blocked: {blocked}   read-only: {readonly_}");
        Console.WriteLine();

        using var client = new EtherNetIpClient();
        if (!client.ConnectWithRoute(address, new RoutePath().AddSlot(slot)))
        {
            Console.Error.WriteLine("Connect failed"); return 2;
        }

        CatStats S(string c) => stats.TryGetValue(c, out var v) ? v : (stats[c] = new CatStats());

        Console.WriteLine("Phase 1 — read every tag");
        var sw = Stopwatch.StartNew();
        foreach (var t in tags) { if (DoRead(client, t)) S(t.Category).ReadOk++; else S(t.Category).ReadFail++; }
        sw.Stop(); Console.WriteLine($"  done in {sw.Elapsed.TotalSeconds:F1}s");

        Console.WriteLine("Phase 2 — write random values to writeable tags");
        sw.Restart();
        foreach (var t in tags)
        {
            if (t.Mode != WriteMode.Writeable) continue;
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
            if (t.Mode != WriteMode.FirmwareBlocked) continue;
            var v = Rand(t.Kind, rng); if (v is null) continue;
            if (DoWrite(client, t, v)) S(t.Category).BlockedUnexpected++; else S(t.Category).BlockedAsExpected++;
        }
        sw.Stop(); Console.WriteLine($"  done in {sw.Elapsed.TotalSeconds:F1}s");

        Console.WriteLine("Phase 5 — settle writeable tags to terminal state");
        sw.Restart();
        int settleOk = 0, settleFail = 0;
        foreach (var t in tags)
        {
            if (t.Mode != WriteMode.Writeable) continue;
            var v = Nines(t.Kind); if (v is null) continue;
            if (DoWrite(client, t, v)) settleOk++; else settleFail++;
        }
        sw.Stop(); Console.WriteLine($"  done in {sw.Elapsed.TotalSeconds:F1}s  settle_ok={settleOk} settle_fail={settleFail}");
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

        var unexpected = T.ReadFail + T.WriteFail + T.VerifyFail + T.BlockedUnexpected + settleFail;
        Console.WriteLine($"Summary: reads={T.ReadOk}/{T.ReadOk + T.ReadFail}  writes={T.WriteOk}/{T.WriteOk + T.WriteFail}  verify={T.VerifyOk}/{T.VerifyOk + T.VerifyFail}  blocked_as_expected={T.BlockedAsExpected}  unexpected_anomalies={unexpected}");
        Console.WriteLine(unexpected == 0 ? "RESULT: PASS" : $"RESULT: FAIL ({unexpected} anomalies)");
        return unexpected == 0 ? 0 : 1;
    }
}
