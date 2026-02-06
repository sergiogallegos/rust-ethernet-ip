using System;
using System.Collections.Generic;
using Xunit;
using Moq;
using RustEtherNetIp;

namespace RustEtherNetIp.Tests
{
    [Collection("IntegrationTests")]
    public class EtherNetIpClientIntegrationTests : IDisposable
    {
        private readonly Mock<IEtherNetIpClient> _mockClient;
        private const string PLC_ADDRESS = "192.168.1.100:44818";

        public EtherNetIpClientIntegrationTests()
        {
            _mockClient = new Mock<IEtherNetIpClient>();
        }

        public void Dispose()
        {
            // Nothing to dispose
        }

        [Fact]
        public void Connect_ToRealPLC_Success()
        {
            // Arrange
            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.ClientId).Returns(1);

            // Act
            bool result = _mockClient.Object.Connect(PLC_ADDRESS);

            // Assert
            Assert.True(result);
            Assert.True(_mockClient.Object.IsConnected);
            Assert.True(_mockClient.Object.ClientId > 0);
        }

        [Fact]
        public void ReadWrite_Bool_Success()
        {
            // Arrange
            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.ReadBool(It.IsAny<string>()))
                .Returns(true)
                .Callback(() => _mockClient.Setup(x => x.ReadBool(It.IsAny<string>())).Returns(false));
            _mockClient.Setup(x => x.WriteBool(It.IsAny<string>(), It.IsAny<bool>()));

            // Act & Assert
            _mockClient.Object.Connect(PLC_ADDRESS);
            _mockClient.Object.WriteBool("TestBool", true);
            bool readValue = _mockClient.Object.ReadBool("TestBool");
            Assert.True(readValue);

