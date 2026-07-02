using Xunit;
using System.Net;
using System.Net.Sockets;

namespace RustEtherNetIp.IntegrationTests;

public sealed class NativePinvokeIntegrationTests : IClassFixture<PlcSimulatorFixture>
{
    private readonly PlcSimulatorFixture _simulator;

    public NativePinvokeIntegrationTests(PlcSimulatorFixture simulator)
    {
        _simulator = simulator;
    }

    [Fact]
    public void ConnectAndDisconnect_UsesRealNativeBoundary()
    {
        using var client = CreateConnectedClient();

        Assert.True(client.IsConnected);
        Assert.Null(client.LastConnectError);

        client.Disconnect();

        Assert.False(client.IsConnected);
    }

    [Fact]
    public void ConnectFailure_PopulatesLastConnectError()
    {
        using var listener = new TcpListener(IPAddress.Loopback, 0);
        listener.Start();
        int unusedPort = ((IPEndPoint)listener.LocalEndpoint).Port;
        listener.Stop();

        using var client = new EtherNetIpClient();

        Assert.False(client.Connect($"127.0.0.1:{unusedPort}"));
        Assert.Contains("code -1", client.LastConnectError);
    }

    [Fact]
    public void ScalarReadsAndWrites_RoundTripThroughSimulator()
    {
        using var client = CreateConnectedClient();

        client.WriteDint("DINT_TAG", 5678);
        Assert.Equal(5678, client.ReadDint("DINT_TAG"));

        client.WriteReal("REAL_TAG", 4.25f);
        Assert.Equal(4.25f, client.ReadReal("REAL_TAG"), precision: 3);

        client.WriteBool("BOOL_TAG", false);
        Assert.False(client.ReadBool("BOOL_TAG"));
    }

    [Fact]
    public void StringWriteAndRead_PreservesUtf8Text()
    {
        using var client = CreateConnectedClient();
        const string expected = "Grüße_Ω";

        client.WriteString("STRING_TAG", expected);

        Assert.Equal(expected, client.ReadString("STRING_TAG"));
    }

    [Fact]
    public void NativeBatchReadWrite_RoundTripsScalars()
    {
        using var client = CreateConnectedClient();

        var writeResults = client.WriteTagsBatch(new Dictionary<string, object>
        {
            ["DINT_TAG"] = 2468,
            ["REAL_TAG"] = 9.5f
        });

        Assert.True(writeResults["DINT_TAG"].Success, writeResults["DINT_TAG"].ErrorMessage);
        Assert.True(writeResults["REAL_TAG"].Success, writeResults["REAL_TAG"].ErrorMessage);

        var readResults = client.ReadTagsBatch(new[] { "DINT_TAG", "REAL_TAG" });

        Assert.True(readResults["DINT_TAG"].Success, readResults["DINT_TAG"].ErrorMessage);
        Assert.Equal(2468, Assert.IsType<int>(readResults["DINT_TAG"].Value));
        Assert.True(readResults["REAL_TAG"].Success, readResults["REAL_TAG"].ErrorMessage);
        Assert.Equal(9.5f, Assert.IsType<float>(readResults["REAL_TAG"].Value), precision: 3);
    }

    [Fact]
    public async Task WriteUdtMember_CompletesInsteadOfDeadlocking()
    {
        using var client = CreateConnectedClient();

        var task = Task.Run(() =>
        {
            var exception = Assert.Throws<InvalidOperationException>(() =>
                client.WriteUdtMember("UDT_TAG", "Member1_DINT", PlcValue.Dint(7)));
            Assert.Contains("raw data", exception.Message, StringComparison.OrdinalIgnoreCase);
        });

        await task.WaitAsync(TimeSpan.FromSeconds(30));
    }

    [Fact]
    public void RapidReads_DoNotOverlapKeepAlive()
    {
        using var client = new EtherNetIpClient
        {
            KeepAliveInterval = TimeSpan.FromMilliseconds(50)
        };
        Assert.True(client.Connect(_simulator.Address), client.LastConnectError);
        client.WriteDint("DINT_TAG", 1234);

        for (int i = 0; i < 200; i++)
            Assert.Equal(1234, client.ReadDint("DINT_TAG"));
    }

    private EtherNetIpClient CreateConnectedClient()
    {
        var client = new EtherNetIpClient();
        Assert.True(client.Connect(_simulator.Address), client.LastConnectError);
        return client;
    }
}
