using System.Diagnostics;
using CSharpWebHmi.Models;
using RustEtherNetIp;

namespace CSharpWebHmi.Services;

public sealed class PlcDashboardService : IDisposable
{
    private const string DefaultAddress = "192.168.0.20:44818";
    private const string PulseTag = "gTestArray_BOOL[0]";

    private readonly SemaphoreSlim _gate = new(1, 1);
    private readonly ILogger<PlcDashboardService> _logger;
    private readonly string _configuredMode;
    private readonly string _address;
    private readonly int _slot;
    private readonly bool _useRoute;
    private readonly string _controllerName;
    private readonly string _controllerFirmware;
    private readonly bool _allowWrites;
    private readonly bool _fallbackToSimulation;
    private EtherNetIpClient? _client;

    public PlcDashboardService(IConfiguration configuration, ILogger<PlcDashboardService> logger)
    {
        _logger = logger;
        _configuredMode = (configuration["HMI_MODE"] ?? "simulation").Trim().ToLowerInvariant();
        _address = configuration["HMI_PLC_ADDRESS"] ?? DefaultAddress;
        _slot = int.TryParse(configuration["HMI_PLC_SLOT"], out var slot) ? slot : 0;
        _useRoute = !bool.TryParse(configuration["HMI_USE_ROUTE"], out var useRoute) || useRoute;
        _controllerName = configuration["HMI_CONTROLLER_NAME"] ?? "Logix controller";
        _controllerFirmware = configuration["HMI_CONTROLLER_FIRMWARE"] ?? "User configured";
        _allowWrites = bool.TryParse(configuration["HMI_ALLOW_WRITES"], out var allowWrites) && allowWrites;
        _fallbackToSimulation = !bool.TryParse(configuration["HMI_FALLBACK_TO_SIMULATION"], out var fallback)
            || fallback;
    }

    public async Task<DashboardSnapshot> GetSnapshotAsync(CancellationToken cancellationToken)
    {
        await _gate.WaitAsync(cancellationToken);
        try
        {
            if (_configuredMode == "live")
            {
                try
                {
                    EnsureConnected();
                    return ReadLiveSnapshot();
                }
                catch (Exception ex) when (_fallbackToSimulation)
                {
                    _logger.LogWarning(ex, "Live PLC snapshot failed; returning simulation data");
                    ResetClient();
                    return CreateSimulationSnapshot($"Live target unavailable — simulation active: {ex.Message}");
                }
            }

            return CreateSimulationSnapshot();
        }
        finally
        {
            _gate.Release();
        }
    }

    public async Task<CommandResult> PulseTestBoolAsync(CancellationToken cancellationToken)
    {
        if (!_allowWrites)
            return new CommandResult(false, "Writes are disabled. Set HMI_ALLOW_WRITES=true to enable the allowlisted test pulse.");
        if (_configuredMode != "live")
            return new CommandResult(false, "The test pulse is available only in live mode.");

        await _gate.WaitAsync(cancellationToken);
        try
        {
            EnsureConnected();
            bool original = _client!.ReadBool(PulseTag);
            _client.WriteBool(PulseTag, !original);
            await Task.Delay(300, cancellationToken);
            _client.WriteBool(PulseTag, original);
            return new CommandResult(true, $"Pulsed and restored {PulseTag}.");
        }
        catch (Exception ex)
        {
            ResetClient();
            return new CommandResult(false, ex.Message);
        }
        finally
        {
            _gate.Release();
        }
    }

    private void EnsureConnected()
    {
        if (_client is not null && _client.CheckHealth())
            return;

        ResetClient();
        _client = new EtherNetIpClient();
        bool connected = _useRoute
            ? _client.ConnectWithRoute(_address, new RoutePath().AddSlot(checked((byte)_slot)))
            : _client.Connect(_address);
        if (!connected)
        {
            string detail = _client.LastConnectError ?? "Native connection failed without additional detail.";
            ResetClient();
            string route = _useRoute ? $" through slot {_slot}" : " directly";
            throw new InvalidOperationException($"Unable to connect to {_address}{route}: {detail}");
        }
    }

