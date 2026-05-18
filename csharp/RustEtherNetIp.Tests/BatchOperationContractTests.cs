using Xunit;

namespace RustEtherNetIp.Tests
{
    public class BatchOperationContractTests
    {
        [Fact]
        public void ReadFactory_CreatesReadOperation()
        {
            var operation = BatchOperation.Read("DINT_TAG");

            Assert.Equal("DINT_TAG", operation.TagName);
            Assert.False(operation.IsWrite);
            Assert.Null(operation.Value);
        }

        [Fact]
        public void WriteFactory_CreatesWriteOperation()
        {
            var operation = BatchOperation.Write("DINT_TAG", 42);

            Assert.Equal("DINT_TAG", operation.TagName);
            Assert.True(operation.IsWrite);
            Assert.Equal(42, operation.Value);
        }

        [Fact]
        public void ResultTypes_DefaultToFailureOrEmptyValues()
        {
            var batchResult = new BatchOperationResult();
            var readResult = new TagReadResultBatch();
            var writeResult = new TagWriteResult();

            Assert.False(batchResult.Success);
            Assert.False(readResult.Success);
            Assert.False(writeResult.Success);
            Assert.Equal(string.Empty, batchResult.TagName);
            Assert.Equal(string.Empty, readResult.TagName);
            Assert.Equal(string.Empty, writeResult.TagName);
        }
    }
}
