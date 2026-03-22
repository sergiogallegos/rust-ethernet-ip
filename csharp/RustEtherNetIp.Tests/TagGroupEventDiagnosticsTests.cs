using System;
using Xunit;
using RustEtherNetIp;

namespace RustEtherNetIp.Tests
{
    public class TagGroupEventDiagnosticsTests
    {
        [Fact]
        public void FromException_Timeout_IsRetriableTimeout()
        {
            var diagnostic = TagGroupFailureDiagnostic.FromException(new TimeoutException("scan timeout"));

            Assert.Equal(TagGroupFailureCategory.Timeout, diagnostic.Category);
            Assert.True(diagnostic.Retriable);
            Assert.Null(diagnostic.StatusCode);
        }

        [Fact]
        public void FromException_NotConnected_IsRetriableNetwork()
        {
            var diagnostic = TagGroupFailureDiagnostic.FromException(
                new InvalidOperationException("Not connected to PLC"));

            Assert.Equal(TagGroupFailureCategory.Network, diagnostic.Category);
            Assert.True(diagnostic.Retriable);
            Assert.Null(diagnostic.StatusCode);
        }

        [Fact]
        public void FromException_Argument_IsDataNotRetriable()
        {
            var diagnostic = TagGroupFailureDiagnostic.FromException(
                new ArgumentException("unsupported conversion"));

            Assert.Equal(TagGroupFailureCategory.Data, diagnostic.Category);
            Assert.False(diagnostic.Retriable);
            Assert.Null(diagnostic.StatusCode);
        }
    }
}
