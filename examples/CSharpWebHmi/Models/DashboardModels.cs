namespace CSharpWebHmi.Models;

public sealed record DashboardSnapshot(
    string Mode,
    string ConnectionState,
    string Target,
    int Slot,
    string Controller,
    string Firmware,
    string LibraryVersion,
    uint AbiVersion,
    bool WritesEnabled,
    DateTimeOffset RefreshedAt,
    DateTimeOffset? LastGoodAt,
    double ScanTimeMs,
    int GoodSignals,
    int TotalSignals,
    string OperatorMessage,
    IReadOnlyList<DashboardSignal> Signals,
    IReadOnlyList<ScopeSummary> Scopes,
    IReadOnlyList<double> AnalogProfile,
    IReadOnlyList<int> CounterProfile,
    IReadOnlyList<bool> DigitalProfile,
    IReadOnlyList<DashboardNotice> Notices);

public sealed record DashboardSignal(
    string Id,
    string Label,
    string Tag,
    string Scope,
    string DataType,
    object? Value,
    string DisplayValue,
    string? Unit,
    string Quality);

public sealed record ScopeSummary(
    string Id,
    string Label,
    string Detail,
    int Good,
    int Total,
    string State);

public sealed record DashboardNotice(
    string Severity,
    string Code,
    string Message);

public sealed record CommandResult(bool Success, string Message);