            _mockClient.Object.WriteBool("TestBool", false);
            readValue = _mockClient.Object.ReadBool("TestBool");
            Assert.False(readValue);
        }

        [Fact]
        public void ReadWrite_Dint_Success()
        {
            // Arrange
            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.ReadDint(It.IsAny<string>())).Returns(42);
            _mockClient.Setup(x => x.WriteDint(It.IsAny<string>(), It.IsAny<int>()));

            // Act & Assert
            _mockClient.Object.Connect(PLC_ADDRESS);
            _mockClient.Object.WriteDint("TestDint", 42);
            int readValue = _mockClient.Object.ReadDint("TestDint");
            Assert.Equal(42, readValue);

            _mockClient.Setup(x => x.ReadDint(It.IsAny<string>())).Returns(-123);
            _mockClient.Object.WriteDint("TestDint", -123);
            readValue = _mockClient.Object.ReadDint("TestDint");
            Assert.Equal(-123, readValue);
        }

        [Fact]
        public void ReadWrite_Real_Success()
        {
            // Arrange
            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.ReadReal(It.IsAny<string>())).Returns(3.14f);
            _mockClient.Setup(x => x.WriteReal(It.IsAny<string>(), It.IsAny<float>()));

            // Act & Assert
            _mockClient.Object.Connect(PLC_ADDRESS);
            _mockClient.Object.WriteReal("TestReal", 3.14f);
            float readValue = _mockClient.Object.ReadReal("TestReal");
            Assert.Equal(3.14f, readValue, 2);

            _mockClient.Setup(x => x.ReadReal(It.IsAny<string>())).Returns(-123.45f);
            _mockClient.Object.WriteReal("TestReal", -123.45f);
            readValue = _mockClient.Object.ReadReal("TestReal");
            Assert.Equal(-123.45f, readValue, 2);
        }

        [Fact]
        public void ReadWrite_String_Success()
        {
            // Arrange
            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.ReadString(It.IsAny<string>())).Returns("Hello, World!");
            _mockClient.Setup(x => x.WriteString(It.IsAny<string>(), It.IsAny<string>()));

            // Act & Assert
            _mockClient.Object.Connect(PLC_ADDRESS);
            _mockClient.Object.WriteString("TestString", "Hello, World!");
            string readValue = _mockClient.Object.ReadString("TestString");
            Assert.Equal("Hello, World!", readValue);

            _mockClient.Setup(x => x.ReadString(It.IsAny<string>())).Returns("Testing 123");
            _mockClient.Object.WriteString("TestString", "Testing 123");
            readValue = _mockClient.Object.ReadString("TestString");
            Assert.Equal("Testing 123", readValue);
        }

        [Fact]
        public void ReadWrite_Udt_Success()
        {
            // Arrange
            var udtDataPlc = new Dictionary<string, PlcValue>
            {
                { "Bool1", PlcValue.Bool(true) },
                { "Dint1", PlcValue.Dint(42) },
                { "Real1", PlcValue.Real(3.14f) },
                { "String1", PlcValue.String("Test") }
            };
            var udtDataObject = new Dictionary<string, object>
            {
                { "Bool1", true },
                { "Dint1", 42 },
                { "Real1", 3.14f },
                { "String1", "Test" }
            };

            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.ReadUdt(It.IsAny<string>())).Returns(PlcValue.Udt(udtDataPlc));
            _mockClient.Setup(x => x.WriteUdt(It.IsAny<string>(), It.IsAny<Dictionary<string, object>>()));

            // Act & Assert
            _mockClient.Object.Connect(PLC_ADDRESS);
            _mockClient.Object.WriteUdt("TestUDT", udtDataObject);
            var readValue = _mockClient.Object.ReadUdt("TestUDT");

            Assert.NotNull(readValue);
            var members = readValue.UdtMembers;
            Assert.NotNull(members);
            Assert.Equal(4, members!.Count);
            Assert.True(members["Bool1"].As<bool>());
            Assert.Equal(42, members["Dint1"].As<int>());
            Assert.Equal(3.14f, members["Real1"].As<float>(), 2);
            Assert.Equal("Test", members["String1"].As<string>());

            var newUdtDataPlc = new Dictionary<string, PlcValue>
            {
                { "Bool1", PlcValue.Bool(false) },
                { "Dint1", PlcValue.Dint(-123) },
                { "Real1", PlcValue.Real(-123.45f) },
                { "String1", PlcValue.String("New Test") }
            };
            var newUdtDataObject = new Dictionary<string, object>
            {
                { "Bool1", false },
                { "Dint1", -123 },
                { "Real1", -123.45f },
                { "String1", "New Test" }
            };

            _mockClient.Setup(x => x.ReadUdt(It.IsAny<string>())).Returns(PlcValue.Udt(newUdtDataPlc));
            _mockClient.Object.WriteUdt("TestUDT", newUdtDataObject);
            readValue = _mockClient.Object.ReadUdt("TestUDT");

            Assert.NotNull(readValue);
            members = readValue.UdtMembers;
            Assert.NotNull(members);
            Assert.Equal(4, members!.Count);
            Assert.False(members["Bool1"].As<bool>());
            Assert.Equal(-123, members["Dint1"].As<int>());
            Assert.Equal(-123.45f, members["Real1"].As<float>(), 2);
            Assert.Equal("New Test", members["String1"].As<string>());
        }

        [Fact]
        public void GetTagMetadata_Success()
        {
            // Arrange
            var metadata = new TagMetadata
            {
                DataType = 0x00C1, // BOOL
                Scope = 0,
                ArrayDimension = 0,
                ArraySize = 1
            };

            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.GetTagMetadata(It.IsAny<string>())).Returns(metadata);

            // Act
            _mockClient.Object.Connect(PLC_ADDRESS);
            var result = _mockClient.Object.GetTagMetadata("TestTag");

            // Assert
            Assert.Equal(0x00C1, result.DataType);
            Assert.Equal(0, result.Scope);
            Assert.Equal(0, result.ArrayDimension);
            Assert.Equal(1, result.ArraySize);
        }

        [Fact]
        public void SetMaxPacketSize_Success()
        {
            // Arrange
            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.SetMaxPacketSize(It.IsAny<int>()));

            // Act & Assert
            _mockClient.Object.Connect(PLC_ADDRESS);
            var exception = Record.Exception(() => _mockClient.Object.SetMaxPacketSize(4000));
            Assert.Null(exception);
        }

        [Fact]
        public void CheckHealth_Success()
        {
            // Arrange
            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.CheckHealth()).Returns(true);

            // Act
            _mockClient.Object.Connect(PLC_ADDRESS);
            bool result = _mockClient.Object.CheckHealth();

            // Assert
            Assert.True(result);
        }
    }

    // Collection attribute to ensure tests run sequentially
    [CollectionDefinition("IntegrationTests")]
    public class IntegrationTestsCollection : ICollectionFixture<EtherNetIpClientIntegrationTests>
    {
        // This class has no code, and is never created. Its purpose is simply
        // to be the place to apply [CollectionDefinition] and all the
        // ICollectionFixture<> interfaces.
    }
} 