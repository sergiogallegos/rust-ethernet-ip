using System;
using System.Collections.Generic;

namespace RustEtherNetIp
{
    /// <summary>
    /// Represents a UDT (User Defined Type) template for parsing and serializing UDT data.
    /// </summary>
    public class UdtTemplate
    {
        public string Name { get; set; } = string.Empty;
        public string Description { get; set; } = string.Empty;
        public int TotalSize { get; set; }
        public List<UdtMemberTemplate> Members { get; set; } = new List<UdtMemberTemplate>();
        
        /// <summary>
        /// Parses raw UDT data according to this template.
        /// </summary>
        /// <param name="rawData">Raw bytes from the PLC.</param>
        /// <returns>Dictionary of member names and their parsed values.</returns>
        public Dictionary<string, PlcValue> ParseRawData(byte[] rawData)
        {
            var result = new Dictionary<string, PlcValue>();
            if (rawData == null)
                throw new ArgumentNullException(nameof(rawData));
            
            foreach (var member in Members)
            {
                int memberOffset = member.Offset >= 0 ? member.Offset : 0;
                if (memberOffset + member.Size > rawData.Length)
                {
                    result[$"_error_{member.Name}"] = PlcValue.String("Insufficient data");
                    break;
                }
                
                try
                {
                    var value = ParseMemberValue(rawData, memberOffset, member);
                    result[member.Name] = value;
                }
                catch (Exception ex)
                {
                    result[$"_error_{member.Name}"] = PlcValue.String($"Parse error: {ex.Message}");
                }
            }
            
            return result;
        }
        
        private PlcValue ParseMemberValue(byte[] data, int offset, UdtMemberTemplate member)
        {
            return member.DataType.ToLower() switch
            {
                "bool" => PlcValue.Bool((data[offset] & (1 << member.BitOffset)) != 0),
                "sint" => PlcValue.Sint((sbyte)data[offset]),
                "int" => PlcValue.Int(BitConverter.ToInt16(data, offset)),
                "dint" => PlcValue.Dint(BitConverter.ToInt32(data, offset)),
                "real" => PlcValue.Real(BitConverter.ToSingle(data, offset)),
                "string" => PlcValue.String(System.Text.Encoding.ASCII.GetString(data, offset, member.Size).TrimEnd('\0')),
                _ => PlcValue.String($"Unknown type: {member.DataType}")
            };
        }
    }
    
    /// <summary>
    /// Represents a member of a UDT template.
    /// </summary>
    public class UdtMemberTemplate
    {
        public string Name { get; set; } = string.Empty;
        public string DataType { get; set; } = string.Empty;
        public int Size { get; set; }
        public int Offset { get; set; }
        public int BitOffset { get; set; } = 0; // For bit-level access
        public string Description { get; set; } = string.Empty;
    }
    
    /// <summary>
    /// Factory for creating common UDT templates.
    /// </summary>
    public static class UdtTemplateFactory
    {
        /// <summary>
        /// Creates a generic UDT template for raw data parsing.
        /// </summary>
        /// <param name="name">Name of the UDT.</param>
        /// <param name="size">Size of the UDT in bytes.</param>
        /// <returns>A generic UDT template.</returns>
        public static UdtTemplate CreateGenericTemplate(string name, int size)
        {
            return new UdtTemplate
            {
                Name = name,
                Description = $"Generic UDT template for {name}",
                TotalSize = size,
                Members = new List<UdtMemberTemplate>
                {
                    // Generic template - can be extended by applications
                    new UdtMemberTemplate { Name = "_raw_data", DataType = "bytes", Size = size, Offset = 0, BitOffset = 0, Description = "Raw UDT data" }
                }
            };
        }
        
        /// <summary>
        /// Creates a template for parsing raw UDT data without specific member definitions.
        /// </summary>
        /// <param name="name">Name of the UDT.</param>
        /// <param name="rawData">Raw data bytes from the PLC.</param>
        /// <returns>A template that can parse the raw data generically.</returns>
        public static UdtTemplate CreateFromRawData(string name, byte[] rawData)
        {
            return new UdtTemplate
            {
                Name = name,
                Description = $"UDT template created from raw data for {name}",
                TotalSize = rawData.Length,
                Members = new List<UdtMemberTemplate>
                {
                    new UdtMemberTemplate { Name = "_raw_data", DataType = "bytes", Size = rawData.Length, Offset = 0, BitOffset = 0, Description = "Raw UDT data" },
                    new UdtMemberTemplate { Name = "_size", DataType = "dint", Size = 4, Offset = 0, BitOffset = 0, Description = "UDT size in bytes" }
                }
            };
        }
    }
}
