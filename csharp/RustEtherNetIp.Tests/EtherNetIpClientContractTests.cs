using System;
using Xunit;

namespace RustEtherNetIp.Tests
{
    public class EtherNetIpClientContractTests
    {
        [Fact]
        public void Operations_WhenNotConnected_ThrowInvalidOperationException()
        {
            using var client = new EtherNetIpClient();

            Assert.False(client.IsConnected);
            Assert.False(client.CheckHealth());
            Assert.False(client.CheckHealthDetailed());
            Assert.Throws<InvalidOperationException>(() => client.ReadBool("BOOL_TAG"));
            Assert.Throws<InvalidOperationException>(() => client.WriteDint("DINT_TAG", 1));
            var batchResult = client.ReadTagsBatch(new[] { "DINT_TAG" });
            Assert.False(batchResult["DINT_TAG"].Success);
            Assert.Contains("Not connected", batchResult["DINT_TAG"].ErrorMessage, StringComparison.OrdinalIgnoreCase);
            Assert.Throws<InvalidOperationException>(() => client.GetDiagnosticsSnapshot());
            Assert.Throws<InvalidOperationException>(() => client.GetDiagnosticsSnapshotDetailed());
        }

        [Fact]
        public void BatchApis_ValidateNullAndEmptyInputsBeforeNativeCalls()
        {
            using var client = new EtherNetIpClient();

            Assert.Throws<ArgumentException>(() => client.ReadTagsBatch(null!));
            Assert.Throws<ArgumentException>(() => client.ReadTagsBatch(Array.Empty<string>()));
            Assert.Throws<ArgumentException>(() => client.WriteTagsBatch(null!));
            Assert.Throws<ArgumentException>(() => client.WriteTagsBatch(new System.Collections.Generic.Dictionary<string, object>()));
            Assert.Throws<ArgumentException>(() => client.ExecuteBatch(null!));
            Assert.Throws<ArgumentException>(() => client.ExecuteBatch(Array.Empty<BatchOperation>()));
        }

        [Fact]
        public void ConnectWithRoute_RejectsNullRoutePathBeforeNativeCall()
        {
            using var client = new EtherNetIpClient();

            Assert.Throws<ArgumentNullException>(() => client.ConnectWithRoute("127.0.0.1:44818", null!));
        }

        [Fact]
        public void DisposedClient_ThrowsObjectDisposedExceptionForPublicOperations()
        {
            var client = new EtherNetIpClient();
            client.Dispose();

            Assert.Throws<ObjectDisposedException>(() => client.Connect("127.0.0.1:44818"));
            Assert.Throws<ObjectDisposedException>(() => client.ConnectWithRoute("127.0.0.1:44818", new RoutePath().AddSlot(0)));
            Assert.Throws<ObjectDisposedException>(() => client.ReadBool("BOOL_TAG"));
            Assert.Throws<ObjectDisposedException>(() => client.UpsertTagGroup("Core", new[] { "Tag1" }));
        }

        [Fact]
        public void StatisticsAndClientId_AreSafeBeforeConnection()
        {
            using var client = new EtherNetIpClient();

            Assert.Equal(-1, client.ClientId);
            Assert.NotNull(client.Statistics);
            Assert.False(client.IsConnected);
        }
    }
}
