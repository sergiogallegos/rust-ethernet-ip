using System;
using System.IO;
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
        public void StagedNativeLibraryRemainsUnchangedAfterPInvokeLoad()
        {
            var nativeLibPath = SimulatorTestHarness.StageNativeLibrary();
            var before = new FileInfo(nativeLibPath);
            long originalLength = before.Length;
            DateTime originalWriteTime = before.LastWriteTimeUtc;

            // Exercise the library through the same CLR-managed P/Invoke path
            // used by consumers, then verify staging is a read-only operation.
            Assert.Equal(3u, NativeRuntime.AbiVersion);
            Assert.Equal(nativeLibPath, SimulatorTestHarness.StageNativeLibrary());

            var after = new FileInfo(nativeLibPath);
            Assert.Equal(originalLength, after.Length);
            Assert.Equal(originalWriteTime, after.LastWriteTimeUtc);
        }
    }
} 
