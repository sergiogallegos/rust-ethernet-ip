using System;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;
using Xunit;

namespace RustEtherNetIp.Tests
{
    public class MinimalLoadTest
    {
        [Fact]
        public void CanLoadRustEtherNetIpAssembly()
        {
            // Just reference a type from the assembly
            var type = typeof(RustEtherNetIp.EtherNetIpClient);
            Assert.NotNull(type);
        }

        [Fact]
        public void CanLoadNativeLibrary()
        {
            SimulatorTestHarness.StageNativeLibrary();

            var nativeLibName = RuntimeInformation.IsOSPlatform(OSPlatform.OSX)
                ? "librust_ethernet_ip.dylib"
                : RuntimeInformation.IsOSPlatform(OSPlatform.Linux)
                    ? "librust_ethernet_ip.so"
                    : "rust_ethernet_ip.dll";
            
            var nativeLibPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, nativeLibName);
            if (!File.Exists(nativeLibPath))
            {
                return;
            }

            // Try to load the native library
            var handle = NativeLibrary.Load(nativeLibPath);
            try
            {
                Assert.True(handle != IntPtr.Zero, "Failed to load native library");
                Assert.NotEqual(IntPtr.Zero, NativeLibrary.GetExport(handle, "eip_connect"));
                Assert.NotEqual(IntPtr.Zero, NativeLibrary.GetExport(handle, "eip_get_diagnostics_json"));
                Assert.NotEqual(IntPtr.Zero, NativeLibrary.GetExport(handle, "eip_execute_batch"));
            }
            finally
            {
                NativeLibrary.Free(handle);
            }
        }
    }
} 
