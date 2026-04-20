using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Threading.Tasks;

namespace RustEtherNetIp
{
    /// <summary>
    /// Metadata information for a PLC tag.
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct TagMetadata
    {
        public int DataType;
        public int Scope;
        public int ArrayDimension;
        public int ArraySize;
    }

    internal class TagGroupRegistration
    {
        public string GroupName { get; set; } = string.Empty;
        public string[] TagNames { get; set; } = Array.Empty<string>();
        public int UpdateRateMs { get; set; }
        public TagGroup? Group { get; set; }
    }

    /// <summary>
    /// Lightweight descriptor for a registered tag group.
    /// </summary>
    public class TagGroupConfigInfo
    {
        public string GroupName { get; set; } = string.Empty;
        public string[] TagNames { get; set; } = Array.Empty<string>();
        public int UpdateRateMs { get; set; }
        public bool IsActive { get; set; }
    }

    /// <summary>
    /// Result of a one-shot group read.
    /// </summary>
    public class TagGroupSnapshot
    {
        public string GroupName { get; set; } = string.Empty;
        public DateTime SampledAt { get; set; }
        public Dictionary<string, object?> Values { get; set; } = new();
        public Dictionary<string, string> Errors { get; set; } = new();
        public bool HasErrors => Errors.Count > 0;
    }

    /// <summary>
    /// Extension methods for convenient EtherNet/IP operations.
    /// </summary>
    public static class EtherNetIpExtensions
    {
        /// <summary>
        /// Creates and connects to a PLC in one operation.
        /// </summary>
        public static EtherNetIpClient ConnectToPlc(string address)
        {
            var client = new EtherNetIpClient();
            if (!client.Connect(address))
            {
                client.Dispose();
                throw new Exception($"Failed to connect to PLC at {address}");
            }
            return client;
        }

        /// <summary>
        /// Attempts to connect to a PLC with retry logic.
        /// </summary>
        public static EtherNetIpClient? TryConnectToPlc(string address, int maxRetries = 3, int retryDelayMs = 1000)
        {
            for (int attempt = 0; attempt < maxRetries; attempt++)
            {
                try
                {
                    return ConnectToPlc(address);
                }
                catch
                {
                    if (attempt < maxRetries - 1)
                    {
                        Task.Delay(retryDelayMs).Wait();
                    }
                }
            }
            return null;
        }
    }
}
