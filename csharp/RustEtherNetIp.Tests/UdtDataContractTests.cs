using RustEtherNetIp;
using Xunit;

namespace RustEtherNetIp.Tests
{
    public class UdtDataContractTests
    {
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
