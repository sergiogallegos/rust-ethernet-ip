using System;
using System.Collections.Generic;
using RustEtherNetIp;

namespace CSharpRandomToNines;

internal enum Kind { Dint, Int, Real, Bool }

internal readonly record struct Spec(string Tag, Kind Kind);

internal static class Program
{
    private static readonly Spec[] Tags =
    [
        new("gTestArray_DINT[0]", Kind.Dint),
        new("gTestArray_DINT[5]", Kind.Dint),
        new("gTestArray_DINT[9]", Kind.Dint),
        new("gTestArray_REAL[0]", Kind.Real),
        new("gTestArray_REAL[4]", Kind.Real),
        new("gTestArray_INT[0]", Kind.Int),
        new("gTestArray_INT[9]", Kind.Int),
        new("gTestArray_BOOL[0]", Kind.Bool),
        new("gTestArray_BOOL[5]", Kind.Bool),
        new("gTestArray_Large[300]", Kind.Dint),
        new("gTestArray_Large[999]", Kind.Dint),
        new("gTestUDT.Member1_DINT", Kind.Dint),
        new("gTestUDT.Member2_REAL", Kind.Real),
        new("gTestUDT.Member3_BOOL", Kind.Bool),
        new("gTestUDT.Member4_INT", Kind.Int),
        new("gTestUDT.Array_DINT[5]", Kind.Dint),
        new("gTestUDT.Array_REAL[2]", Kind.Real),
        new("gTestUDT.Array_BOOL[10]", Kind.Bool),
        new("Program:TestProgram.gTestArray_DINT[5]", Kind.Dint),
        new("Program:TestProgram.gTestArray_REAL[0]", Kind.Real),
        new("Program:TestProgram.gTestArray_BOOL[0]", Kind.Bool),
        new("Program:TestProgram.gTestUDT.Member1_DINT", Kind.Dint),
        new("Program:TestProgram.gTestUDT.Member2_REAL", Kind.Real),
        new("Program:TestProgram.gTestUDT.Member3_BOOL", Kind.Bool),
        new("Program:TestProgram.gTestUDT.Member4_INT", Kind.Int),
        new("Program:TestProgram.gTestUDT.Array_DINT[5]", Kind.Dint),
        new("Program:TestProgram.gTestUDT.Array_REAL[2]", Kind.Real),
    ];

    private static object RandValue(Kind kind, Random rng) => kind switch
    {
        Kind.Dint => rng.Next(1_000, 900_000),
        Kind.Int  => (short)rng.Next(100, 20_000),
        Kind.Real => (float)(rng.NextDouble() * 9000.0 + 1.0),
        Kind.Bool => rng.Next(2) == 1,
        _ => throw new InvalidOperationException(),
    };

    private static object NinesValue(Kind kind) => kind switch
    {
        Kind.Dint => 999_999,
        Kind.Int  => (short)9_999,
        Kind.Real => 99.99f,
        Kind.Bool => true,
        _ => throw new InvalidOperationException(),
    };

    private static void Write(EtherNetIpClient client, Spec s, object v)
    {
        switch (s.Kind)
        {
            case Kind.Dint: client.WriteDint(s.Tag, (int)v); break;
            case Kind.Int:  client.WriteInt(s.Tag, (short)v); break;
            case Kind.Real: client.WriteReal(s.Tag, (float)v); break;
            case Kind.Bool: client.WriteBool(s.Tag, (bool)v); break;
        }
    }

    private static object Read(EtherNetIpClient client, Spec s) => s.Kind switch
    {
        Kind.Dint => client.ReadDint(s.Tag),
        Kind.Int  => (object)client.ReadInt(s.Tag),
        Kind.Real => client.ReadReal(s.Tag),
        Kind.Bool => client.ReadBool(s.Tag),
        _ => throw new InvalidOperationException(),
    };

    private static bool ValuesMatch(object a, object b, Kind k) => k switch
    {
        Kind.Dint => (int)a == (int)b,
        Kind.Int  => (short)a == (short)b,
        Kind.Bool => (bool)a == (bool)b,
        Kind.Real => Math.Abs((float)a - (float)b) < 0.001f,
        _ => false,
    };

    private static int Main()
    {
        var address = Environment.GetEnvironmentVariable("TEST_PLC_ADDRESS") ?? "192.168.0.1:44818";
        var slot = byte.TryParse(Environment.GetEnvironmentVariable("TEST_PLC_SLOT"), out var s) ? s : (byte)0;
        var rng  = new Random();

        Console.WriteLine("C# wrapper random->verify->nines cycle");
        Console.WriteLine($"PLC: {address} (slot {slot})");
        Console.WriteLine($"Tags: {Tags.Length}");
        Console.WriteLine();

        using var client = new EtherNetIpClient();
        if (!client.ConnectWithRoute(address, new RoutePath().AddSlot(slot)))
        {
            Console.Error.WriteLine($"ConnectWithRoute failed for {address}");
            return 2;
        }

        var written = new List<(Spec, object)>(Tags.Length);

        Console.WriteLine("Phase 1 - write random values");
        int writeOk = 0, writeFail = 0;
        foreach (var spec in Tags)
        {
            var v = RandValue(spec.Kind, rng);
            try
            {
                Write(client, spec, v);
                Console.WriteLine($"  WR  {spec.Tag,-55} {v}");
                written.Add((spec, v));
                writeOk++;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"  ERR {spec.Tag,-55} {ex.Message}");
                writeFail++;
            }
        }
        Console.WriteLine($"  -> {writeOk} ok, {writeFail} failed");
        Console.WriteLine();

        Console.WriteLine("Phase 2 - read back and verify");
        int verifyOk = 0, verifyFail = 0;
        foreach (var (spec, expected) in written)
        {
            try
            {
                var actual = Read(client, spec);
                var ok = ValuesMatch(actual, expected, spec.Kind);
                Console.WriteLine($"  {(ok ? "OK " : "MIS")}  {spec.Tag,-55} expected={expected} actual={actual}");
                if (ok) verifyOk++; else verifyFail++;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"  ERR {spec.Tag,-55} {ex.Message}");
                verifyFail++;
            }
        }
        Console.WriteLine($"  -> {verifyOk} matched, {verifyFail} mismatched/failed");
        Console.WriteLine();

        Console.WriteLine("Phase 3 - settle to terminal state (DINT=999999, INT=9999, REAL=99.99, BOOL=true)");
        int finalOk = 0, finalFail = 0;
        foreach (var spec in Tags)
        {
            try
            {
                Write(client, spec, NinesValue(spec.Kind));
                finalOk++;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"  ERR {spec.Tag,-55} {ex.Message}");
                finalFail++;
            }
        }
        Console.WriteLine($"  -> {finalOk} settled to nines/true, {finalFail} failed");
        Console.WriteLine();

        Console.WriteLine($"Summary: random_writes={writeOk}/{Tags.Length}, verify={verifyOk}/{writeOk}, terminal_writes={finalOk}/{Tags.Length}");
        if (writeFail == 0 && verifyFail == 0 && finalFail == 0)
        {
            Console.WriteLine("RESULT: PASS");
            return 0;
        }
        Console.WriteLine("RESULT: FAIL");
        return 1;
    }
}
