using System;
using System.IO;
using System.Runtime.InteropServices;
using Xunit;

namespace RustEtherNetIp.Tests
{
    public class SimulatorIntegrationTests
    {
        private const string SimAddressEnv = "SIM_PLC_ADDRESS";

        [Fact]
        public void ReadWriteDint_WithSimulator()
        {
            var address = Environment.GetEnvironmentVariable(SimAddressEnv);
            if (string.IsNullOrWhiteSpace(address))
            {
                // Simulator not configured; skip without failing.
                return;
            }

            var nativeLibName = RuntimeInformation.IsOSPlatform(OSPlatform.OSX)
                ? "librust_ethernet_ip.dylib"
                : "rust_ethernet_ip.dll";
            var nativeLibPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, nativeLibName);
            if (!File.Exists(nativeLibPath))
            {
                return;
            }

            using var client = new EtherNetIpClient();
            Assert.True(client.Connect(address));

            var initialDint = client.ReadDint("DINT_TAG");
            Assert.Equal(1234, initialDint);

            var initialBool = client.ReadBool("BOOL_TAG");
            Assert.True(initialBool);

            var initialReal = client.ReadReal("REAL_TAG");
            Assert.Equal(3.0f, initialReal, 2);

            var initialString = client.ReadString("STRING_TAG");
            Assert.Equal("Hello PLC", initialString);

            var initialArrayElement = client.ReadDint("DINT_ARRAY[1]");
            Assert.Equal(20, initialArrayElement);

            var initialDintRange = client.ReadDintArrayRange("DINT_ARRAY", 0, 2);
            Assert.Equal(new[] { 10, 20 }, initialDintRange);

            var initialRealRange = client.ReadRealArrayRange("REAL_ARRAY", 0, 2);
            Assert.Equal(new[] { 1.5f, 2.5f }, initialRealRange);

            client.WriteDint("DINT_TAG", 4321);
            client.WriteBool("BOOL_TAG", false);
            client.WriteReal("REAL_TAG", 6.28f);
            client.WriteString("STRING_TAG", "Updated");
            client.WriteDint("DINT_ARRAY[1]", 55);

            var updatedDint = client.ReadDint("DINT_TAG");
            Assert.Equal(4321, updatedDint);

            var updatedBool = client.ReadBool("BOOL_TAG");
            Assert.False(updatedBool);

            var updatedReal = client.ReadReal("REAL_TAG");
            Assert.Equal(6.28f, updatedReal, 2);

            var updatedString = client.ReadString("STRING_TAG");
            Assert.Equal("Updated", updatedString);

            var updatedArrayElement = client.ReadDint("DINT_ARRAY[1]");
            Assert.Equal(55, updatedArrayElement);

            var updatedDintRange = client.ReadDintArrayRange("DINT_ARRAY", 0, 2);
            Assert.Equal(new[] { 10, 55 }, updatedDintRange);
        }
    }
}
