using System;
using RustEtherNetIp;
using Xunit;

namespace RustEtherNetIp.Tests
{
    public class UdtDataContractTests
    {
        [Fact]
        public void ToDictionary_ThrowsNotSupportedUntilImplemented()
        {
            var data = new UdtData(1, new byte[] { 0x01, 0x02, 0x03, 0x04 });
            var template = UdtTemplateFactory.CreateGenericTemplate("ExampleUdt", 4);

            var ex = Assert.Throws<NotSupportedException>(() => data.ToDictionary(template));
            Assert.Contains("not implemented", ex.Message, StringComparison.OrdinalIgnoreCase);
        }
    }
}
