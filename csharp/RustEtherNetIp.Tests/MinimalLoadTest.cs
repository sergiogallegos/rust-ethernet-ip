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
            // Check if the native library exists
            var nativeLibName = RuntimeInformation.IsOSPlatform(OSPlatform.OSX) 
                ? "librust_ethernet_ip.dylib" 
                : "rust_ethernet_ip.dll";
            
            var nativeLibPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, nativeLibName);
            if (!File.Exists(nativeLibPath))
            {
                return;
            }

            // Try to load the native library
            var handle = NativeLibrary.Load(nativeLibPath);
            Assert.True(handle != IntPtr.Zero, "Failed to load native library");
            
            // Clean up
            NativeLibrary.Free(handle);
        }
    }
} 