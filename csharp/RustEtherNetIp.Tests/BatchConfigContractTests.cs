using System;
using Xunit;
using RustEtherNetIp;

namespace RustEtherNetIp.Tests
{
    public class BatchConfigContractTests
    {
        [Fact]
        public void ConfigureBatchOperations_Throws_NotSupportedException()
        {
            using var client = new EtherNetIpClient();
            var config = BatchConfig.Default();

            var ex = Assert.Throws<NotSupportedException>(() => client.ConfigureBatchOperations(config));
            Assert.Contains("not implemented", ex.Message, StringComparison.OrdinalIgnoreCase);
        }

        [Fact]
        public void GetBatchConfig_Throws_NotSupportedException()
        {
            using var client = new EtherNetIpClient();

            var ex = Assert.Throws<NotSupportedException>(() => client.GetBatchConfig());
            Assert.Contains("not implemented", ex.Message, StringComparison.OrdinalIgnoreCase);
        }
    }
}