    private DashboardSnapshot ReadLiveSnapshot()
    {
        var stopwatch = Stopwatch.StartNew();
        var signals = new List<DashboardSignal>();

        for (int i = 0; i < 8; i++)
        {
            int index = i;
            signals.Add(ReadSignal($"dint-{i}", $"Counter {i + 1:00}", $"gTestArray_DINT[{i}]", "Controller Arrays", "DINT", null,
                () => _client!.ReadDint($"gTestArray_DINT[{index}]")));
            signals.Add(ReadSignal($"real-{i}", $"Analog {i + 1:00}", $"gTestArray_REAL[{i}]", "Controller Arrays", "REAL", "raw",
                () => _client!.ReadReal($"gTestArray_REAL[{index}]")));
        }

        for (int i = 0; i < 12; i++)
        {
            int index = i;
            signals.Add(ReadSignal($"bool-{i}", $"Digital {i + 1:00}", $"gTestArray_BOOL[{i}]", "Controller Arrays", "BOOL", null,
                () => _client!.ReadBool($"gTestArray_BOOL[{index}]")));
        }

        signals.Add(ReadSignal("int-0", "Integer Channel", "gTestArray_INT[0]", "Controller Arrays", "INT", null,
            () => _client!.ReadInt("gTestArray_INT[0]")));
        signals.Add(ReadSignal("string-0", "Controller Message", "gTest_STRING", "Controller Scope", "STRING", null,
            () => _client!.ReadString("gTest_STRING")));
        signals.Add(ReadSignal("udt-dint", "UDT Counter", "gTestUDT.Member1_DINT", "UDT Structure", "DINT", null,
            () => _client!.ReadDint("gTestUDT.Member1_DINT")));
        signals.Add(ReadSignal("udt-real", "UDT Analog", "gTestUDT.Member2_REAL", "UDT Structure", "REAL", "raw",
            () => _client!.ReadReal("gTestUDT.Member2_REAL")));
        signals.Add(ReadSignal("udt-bool", "UDT State", "gTestUDT.Member3_BOOL", "UDT Structure", "BOOL", null,
            () => _client!.ReadBool("gTestUDT.Member3_BOOL")));
        signals.Add(ReadSignal("udt-int", "UDT Integer", "gTestUDT.Member4_INT", "UDT Structure", "INT", null,
            () => _client!.ReadInt("gTestUDT.Member4_INT")));
        signals.Add(ReadSignal("udt-string", "UDT Message", "gTestUDT.Member5_String", "UDT Structure", "STRING", null,
            () => _client!.ReadString("gTestUDT.Member5_String")));
        signals.Add(ReadSignal("program-dint", "Program Counter", "Program:TestProgram.gTestArray_DINT[0]", "Program Scope", "DINT", null,
            () => _client!.ReadDint("Program:TestProgram.gTestArray_DINT[0]")));
        signals.Add(ReadSignal("program-real", "Program Analog", "Program:TestProgram.gTestArray_REAL[0]", "Program Scope", "REAL", "raw",
            () => _client!.ReadReal("Program:TestProgram.gTestArray_REAL[0]")));
        signals.Add(ReadSignal("program-bool", "Program State", "Program:TestProgram.gTestArray_BOOL[0]", "Program Scope", "BOOL", null,
            () => _client!.ReadBool("Program:TestProgram.gTestArray_BOOL[0]")));
        signals.Add(ReadSignal("program-string", "Program Message", "Program:TestProgram.gTest_STRING", "Program Scope", "STRING", null,
            () => _client!.ReadString("Program:TestProgram.gTest_STRING")));

        stopwatch.Stop();
        int good = signals.Count(signal => signal.Quality == "Good");
        var scopes = BuildScopes(signals, _useRoute);
        var notices = BuildNotices(signals, live: true);

        return new DashboardSnapshot(
            "Live PLC",
            good == signals.Count ? "Connected" : "Degraded",
            _useRoute ? $"Communication module route → processor slot {_slot}" : "Direct controller endpoint",
            _slot,
            _controllerName,
            _controllerFirmware,
            NativeRuntime.LibraryVersion,
            NativeRuntime.AbiVersion,
            _allowWrites,
            DateTimeOffset.Now,
            stopwatch.Elapsed.TotalMilliseconds,
            good,
            signals.Count,
            good == signals.Count
                ? "Validation cell online — controller, UDT, and program-scope signals are healthy"
                : $"Signal quality degraded — {signals.Count - good} read failure(s) require attention",
            signals,
            scopes,
            signals.Where(signal => signal.Id.StartsWith("real-")).Select(ToDouble).ToArray(),
            signals.Where(signal => signal.Id.StartsWith("dint-")).Select(ToInt).ToArray(),
            signals.Where(signal => signal.Id.StartsWith("bool-")).Select(ToBool).ToArray(),
            notices);
    }

