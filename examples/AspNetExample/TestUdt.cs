using Microsoft.Extensions.Logging;
using RustEtherNetIp;

namespace AspNetExample
{
    public class TestUdt
    {
        private readonly ILogger<TestUdt> _logger;

        public TestUdt(ILogger<TestUdt> logger)
        {
            _logger = logger;
        }

        public void TestUdtReading()
        {
            try
            {
                _logger.LogInformation("Starting UDT test...");
                
                var client = new EtherNetIpClient();
                _logger.LogInformation("Created EtherNetIpClient");
                
                client.Connect("192.168.0.1:44818");
                _logger.LogInformation("Connected to PLC");
                
                // Test reading Part_Data UDT
                _logger.LogInformation("Attempting to read Part_Data UDT...");
                var udtValue = client.ReadUdtChunked("Part_Data");
                _logger.LogInformation("UDT read successful! Type: {Type}, IsUdt: {IsUdt}", udtValue.Type, udtValue.IsUdt);
                
                if (udtValue.IsUdt)
                {
                    var members = udtValue.UdtMembers;
                    if (members == null)
                    {
                        _logger.LogWarning("UDT reported IsUdt=true but returned null member map");
                        return;
                    }

                    _logger.LogInformation("UDT has {Count} members", members.Count);
                    foreach (var member in members)
                    {
                        _logger.LogInformation("Member: {Key} = {Value} (Type: {Type})", 
                            member.Key, member.Value.ToString(), member.Value.Type);
                    }
                }
                else
                {
                    _logger.LogInformation("UDT Value: {Value}", udtValue.ToString());
                }
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error in UDT test: {Message}", ex.Message);
            }
        }
    }
}
