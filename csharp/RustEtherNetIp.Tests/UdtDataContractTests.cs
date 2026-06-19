using System.Text.Json;
using RustEtherNetIp;
using Xunit;

namespace RustEtherNetIp.Tests
{
    public class UdtDataContractTests
    {
        [Fact]
        public void ToJson_EmitsDataAsByteArray_NotBase64()
        {
            // The native FFI deserializes `data` as a Rust Vec<u8> (a JSON number
            // array). Emitting base64 here made typed UDT writes silently fail.
            var data = new UdtData(7, new byte[] { 0xCF, 0x78, 0x02, 0x00 });

            using var doc = JsonDocument.Parse(data.ToJson());
            var root = doc.RootElement;

            Assert.Equal(7, root.GetProperty("symbol_id").GetInt32());
            var dataElement = root.GetProperty("data");
            Assert.Equal(JsonValueKind.Array, dataElement.ValueKind);
            Assert.Equal(new byte[] { 0xCF, 0x78, 0x02, 0x00 },
                System.Linq.Enumerable.ToArray(
                    System.Linq.Enumerable.Select(dataElement.EnumerateArray(), e => (byte)e.GetInt32())));
        }

        [Fact]
        public void ToJson_RoundTripsThroughFromJson()
        {
            var original = new UdtData(42, new byte[] { 1, 2, 3, 250 });
            var restored = UdtData.FromJson(original.ToJson());

            Assert.Equal(original.SymbolId, restored.SymbolId);
            Assert.Equal(original.Data, restored.Data);
        }

        [Fact]
        public void ToDictionary_ParsesTemplateMembersWithOffsets()
        {
            // BOOL at offset 0, DINT at offset 4 (aligned)
            var raw = new byte[] { 0x01, 0x00, 0x00, 0x00, 0x2A, 0x00, 0x00, 0x00 };
            var data = new UdtData(1, raw);
            var template = new UdtTemplate
            {
                Name = "ExampleUdt",
                TotalSize = raw.Length,
                Members =
                {
                    new UdtMemberTemplate { Name = "Bool1", DataType = "bool", Offset = 0, Size = 1 },
                    new UdtMemberTemplate { Name = "Dint1", DataType = "dint", Offset = 4, Size = 4 }
                }
            };

            var result = data.ToDictionary(template);
            Assert.True(result.ContainsKey("Bool1"));
            Assert.True(result.ContainsKey("Dint1"));
            Assert.True(result["Bool1"].As<bool>());
            Assert.Equal(42, result["Dint1"].As<int>());
        }

        [Fact]
        public void ToDictionary_ThrowsWhenTemplateCannotParseData()
        {
            var data = new UdtData(1, new byte[] { 0x01, 0x02, 0x03, 0x04 });
            var template = new UdtTemplate
            {
                Name = "ExampleUdt",
                TotalSize = 4,
                Members =
                {
                    new UdtMemberTemplate { Name = "TooLarge", DataType = "dint", Offset = 4, Size = 4 }
                }
            };

            Assert.Throws<InvalidOperationException>(() => data.ToDictionary(template));
        }
    }
}