    private DashboardSignal ReadSignal(
        string id,
        string label,
        string tag,
        string scope,
        string dataType,
        string? unit,
        Func<object?> read)
    {
        try
        {
            object? value = read();
            return new DashboardSignal(id, label, tag, scope, dataType, value, FormatValue(value), unit, "Good");
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Read failed for {Tag}", tag);
            return new DashboardSignal(id, label, tag, scope, dataType, null, "—", unit, "Bad");
        }
    }

    private DashboardSnapshot CreateSimulationSnapshot(string? fallbackMessage = null)
    {
        double phase = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() / 1800.0;
        var signals = new List<DashboardSignal>();
        for (int i = 0; i < 8; i++)
        {
            int counter = 4200 + (i * 137) + (int)(Math.Sin(phase + i) * 24);
            float analog = (float)(48 + (i * 4.25) + Math.Sin(phase * 0.8 + i) * 5.5);
            signals.Add(SimSignal($"dint-{i}", $"Counter {i + 1:00}", $"gTestArray_DINT[{i}]", "Controller Arrays", "DINT", counter));
            signals.Add(SimSignal($"real-{i}", $"Analog {i + 1:00}", $"gTestArray_REAL[{i}]", "Controller Arrays", "REAL", analog, "raw"));
        }
        for (int i = 0; i < 12; i++)
            signals.Add(SimSignal($"bool-{i}", $"Digital {i + 1:00}", $"gTestArray_BOOL[{i}]", "Controller Arrays", "BOOL", i is 0 or 1 or 3 or 6 or 9));

        signals.Add(SimSignal("int-0", "Integer Channel", "gTestArray_INT[0]", "Controller Arrays", "INT", (short)128));
        signals.Add(SimSignal("string-0", "Controller Message", "gTest_STRING", "Controller Scope", "STRING", "CELL_READY"));
        signals.Add(SimSignal("udt-dint", "UDT Counter", "gTestUDT.Member1_DINT", "UDT Structure", "DINT", 1842));
        signals.Add(SimSignal("udt-real", "UDT Analog", "gTestUDT.Member2_REAL", "UDT Structure", "REAL", 73.4f, "raw"));
        signals.Add(SimSignal("udt-bool", "UDT State", "gTestUDT.Member3_BOOL", "UDT Structure", "BOOL", true));
        signals.Add(SimSignal("udt-int", "UDT Integer", "gTestUDT.Member4_INT", "UDT Structure", "INT", (short)42));
        signals.Add(SimSignal("udt-string", "UDT Message", "gTestUDT.Member5_String", "UDT Structure", "STRING", "STRUCTURE_OK"));
        signals.Add(SimSignal("program-dint", "Program Counter", "Program:TestProgram.gTestArray_DINT[0]", "Program Scope", "DINT", 9204));
        signals.Add(SimSignal("program-real", "Program Analog", "Program:TestProgram.gTestArray_REAL[0]", "Program Scope", "REAL", 64.8f, "raw"));
        signals.Add(SimSignal("program-bool", "Program State", "Program:TestProgram.gTestArray_BOOL[0]", "Program Scope", "BOOL", true));
        signals.Add(SimSignal("program-string", "Program Message", "Program:TestProgram.gTest_STRING", "Program Scope", "STRING", "PROGRAM_READY"));

        var notices = new List<DashboardNotice>
        {
            new("Info", "D-101", "Simulation mode uses the same tag contract as the live Logix dashboard."),
            new("Warning", "D-204", "Writes remain disabled until live mode and the explicit write flag are enabled."),
        };
        if (fallbackMessage is not null)
            notices.Insert(0, new DashboardNotice("Warning", "C-301", fallbackMessage));

        return new DashboardSnapshot(
            "Simulation",
            fallbackMessage is null ? "Simulated" : "Fallback",
            "Local deterministic data source",
            _slot,
            "ControlLogix validation model",
            "Demo",
            "1.2.1",
            3,
            false,
            DateTimeOffset.Now,
            0.8,
            signals.Count,
            signals.Count,
            fallbackMessage ?? "Validation cell ready — simulated controller contract is healthy",
            signals,
            BuildScopes(signals, _useRoute),
            signals.Where(signal => signal.Id.StartsWith("real-")).Select(ToDouble).ToArray(),
            signals.Where(signal => signal.Id.StartsWith("dint-")).Select(ToInt).ToArray(),
            signals.Where(signal => signal.Id.StartsWith("bool-")).Select(ToBool).ToArray(),
            notices);
    }

