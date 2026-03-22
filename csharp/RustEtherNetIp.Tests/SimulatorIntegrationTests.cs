using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using Xunit;

namespace RustEtherNetIp.Tests
{
    public class SimulatorIntegrationTests
    {
        private const string SimAddressEnv = "SIM_PLC_ADDRESS";

        [Fact]
        public void ReadWriteDint_WithSimulator()
        {
            var address = Environment.GetEnvironmentVariable(SimAddressEnv);
            if (string.IsNullOrWhiteSpace(address))
            {
                // Simulator not configured; skip without failing.
                return;
            }

            var nativeLibName = RuntimeInformation.IsOSPlatform(OSPlatform.OSX)
                ? "librust_ethernet_ip.dylib"
                : "rust_ethernet_ip.dll";
            var nativeLibPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, nativeLibName);
            if (!File.Exists(nativeLibPath))
            {
                return;
            }

            using var client = new EtherNetIpClient();
            Assert.True(client.Connect(address));

            var initialDint = client.ReadDint("DINT_TAG");
            Assert.Equal(1234, initialDint);

            var initialBool = client.ReadBool("BOOL_TAG");
            Assert.True(initialBool);

            var initialReal = client.ReadReal("REAL_TAG");
            Assert.Equal(3.0f, initialReal, 2);

            var initialString = client.ReadString("STRING_TAG");
            Assert.Equal("Hello PLC", initialString);

            var initialArrayElement = client.ReadDint("DINT_ARRAY[1]");
            Assert.Equal(20, initialArrayElement);

            var initialDintRange = client.ReadDintArrayRange("DINT_ARRAY", 0, 2);
            Assert.Equal(new[] { 10, 20 }, initialDintRange);

            var initialRealRange = client.ReadRealArrayRange("REAL_ARRAY", 0, 2);
            Assert.Equal(new[] { 1.5f, 2.5f }, initialRealRange);

            client.WriteDint("DINT_TAG", 4321);
            client.WriteBool("BOOL_TAG", false);
            client.WriteReal("REAL_TAG", 6.28f);
            client.WriteString("STRING_TAG", "Updated");
            client.WriteDint("DINT_ARRAY[1]", 55);

            var updatedDint = client.ReadDint("DINT_TAG");
            Assert.Equal(4321, updatedDint);

            var updatedBool = client.ReadBool("BOOL_TAG");
            Assert.False(updatedBool);

            var updatedReal = client.ReadReal("REAL_TAG");
            Assert.Equal(6.28f, updatedReal, 2);

            var updatedString = client.ReadString("STRING_TAG");
            Assert.Equal("Updated", updatedString);

            var updatedArrayElement = client.ReadDint("DINT_ARRAY[1]");
            Assert.Equal(55, updatedArrayElement);

            var updatedDintRange = client.ReadDintArrayRange("DINT_ARRAY", 0, 2);
            Assert.Equal(new[] { 10, 55 }, updatedDintRange);
        }

        [Fact]
        public async Task TagGroupPollingEvent_ReportsPartialError_WithMixedValidAndInvalidTags()
        {
            var address = Environment.GetEnvironmentVariable(SimAddressEnv);
            if (string.IsNullOrWhiteSpace(address))
            {
                // Simulator not configured; skip without failing.
                return;
            }

            var nativeLibName = RuntimeInformation.IsOSPlatform(OSPlatform.OSX)
                ? "librust_ethernet_ip.dylib"
                : "rust_ethernet_ip.dll";
            var nativeLibPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, nativeLibName);
            if (!File.Exists(nativeLibPath))
            {
                return;
            }

            using var client = new EtherNetIpClient();
            Assert.True(client.Connect(address));

            client.UpsertTagGroup("diag", new[] { "DINT_TAG", "THIS_TAG_DOES_NOT_EXIST" }, 100);
            var group = client.SubscribeToTagGroup("diag");

            var tcs = new TaskCompletionSource<TagGroupPollingEventArgs>(
                TaskCreationOptions.RunContinuationsAsynchronously);
            EventHandler<TagGroupPollingEventArgs>? handler = null;
            handler = (_, evt) =>
            {
                if (evt.Kind == TagGroupEventKind.PartialError)
                {
                    tcs.TrySetResult(evt);
                }
            };

            group.PollingEvent += handler;
            try
            {
                var completed = await Task.WhenAny(tcs.Task, Task.Delay(TimeSpan.FromSeconds(3)));
                Assert.True(completed == tcs.Task, "Timed out waiting for PartialError polling event");

                var evt = await tcs.Task;
                Assert.Equal(TagGroupEventKind.PartialError, evt.Kind);
                Assert.True(evt.AllValues.ContainsKey("DINT_TAG"));
                Assert.True(evt.Errors.ContainsKey("THIS_TAG_DOES_NOT_EXIST"));
            }
            finally
            {
                group.PollingEvent -= handler;
                client.RemoveTagGroup("diag");
            }
        }

        [Fact]
        public async Task TagGroupPollingEvent_ReportsReadFailure_WhenClientDisconnects()
        {
            var address = Environment.GetEnvironmentVariable(SimAddressEnv);
            if (string.IsNullOrWhiteSpace(address))
            {
                // Simulator not configured; skip without failing.
                return;
            }

            var nativeLibName = RuntimeInformation.IsOSPlatform(OSPlatform.OSX)
                ? "librust_ethernet_ip.dylib"
                : "rust_ethernet_ip.dll";
            var nativeLibPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, nativeLibName);
            if (!File.Exists(nativeLibPath))
            {
                return;
            }

            using var client = new EtherNetIpClient();
            Assert.True(client.Connect(address));

            client.UpsertTagGroup("read-failure", new[] { "DINT_TAG" }, 100);
            var group = client.SubscribeToTagGroup("read-failure");

            var tcs = new TaskCompletionSource<TagGroupPollingEventArgs>(
                TaskCreationOptions.RunContinuationsAsynchronously);
            EventHandler<TagGroupPollingEventArgs>? handler = null;
            handler = (_, evt) =>
            {
                if (evt.Kind == TagGroupEventKind.ReadFailure)
                {
                    tcs.TrySetResult(evt);
                }
            };

            group.PollingEvent += handler;
            try
            {
                client.Disconnect();

                var completed = await Task.WhenAny(tcs.Task, Task.Delay(TimeSpan.FromSeconds(3)));
                Assert.True(completed == tcs.Task, "Timed out waiting for ReadFailure polling event");

                var evt = await tcs.Task;
                Assert.Equal(TagGroupEventKind.ReadFailure, evt.Kind);
                Assert.NotNull(evt.ErrorMessage);
                Assert.NotNull(evt.Failure);
                Assert.Equal(TagGroupFailureCategory.Network, evt.Failure!.Category);
                Assert.True(evt.Failure.Retriable);
            }
            finally
            {
                group.PollingEvent -= handler;
                client.RemoveTagGroup("read-failure");
            }
        }
    }
}
