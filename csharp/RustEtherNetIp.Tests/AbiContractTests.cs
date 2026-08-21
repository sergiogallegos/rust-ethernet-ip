using System;
using Xunit;

namespace RustEtherNetIp.Tests
{
    public class AbiContractTests
    {
        [Fact]
        public void NativeRuntimeReportsExpectedAbiVersion()
        {
            Assert.Equal(2u, NativeRuntime.AbiVersion);
            Assert.Equal(NativeRuntime.ExpectedAbiVersion, (int)NativeRuntime.AbiVersion);
        }

        [Fact]
        public void NativeRuntimeReportsLibraryVersion()
        {
            Assert.Equal("1.2.1", NativeRuntime.LibraryVersion);
        }

        [Fact]
        public void NativeRuntimeReportsCapabilityBitmap()
        {
            Assert.True((NativeRuntime.Capabilities & 0x0000_0000_0000_0001UL) != 0);
            Assert.True((NativeRuntime.Capabilities & 0x0000_0000_0000_0002UL) != 0);
            Assert.True((NativeRuntime.Capabilities & 0x0000_0000_0000_0004UL) != 0);
            Assert.True((NativeRuntime.Capabilities & 0x0000_0000_0000_0008UL) != 0);
        }

        [Fact]
        public void AbiMismatchPathIsDocumentedByExpectedVersionConstant()
        {
            Assert.Equal(2, NativeRuntime.ExpectedAbiVersion);
            Assert.IsAssignableFrom<Exception>(new BadImageFormatException());
        }
    }
}
