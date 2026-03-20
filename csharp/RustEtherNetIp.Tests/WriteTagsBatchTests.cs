using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using Xunit;
using RustEtherNetIp;

namespace RustEtherNetIp.Tests
{
    public class WriteTagsBatchTests
    {
        private class TestClient : EtherNetIpClient
        {
            public bool WasCalled { get; private set; }
            public List<(string TagName, string MemberPath, PlcValue Value)> Calls { get; } = new();

            public override void SetUdtMember(string tagName, string memberPath, PlcValue value)
            {
                WasCalled = true;
                Calls.Add((tagName, memberPath, value));
                // Do NOT call base - avoid real PLC operations in unit tests
            }
        }

        [Fact]
        public void WriteTagsBatch_Uses_SetUdtMember_For_UdtMember()
        {
            var client = new TestClient();

            var tagValues = new Dictionary<string, object>
            {
                { "gTestUDT.Member1_DINT", 123 },
                { "gTestUDT.Member5_String", "Hello PLC" }
            };

            var results = client.WriteTagsBatch(tagValues);

            Assert.True(results.ContainsKey("gTestUDT.Member1_DINT"));
            Assert.True(results.ContainsKey("gTestUDT.Member5_String"));
            Assert.True(results["gTestUDT.Member1_DINT"].Success);
            Assert.True(results["gTestUDT.Member5_String"].Success);

            // Verify SetUdtMember was called at least once (for first member)
            Assert.True(client.WasCalled);
            Assert.Contains(client.Calls, call =>
                call.TagName == "gTestUDT" &&
                call.MemberPath == "Member1_DINT" &&
                call.Value.Type == PlcValueType.Dint &&
                call.Value.As<int>() == 123);
        }

        [Fact]
        public void WriteTagsBatch_Uses_SetUdtMember_For_UdtArrayElement()
        {
            var client = new TestClient();

            var tagValues = new Dictionary<string, object>
            {
                { "gTestUDT_Array[0].Member1_DINT", 500 }
            };

            var results = client.WriteTagsBatch(tagValues);

            Assert.True(results.ContainsKey("gTestUDT_Array[0].Member1_DINT"));
            Assert.True(results["gTestUDT_Array[0].Member1_DINT"].Success);

            Assert.True(client.WasCalled);
            Assert.Contains(client.Calls, call =>
                call.TagName == "gTestUDT_Array[0]" &&
                call.MemberPath == "Member1_DINT" &&
                call.Value.Type == PlcValueType.Dint &&
                call.Value.As<int>() == 500);
        }

        [Fact]
        public void ToRustRawValue_UdtData_Uses_Numeric_Byte_Array()
        {
            var method = typeof(EtherNetIpClient).GetMethod(
                "ToRustRawValue",
                BindingFlags.NonPublic | BindingFlags.Static);

            Assert.NotNull(method);

            var value = PlcValue.UdtFromData(new UdtData(77, new byte[] { 0, 1, 127, 255 }));
            var raw = method!.Invoke(null, new object[] { value });
            var json = JsonSerializer.SerializeToElement(raw);

            Assert.Equal(77, json.GetProperty("symbol_id").GetInt32());

            var data = json.GetProperty("data");
            Assert.Equal(JsonValueKind.Array, data.ValueKind);
            Assert.Equal(new[] { 0, 1, 127, 255 }, data.EnumerateArray().Select(x => x.GetInt32()));
        }
    }
}