    private static DashboardSignal SimSignal(
        string id,
        string label,
        string tag,
        string scope,
        string dataType,
        object value,
        string? unit = null) =>
        new(id, label, tag, scope, dataType, value, FormatValue(value), unit, "Good");

    private static IReadOnlyList<ScopeSummary> BuildScopes(IReadOnlyList<DashboardSignal> signals, bool routed)
    {
        var definitions = new[]
        {
            ("arrays", "Controller Arrays", "Atomic arrays and packed BOOL addressing"),
            ("udt", "UDT Structure", "Typed member access with structure handles"),
            ("program", "Program Scope", "Program:TestProgram symbolic paths"),
            ("route", routed ? "Routed Target" : "Direct Target",
                routed ? "Backplane route through the communication module" : "Direct processor Ethernet endpoint"),
        };

        return definitions.Select(definition =>
        {
            var subset = definition.Item1 == "route"
                ? signals
                : signals.Where(signal => signal.Scope == definition.Item2).ToArray();
            int total = subset.Count();
            int good = subset.Count(signal => signal.Quality == "Good");
            return new ScopeSummary(
                definition.Item1,
                definition.Item2,
                definition.Item3,
                good,
                total,
                total > 0 && good == total ? "Healthy" : "Attention");
        }).ToArray();
    }

    private static IReadOnlyList<DashboardNotice> BuildNotices(IReadOnlyList<DashboardSignal> signals, bool live)
    {
        var notices = new List<DashboardNotice>();
        int failures = signals.Count(signal => signal.Quality != "Good");
        if (failures > 0)
            notices.Add(new DashboardNotice("Alarm", "C-401", $"{failures} PLC signal read(s) failed during the last scan."));
        else
            notices.Add(new DashboardNotice("Info", "C-100", "All configured controller, UDT, and program-scope reads returned good quality."));

        notices.Add(new DashboardNotice(
            "Info",
            "R-121",
            live ? "Live data passes through the C# wrapper and Rust EtherNet/IP 1.2.1 core." : "Simulation preserves the live dashboard data contract."));
        return notices;
    }

    private static string FormatValue(object? value) => value switch
    {
        null => "—",
        bool boolean => boolean ? "ON" : "OFF",
        float single => single.ToString("0.00"),
        double number => number.ToString("0.00"),
        _ => value.ToString() ?? "—",
    };

    private static double ToDouble(DashboardSignal signal) => signal.Value switch
    {
        float value => value,
        double value => value,
        int value => value,
        short value => value,
        _ => 0,
    };

    private static int ToInt(DashboardSignal signal) => signal.Value switch
    {
        int value => value,
        short value => value,
        _ => 0,
    };

    private static bool ToBool(DashboardSignal signal) => signal.Value is true;

    private void ResetClient()
    {
        _client?.Dispose();
        _client = null;
    }

    public void Dispose()
    {
        ResetClient();
        _gate.Dispose();
    }
}
