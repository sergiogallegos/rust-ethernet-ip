using Xunit;

namespace RustEtherNetIp.Tests;

public class Utf8MarshallingTests
{
    [Fact]
    public void AllocUtf8_RoundTripsNonAsciiText()
    {
        var expected = "Grüße_Ω";
        var ptr = EtherNetIpClient.AllocUtf8(expected);

        try
        {
            Assert.Equal(expected, EtherNetIpClient.PtrToStringUtf8Safe(ptr));
        }
        finally
        {
            EtherNetIpClient.FreeUtf8(ptr);
        }
    }
}
