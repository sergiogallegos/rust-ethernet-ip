using System.Diagnostics;
using System.Runtime.InteropServices;

namespace RustEtherNetIp.IntegrationTests;

public sealed class PlcSimulatorFixture : IDisposable
{
    private readonly Process? _process;

    public PlcSimulatorFixture()
    {
        StageNativeLibrary();

        var configuredAddress = Environment.GetEnvironmentVariable("SIM_PLC_ADDRESS");
        if (!string.IsNullOrWhiteSpace(configuredAddress))
        {
            Address = configuredAddress;
            return;
        }

        var repoRoot = FindRepoRoot();
        var simulatorBinary = Environment.GetEnvironmentVariable("PLC_SIM_BINARY");
        var startInfo = string.IsNullOrWhiteSpace(simulatorBinary)
            ? new ProcessStartInfo("cargo", "run --quiet --bin plc_sim")
            : new ProcessStartInfo(simulatorBinary);

        startInfo.WorkingDirectory = repoRoot;
        startInfo.RedirectStandardOutput = true;
        startInfo.RedirectStandardError = true;
        startInfo.UseShellExecute = false;
        startInfo.CreateNoWindow = true;

        _process = Process.Start(startInfo)
            ?? throw new InvalidOperationException("Failed to start plc_sim process.");

        Address = ReadSimulatorAddress(_process);
    }

    public string Address { get; }

    public void Dispose()
    {
        if (_process == null)
            return;

        try
        {
            if (!_process.HasExited)
            {
                _process.Kill(entireProcessTree: true);
                _process.WaitForExit(5000);
            }
        }
        finally
        {
            _process.Dispose();
        }
    }

    private static string ReadSimulatorAddress(Process process)
    {
        var deadline = DateTime.UtcNow.AddSeconds(60);
        while (DateTime.UtcNow < deadline)
        {
            var remaining = deadline - DateTime.UtcNow;
            var lineTask = process.StandardOutput.ReadLineAsync();
            if (!lineTask.Wait(remaining))
                break;

            var line = lineTask.Result;
            if (line == null)
                break;

            const string marker = "SIM_PLC_ADDRESS=";
            int markerIndex = line.IndexOf(marker, StringComparison.Ordinal);
            if (markerIndex >= 0)
                return line[(markerIndex + marker.Length)..].Trim();

            const string listening = "PLC simulator listening on ";
            int listeningIndex = line.IndexOf(listening, StringComparison.Ordinal);
            if (listeningIndex >= 0)
                return line[(listeningIndex + listening.Length)..].Trim();
        }

        string error = process.StandardError.ReadToEnd();
        throw new TimeoutException($"plc_sim did not report a listening address. stderr: {error}");
    }

    private static void StageNativeLibrary()
    {
        string source = ResolveNativeLibrary();
        string destination = Path.Combine(AppContext.BaseDirectory, NativeLibraryFileName);
        if (!File.Exists(destination) || File.GetLastWriteTimeUtc(destination) < File.GetLastWriteTimeUtc(source))
            File.Copy(source, destination, overwrite: true);
    }

    private static string ResolveNativeLibrary()
    {
        var configured = Environment.GetEnvironmentVariable("RUST_ETHERNET_IP_NATIVE_LIB");
        if (!string.IsNullOrWhiteSpace(configured) && File.Exists(configured))
            return configured;

        var repoRoot = FindRepoRoot();
        var releasePath = Path.Combine(repoRoot, "target", "release", NativeLibraryFileName);
        if (File.Exists(releasePath))
            return releasePath;

        var debugPath = Path.Combine(repoRoot, "target", "debug", NativeLibraryFileName);
        if (File.Exists(debugPath))
            return debugPath;

        throw new FileNotFoundException($"Could not find {NativeLibraryFileName}; build the native library with `cargo build --release --features ffi --locked`.");
    }

    private static string FindRepoRoot()
    {
        var directory = new DirectoryInfo(AppContext.BaseDirectory);
        while (directory != null)
        {
            if (File.Exists(Path.Combine(directory.FullName, "Cargo.toml")) &&
                Directory.Exists(Path.Combine(directory.FullName, "csharp")))
            {
                return directory.FullName;
            }

            directory = directory.Parent;
        }

        throw new DirectoryNotFoundException("Could not locate repository root from test output directory.");
    }

    private static string NativeLibraryFileName
    {
        get
        {
            if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
                return "rust_ethernet_ip.dll";
            if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
                return "librust_ethernet_ip.dylib";
            return "librust_ethernet_ip.so";
        }
    }
}
