using System;
using System.Collections.Generic;
using System.Text.Json;

namespace RustEtherNetIp
{
    /// <summary>
    /// Represents raw UDT (User Defined Type) data with symbol_id and raw bytes.
    /// This allows for generic UDT handling without needing to know the UDT's member structure ahead of time.
    /// </summary>
    /// <remarks>
    /// The UdtData structure stores the raw bytes of a UDT instance along with its
    /// symbol_id (template instance ID). This allows for generic UDT handling without
    /// needing to know the UDT's member structure ahead of time. The raw data can be
    /// parsed later using a UDT definition.
    /// 
    /// The symbol_id is crucial for writing UDTs back to the PLC.
    /// </remarks>
    public class UdtData
    {
        /// <summary>
        /// The symbol_id (template instance ID) of the UDT.
        /// This is crucial for writing UDTs back to the PLC.
        /// </summary>
        public int SymbolId { get; set; }

        /// <summary>
        /// The raw byte data of the UDT instance.
        /// </summary>
        public byte[] Data { get; set; }

        /// <summary>
        /// Creates a new UdtData instance
        /// </summary>
        public UdtData()
        {
            SymbolId = 0;
            Data = Array.Empty<byte>();
        }

        /// <summary>
        /// Creates a new UdtData instance with the specified symbol_id and data
        /// </summary>
        /// <param name="symbolId">The symbol_id (template instance ID)</param>
        /// <param name="data">The raw byte data</param>
        public UdtData(int symbolId, byte[] data)
        {
            SymbolId = symbolId;
            Data = data ?? Array.Empty<byte>();
        }

        /// <summary>
        /// Converts the UdtData to JSON format for transmission to the Rust library
        /// </summary>
        public string ToJson()
        {
            var obj = new
            {
                symbol_id = SymbolId,
                data = Convert.ToBase64String(Data)
            };
            return JsonSerializer.Serialize(obj);
        }

        /// <summary>
        /// Creates a UdtData instance from JSON
        /// </summary>
        public static UdtData FromJson(string json)
        {
            var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;

            int symbolId = root.GetProperty("symbol_id").GetInt32();
            string dataBase64 = root.GetProperty("data").GetString();
            byte[] data = Convert.FromBase64String(dataBase64);

            return new UdtData(symbolId, data);
        }

        /// <summary>
        /// Converts UdtData to a Dictionary format (requires UDT definition)
        /// </summary>
        /// <param name="udtDefinition">The UDT definition to use for parsing</param>
        /// <returns>A dictionary of member names to PlcValues</returns>
        /// <remarks>
        /// This method requires the UDT definition to parse the raw bytes.
        /// Use GetUdtDefinition() from EtherNetIpClient to obtain the definition.
        /// </remarks>
        public Dictionary<string, PlcValue> ToDictionary(UdtTemplate udtDefinition)
        {
            // This would require implementing the parsing logic based on the UDT definition
            // For now, return an empty dictionary
            // TODO: Implement full parsing based on UdtTemplate structure
            return new Dictionary<string, PlcValue>();
        }

        public override string ToString()
        {
            return $"UdtData(SymbolId={SymbolId}, DataLength={Data?.Length ?? 0} bytes)";
        }
    }
}

