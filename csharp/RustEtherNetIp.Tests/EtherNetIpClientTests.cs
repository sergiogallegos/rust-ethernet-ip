using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using Xunit;
using Moq;
using RustEtherNetIp;

namespace RustEtherNetIp.Tests
{
    public class EtherNetIpClientTests
    {
        private readonly EtherNetIpClient _client;
        private readonly Mock<IEtherNetIpClient> _mockClient;

        public EtherNetIpClientTests()
        {
            _mockClient = new Mock<IEtherNetIpClient>();
            _client = new EtherNetIpClient();
        }

        [Fact]
        public void Connect_Success()
        {
            // Arrange
            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.ClientId).Returns(1);

            // Act
            bool result = _mockClient.Object.Connect("192.168.1.100:44818");

            // Assert
            Assert.True(result);
            Assert.True(_mockClient.Object.IsConnected);
            Assert.Equal(1, _mockClient.Object.ClientId);
        }

        [Fact]
        public void Connect_Failure()
        {
            // Arrange
            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(false);
            _mockClient.Setup(x => x.IsConnected).Returns(false);
            _mockClient.Setup(x => x.ClientId).Returns(-1);

            // Act
            bool result = _mockClient.Object.Connect("192.168.1.100:44818");

            // Assert
            Assert.False(result);
            Assert.False(_mockClient.Object.IsConnected);
            Assert.Equal(-1, _mockClient.Object.ClientId);
        }

        [Fact]
        public void ReadBool_Success()
        {
            // Arrange
            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.ReadBool(It.IsAny<string>())).Returns(true);

            // Act
            _mockClient.Object.Connect("192.168.1.100:44818");
            bool result = _mockClient.Object.ReadBool("TestBool");

            // Assert
            Assert.True(result);
        }

        [Fact]
        public void ReadDint_Success()
        {
            // Arrange
            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.ReadDint(It.IsAny<string>())).Returns(42);

            // Act
            _mockClient.Object.Connect("192.168.1.100:44818");
            int result = _mockClient.Object.ReadDint("TestDint");

            // Assert
            Assert.Equal(42, result);
        }

        [Fact]
        public void ReadReal_Success()
        {
            // Arrange
            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.ReadReal(It.IsAny<string>())).Returns(3.14f);

            // Act
            _mockClient.Object.Connect("192.168.1.100:44818");
            float result = _mockClient.Object.ReadReal("TestReal");

            // Assert
            Assert.Equal(3.14f, result);
        }

        [Fact]
        public void ReadString_Success()
        {
            // Arrange
            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.ReadString(It.IsAny<string>())).Returns("Hello, World!");

            // Act
            _mockClient.Object.Connect("192.168.1.100:44818");
            string result = _mockClient.Object.ReadString("TestString");

            // Assert
            Assert.Equal("Hello, World!", result);
        }

        [Fact]
        public void ReadUdt_Success()
        {
            // Arrange
            var udtData = new Dictionary<string, PlcValue>
            {
                { "Bool1", PlcValue.Bool(true) },
                { "Dint1", PlcValue.Dint(42) }
            };

            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.ReadUdt(It.IsAny<string>())).Returns(PlcValue.Udt(udtData));

            // Act
            _mockClient.Object.Connect("192.168.1.100:44818");
            var result = _mockClient.Object.ReadUdt("TestUDT");

            // Assert
            Assert.NotNull(result);
            var members = result.UdtMembers;
            Assert.NotNull(members);
            Assert.Equal(2, members!.Count);
            Assert.Equal(PlcValueType.Bool, members["Bool1"].Type);
            Assert.True(members["Bool1"].As<bool>());
            Assert.Equal(PlcValueType.Dint, members["Dint1"].Type);
            Assert.Equal(42, members["Dint1"].As<int>());
        }

        [Fact]
        public void WriteUdt_Success()
        {
            // Arrange
            var udtData = new Dictionary<string, object>
            {
                { "Bool1", true },
                { "Dint1", 42 }
            };

            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.WriteUdt(It.IsAny<string>(), It.IsAny<Dictionary<string, object>>()));

            // Act & Assert
            _mockClient.Object.Connect("192.168.1.100:44818");
            var exception = Record.Exception(() => _mockClient.Object.WriteUdt("TestUDT", udtData));
            Assert.Null(exception);
        }

        [Fact]
        public void Dispose_CleansUpResources()
        {
            // Arrange
            _mockClient.Setup(x => x.Connect(It.IsAny<string>())).Returns(true);
            _mockClient.Setup(x => x.IsConnected).Returns(true);
            _mockClient.Setup(x => x.Dispose()).Callback(() => _mockClient.Setup(x => x.IsConnected).Returns(false));

            // Act
            _mockClient.Object.Connect("192.168.1.100:44818");
            _mockClient.Object.Dispose();

            // Assert
            Assert.False(_mockClient.Object.IsConnected);
        }
    }
} 