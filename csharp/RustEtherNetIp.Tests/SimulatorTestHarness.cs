using System;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;

namespace RustEtherNetIp.Tests
{
    internal sealed class SimulatorTestHarness : IDisposable
    {
        private static readonly object NativeLibraryLock = new();
        private static bool _nativeLibraryStaged;

        private Process? _process;

        public string Address { get; }

        public SimulatorTestHarness()
        {
            StageNativeLibrary();

            var configuredAddress = Environment.GetEnvironmentVariable("SIM_PLC_ADDRESS");
            if (!string.IsNullOrWhiteSpace(configuredAddress))
            {
                Address = configuredAddress;
                return;
            }

            var simulatorPath = ResolveSimulatorBinary();
            _process = new Process
            {
                StartInfo = new ProcessStartInfo
                {
                    FileName = simulatorPath,
                    WorkingDirectory = RepoRoot,
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                    UseShellExecute = false,
                    CreateNoWindow = true
                },
                EnableRaisingEvents = true
            };

            if (!_process.Start())
                throw new InvalidOperationException("Failed to start simulator process.");

            Address = _process.StandardOutput.ReadLine()?.Trim()
                ?? throw new InvalidOperationException(ReadSimulatorStartupError("Simulator did not print an address."));

            if (string.IsNullOrWhiteSpace(Address))
                throw new InvalidOperationException(ReadSimulatorStartupError("Simulator printed an empty address."));
        }

        public EtherNetIpClient ConnectClient()
        {
            var client = new EtherNetIpClient();
            if (!client.Connect(Address))
            {
                client.Dispose();
                throw new InvalidOperationException($"Could not connect C# wrapper to simulator at {Address}.");
            }

            return client;
        }

        public EtherNetIpClient ConnectClientWithRoute(RoutePath routePath)
        {
            var client = new EtherNetIpClient();
            if (!client.ConnectWithRoute(Address, routePath))
            {
                client.Dispose();
                throw new InvalidOperationException($"Could not connect C# wrapper to simulator at {Address} with route.");
            }

            return client;
        }

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
                _process = null;
            }
        }

        public static void StageNativeLibrary()
        {
            lock (NativeLibraryLock)
            {
                if (_nativeLibraryStaged)
                    return;

                var source = ResolveNativeLibraryWithFfiExports();
                var destination = Path.Combine(AppContext.BaseDirectory, NativeLibraryFileName);

                if (!PathsEqual(source, destination))
                {
                    try
                    {
                        File.Copy(source, destination, overwrite: true);
                    }
                    catch (IOException) when (File.Exists(destination))
                    {
                        // Another parallel testhost process already loaded and
                        // locked the destination DLL (Windows file-locking on
                        // LoadLibrary). The export assertion below validates
                        // that the existing copy is the right one.
                    }
                }

                AssertNativeLibraryHasRequiredExports(destination);
                _nativeLibraryStaged = true;
            }
        }

        private string ReadSimulatorStartupError(string prefix)
        {
            var stderr = _process?.StandardError.ReadToEnd();
            return string.IsNullOrWhiteSpace(stderr) ? prefix : $"{prefix} {stderr.Trim()}";
        }

        private static string ResolveSimulatorBinary()
        {
            var configured = Environment.GetEnvironmentVariable("RUST_ETHERNET_IP_SIM_BIN");
            if (!string.IsNullOrWhiteSpace(configured))
            {
                if (!File.Exists(configured))
                    throw new FileNotFoundException("Configured simulator binary does not exist.", configured);
                return configured;
            }

            var simulatorPath = Path.Combine(
                RepoRoot,
                "target",
                "debug",
                "examples",
                RuntimeInformation.IsOSPlatform(OSPlatform.Windows)
                    ? "python_test_simulator.exe"
                    : "python_test_simulator");

            if (File.Exists(simulatorPath))
                return simulatorPath;

            RunCargo("build", "--features", "ffi", "--example", "python_test_simulator");

            if (!File.Exists(simulatorPath))
                throw new FileNotFoundException("Cargo build completed but simulator binary was not produced.", simulatorPath);

            return simulatorPath;
        }

        private static string ResolveNativeLibraryWithFfiExports()
        {
            var configured = Environment.GetEnvironmentVariable("RUST_ETHERNET_IP_NATIVE_LIB");
            if (!string.IsNullOrWhiteSpace(configured))
            {
                AssertNativeLibraryHasRequiredExports(configured);
                return configured;
            }

            var debugPath = Path.Combine(RepoRoot, "target", "debug", NativeLibraryFileName);
            if (File.Exists(debugPath) && HasRequiredExports(debugPath))
                return debugPath;

            RunCargo("build", "--features", "ffi");

            if (File.Exists(debugPath) && HasRequiredExports(debugPath))
                return debugPath;

            var releasePath = Path.Combine(RepoRoot, "target", "release", NativeLibraryFileName);
            if (File.Exists(releasePath) && HasRequiredExports(releasePath))
                return releasePath;

            throw new FileNotFoundException($"Could not find a native {NativeLibraryFileName} with required FFI exports.");
        }

        private static string NativeLibraryFileName
        {
            get
            {
                if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
                    return "librust_ethernet_ip.dylib";
                if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
                    return "librust_ethernet_ip.so";
                return "rust_ethernet_ip.dll";
            }
        }

        private static string RepoRoot
        {
            get
            {
                var current = new DirectoryInfo(AppContext.BaseDirectory);
                while (current != null)
                {
                    if (File.Exists(Path.Combine(current.FullName, "Cargo.toml")))
                        return current.FullName;
                    current = current.Parent;
                }

                throw new DirectoryNotFoundException("Could not locate repository root from test output directory.");
            }
        }

        private static void RunCargo(params string[] args)
        {
            var startInfo = new ProcessStartInfo
            {
                FileName = "cargo",
                WorkingDirectory = RepoRoot,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true
            };

            foreach (var arg in args)
                startInfo.ArgumentList.Add(arg);

            using var process = Process.Start(startInfo)
                ?? throw new InvalidOperationException("Failed to start cargo.");

            var stdout = process.StandardOutput.ReadToEnd();
            var stderr = process.StandardError.ReadToEnd();
            if (!process.WaitForExit(120000))
            {
                process.Kill(entireProcessTree: true);
                throw new TimeoutException($"cargo {string.Join(' ', args)} timed out.");
            }

            if (process.ExitCode != 0)
                throw new InvalidOperationException($"cargo {string.Join(' ', args)} failed. {stderr}{stdout}");
        }

        private static bool HasRequiredExports(string path)
        {
            try
            {
                AssertNativeLibraryHasRequiredExports(path);
                return true;
            }
            catch
            {
                return false;
            }
        }

        private static void AssertNativeLibraryHasRequiredExports(string path)
        {
            if (!File.Exists(path))
                throw new FileNotFoundException("Native library does not exist.", path);

            var handle = NativeLibrary.Load(path);
            try
            {
                NativeLibrary.GetExport(handle, "eip_connect");
                NativeLibrary.GetExport(handle, "eip_get_diagnostics_json");
                NativeLibrary.GetExport(handle, "eip_execute_batch");
            }
            finally
            {
                NativeLibrary.Free(handle);
            }
        }

        private static bool PathsEqual(string first, string second)
        {
            return string.Equals(
                Path.GetFullPath(first).TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar),
                Path.GetFullPath(second).TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar),
                StringComparison.OrdinalIgnoreCase);
        }
    }
}
