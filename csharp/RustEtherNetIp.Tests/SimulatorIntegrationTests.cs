using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Xunit;

namespace RustEtherNetIp.Tests
{
    public class SimulatorIntegrationTests
    {
        [Fact]
        public void ReadWriteScalarsAndArrayRanges_WithSimulator()
        {
            using var sim = new SimulatorTestHarness();
            using var client = sim.ConnectClient();

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
        public async Task AsyncWrappers_RoundTripThroughNativeWrapper_WithSimulator()
        {
            using var sim = new SimulatorTestHarness();
            using var client = sim.ConnectClient();

            Assert.Equal(1234, await client.ReadDintAsync("DINT_TAG"));

            await client.WriteDintAsync("DINT_TAG", 9999);
            Assert.Equal(9999, await client.ReadDintAsync("DINT_TAG"));

            await client.WriteBoolAsync("BOOL_TAG", false);
            Assert.False(await client.ReadBoolAsync("BOOL_TAG"));

            Assert.True(await client.CheckHealthAsync());
        }

        [Fact]
        public void BatchReadWriteAndExecuteBatch_PreservePerTagResults_WithSimulator()
        {
            using var sim = new SimulatorTestHarness();
            using var client = sim.ConnectClient();

            var initial = client.ReadTagsBatch(new[] { "DINT_TAG", "REAL_TAG", "BOOL_TAG", "STRING_TAG" });
            Assert.All(initial.Values, result => Assert.True(result.Success, result.ErrorMessage));
            Assert.Equal(1234, initial["DINT_TAG"].Value);
            Assert.Equal(3.0f, Assert.IsType<float>(initial["REAL_TAG"].Value), 2);
            Assert.Equal(true, initial["BOOL_TAG"].Value);
            Assert.Equal("Hello PLC", initial["STRING_TAG"].Value);

            var failedRead = client.ReadTagsBatch(new[] { "DINT_TAG", "THIS_TAG_DOES_NOT_EXIST" });
            Assert.True(failedRead["DINT_TAG"].Success);
            Assert.False(failedRead["THIS_TAG_DOES_NOT_EXIST"].Success);
            Assert.NotNull(failedRead["THIS_TAG_DOES_NOT_EXIST"].ErrorMessage);

            var writeResults = client.WriteTagsBatch(new Dictionary<string, object>
            {
                ["DINT_TAG"] = 2468,
                ["REAL_TAG"] = 6.5f,
                ["BOOL_TAG"] = false,
                ["STRING_TAG"] = "Batch Updated"
            });
            Assert.All(writeResults.Values, result => Assert.True(result.Success, result.ErrorMessage));

            var updated = client.ReadTagsBatch(new[] { "DINT_TAG", "REAL_TAG", "BOOL_TAG", "STRING_TAG" });
            Assert.Equal(2468, updated["DINT_TAG"].Value);
            Assert.Equal(6.5f, Assert.IsType<float>(updated["REAL_TAG"].Value), 2);
            Assert.Equal(false, updated["BOOL_TAG"].Value);
            Assert.Equal("Batch Updated", updated["STRING_TAG"].Value);

            var executeResults = client.ExecuteBatch(new[]
            {
                BatchOperation.Write("DINT_TAG", 1357),
                BatchOperation.Read("DINT_TAG"),
                BatchOperation.Read("THIS_TAG_DOES_NOT_EXIST"),
            });

            Assert.Equal(3, executeResults.Length);
            Assert.True(executeResults[0].Success, executeResults[0].ErrorMessage);
            Assert.True(executeResults[1].Success, executeResults[1].ErrorMessage);
            Assert.Equal(1357, executeResults[1].Value);
            Assert.False(executeResults[2].Success);
            Assert.NotNull(executeResults[2].ErrorMessage);
        }

        [Fact]
        public void RouteConnectAndDiagnostics_WorkThroughNativeWrapper_WithSimulator()
        {
            using var sim = new SimulatorTestHarness();
            using var client = sim.ConnectClientWithRoute(new RoutePath().AddSlot(0));

            Assert.True(client.CheckHealth());
            Assert.True(client.CheckHealthDetailed());

            var snapshot = client.GetDiagnosticsSnapshotDetailed();
            Assert.True(snapshot.Connections.ActiveConnections >= 1);
            Assert.Contains(snapshot.Health.OverallHealth, new[]
            {
                DiagnosticsHealthStatus.Healthy,
                DiagnosticsHealthStatus.Warning,
                DiagnosticsHealthStatus.Critical,
                DiagnosticsHealthStatus.Unknown
            });
            Assert.Contains(snapshot.Health.HealthMode, new[]
            {
                DiagnosticsHealthMode.Passive,
                DiagnosticsHealthMode.Verified
            });
        }

        [Fact]
        public async Task TagGroupPollingEvent_ReportsPartialError_WithMixedValidAndInvalidTags()
        {
            using var sim = new SimulatorTestHarness();
            using var client = sim.ConnectClient();
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
        public async Task TagGroupPollingEvent_ReportsErrors_WhenClientDisconnects()
        {
            using var sim = new SimulatorTestHarness();
            using var client = sim.ConnectClient();
            client.UpsertTagGroup("read-failure", new[] { "DINT_TAG" }, 100);
            var group = client.SubscribeToTagGroup("read-failure");

            var tcs = new TaskCompletionSource<TagGroupPollingEventArgs>(
                TaskCreationOptions.RunContinuationsAsynchronously);
            EventHandler<TagGroupPollingEventArgs>? handler = null;
            handler = (_, evt) =>
            {
                if (evt.Kind == TagGroupEventKind.PartialError || evt.Kind == TagGroupEventKind.ReadFailure)
                {
                    tcs.TrySetResult(evt);
                }
            };

            group.PollingEvent += handler;
            try
            {
                client.Disconnect();

                var completed = await Task.WhenAny(tcs.Task, Task.Delay(TimeSpan.FromSeconds(3)));
                Assert.True(completed == tcs.Task, "Timed out waiting for polling error event");

                var evt = await tcs.Task;
                Assert.Contains(evt.Kind, new[] { TagGroupEventKind.PartialError, TagGroupEventKind.ReadFailure });
                Assert.True(evt.Errors.Count > 0 || !string.IsNullOrWhiteSpace(evt.ErrorMessage));
                if (evt.Kind == TagGroupEventKind.ReadFailure)
                {
                    Assert.NotNull(evt.ErrorMessage);
                    Assert.NotNull(evt.Failure);
                    Assert.Equal(TagGroupFailureCategory.Network, evt.Failure!.Category);
                    Assert.True(evt.Failure.Retriable);
                }
                else
                {
                    Assert.True(evt.Errors.ContainsKey("DINT_TAG"));
                }
            }
            finally
            {
                group.PollingEvent -= handler;
                client.RemoveTagGroup("read-failure");
            }
        }
    }
}
