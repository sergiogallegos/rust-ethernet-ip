using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using RustEtherNetIp;

namespace CSharpWrapperSmoke
{
    internal static class Program
    {
        private static void Assert(bool condition, string message)
        {
            if (!condition)
                throw new Exception(message);
        }

        private static void Main()
        {
            var address = Environment.GetEnvironmentVariable("TEST_PLC_ADDRESS") ?? "192.168.0.1:44818";
            var slot = byte.TryParse(Environment.GetEnvironmentVariable("TEST_PLC_SLOT"), out var parsedSlot)
                ? parsedSlot
                : (byte)0;
            var route = new RoutePath().AddSlot(slot);

            using var client = new EtherNetIpClient();
            Assert(client.ConnectWithRoute(address, route), $"ConnectWithRoute failed for {address}");
            Assert(client.CheckHealth(), "CheckHealth returned false");

            var originalDint = client.ReadDint("gTestArray_DINT[5]");
            var originalReal = client.ReadReal("gTestArray_REAL[0]");
            var originalBool = client.ReadBool("gTestArray_BOOL[0]");
            var originalInt = client.ReadInt("gTestArray_INT[0]");
            var originalProgramDint = client.ReadDint("Program:TestProgram.gTestArray_DINT[5]");

            client.WriteDint("gTestArray_DINT[5]", 123456);
            Assert(client.ReadDint("gTestArray_DINT[5]") == 123456, "DINT write/readback failed");

            client.WriteReal("gTestArray_REAL[0]", 12.34f);
            Assert(Math.Abs(client.ReadReal("gTestArray_REAL[0]") - 12.34f) < 0.01f, "REAL write/readback failed");

            client.WriteBool("gTestArray_BOOL[0]", !originalBool);
            Assert(client.ReadBool("gTestArray_BOOL[0]") == !originalBool, "BOOL write/readback failed");

            client.WriteInt("gTestArray_INT[0]", 2345);
            Assert(client.ReadInt("gTestArray_INT[0]") == 2345, "INT write/readback failed");

            client.WriteDint("Program:TestProgram.gTestArray_DINT[5]", 654321);
            Assert(client.ReadDint("Program:TestProgram.gTestArray_DINT[5]") == 654321, "Program DINT write/readback failed");

            var batchRead = client.ReadTagsBatch(new[]
            {
                "gTestArray_DINT[0]",
                "gTestArray_REAL[0]",
                "gTestArray_BOOL[0]",
                "gTestArray_INT[0]",
                "gTestUDT.Member1_DINT"
            });
            Assert(batchRead.Count == 5 && batchRead.Values.All(v => v.Success), "ReadTagsBatch failed");

            var batchWrite = client.WriteTagsBatch(new Dictionary<string, object>
            {
                ["gTestArray_DINT[5]"] = 200001,
                ["gTestArray_DINT[6]"] = 200002,
                ["gTestArray_DINT[7]"] = 200003
            });
            Assert(batchWrite.Values.All(v => v.Success), "WriteTagsBatch failed");

            var execute = client.ExecuteBatch(new[]
            {
                BatchOperation.Read("gTestArray_DINT[0]"),
                BatchOperation.Write("gTestArray_DINT[5]", 300001),
                BatchOperation.Read("gTestArray_REAL[0]"),
                BatchOperation.Read("gTestUDT.Member1_DINT")
            });
            Assert(execute.All(v => v.Success), "ExecuteBatch failed");

            using (var subscriptionReady = new ManualResetEventSlim(false))
            {
                var subscription = client.SubscribeToTag("gTestArray_DINT[5]");
                subscription.ValueChanged += (_, _) => subscriptionReady.Set();
                Assert(subscription.Value is int, "Subscription did not populate initial value");
                client.WriteDint("gTestArray_DINT[5]", 300123);
                Assert(subscriptionReady.Wait(1500), "Subscription did not emit a value-changed event");
                client.UnsubscribeFromTag("gTestArray_DINT[5]");
            }

            bool invalidSubscriptionFailed = false;
            try
            {
                client.SubscribeToTag("NonExistentTag");
            }
            catch
            {
                invalidSubscriptionFailed = true;
            }
            Assert(invalidSubscriptionFailed, "Invalid tag subscription did not fail fast");

            client.UpsertTagGroup("valid_group", new[]
            {
                "gTestArray_DINT[0]",
                "gTestArray_REAL[0]",
                "gTestArray_BOOL[0]"
            }, 200);
            var validSnapshot = client.ReadTagGroupOnce("valid_group");
            Assert(!validSnapshot.HasErrors && validSnapshot.Values.Count == 3, "ReadTagGroupOnce valid group failed");

            client.UpsertTagGroup("partial_group", new[]
            {
                "gTestArray_DINT[0]",
                "NonExistentTag"
            }, 200);
            var partialSnapshot = client.ReadTagGroupOnce("partial_group");
            Assert(partialSnapshot.HasErrors, "ReadTagGroupOnce partial group did not report errors");

            using (var groupEventReady = new ManualResetEventSlim(false))
            {
                var group = client.SubscribeToTagGroup("valid_group");
                group.PollingEvent += (_, evt) =>
                {
                    if (evt.Kind == TagGroupEventKind.Data)
                        groupEventReady.Set();
                };

                Assert(groupEventReady.Wait(2000), "SubscribeToTagGroup did not emit a Data event");
                group.Stop();
            }

            client.WriteDint("gTestArray_DINT[5]", originalDint);
            client.WriteReal("gTestArray_REAL[0]", originalReal);
            client.WriteBool("gTestArray_BOOL[0]", originalBool);
            client.WriteInt("gTestArray_INT[0]", originalInt);
            client.WriteDint("Program:TestProgram.gTestArray_DINT[5]", originalProgramDint);

            Console.WriteLine("C# wrapper smoke validation passed.");
        }
    }
}
