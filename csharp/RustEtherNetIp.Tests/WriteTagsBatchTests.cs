using System;
using System.Collections.Generic;
using Xunit;
using RustEtherNetIp;

namespace RustEtherNetIp.Tests
{
    public class WriteTagsBatchTests
    {
        private class TestClient : EthernetNetIpClient
        {
            public bool WasCalled { get; private set; }
            public string? TagNameArg { get; private set; }
            public string? MemberPathArg { get; private set; }
            public PlcValue? ValueArg { get; private set; }

            public override void SetUdtMember(string tagName, string memberPath, PlcValue value)
            {
                WasCalled = true;
                TagNameArg = tagName;
                MemberPathArg = memberPath;
                ValueArg = value;
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
            Assert.Equal("gTestUDT", client.TagNameArg);
            Assert.Equal("Member1_DINT", client.MemberPathArg);
            Assert.Equal(PlcValueType.Dint, client.ValueArg?.Type);
            Assert.Equal(123, client.ValueArg?.As<int>());
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
            Assert.Equal("gTestUDT_Array[0]", client.TagNameArg);
            Assert.Equal("Member1_DINT", client.MemberPathArg);
            Assert.Equal(PlcValueType.Dint, client.ValueArg?.Type);
            Assert.Equal(500, client.ValueArg?.As<int>());
        }
    }
}