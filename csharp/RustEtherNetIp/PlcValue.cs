using System;
using System.Collections.Generic;
using System.Text.Json;

namespace RustEtherNetIp
{
    /// <summary>
    /// Represents a PLC value that can be of various data types including nested UDTs.
    /// This class provides type-safe access to PLC data with full support for nested structures.
    /// </summary>
    public class PlcValue
    {
        private readonly object _value;
        private readonly PlcValueType _type;

        private PlcValue(object value, PlcValueType type)
        {
            _value = value;
            _type = type;
        }

        /// <summary>
        /// Creates a boolean value
        /// </summary>
        public static PlcValue Bool(bool value) => new PlcValue(value, PlcValueType.Bool);

        /// <summary>
        /// Creates a signed 8-bit integer value
        /// </summary>
        public static PlcValue Sint(sbyte value) => new PlcValue(value, PlcValueType.Sint);

        /// <summary>
        /// Creates a signed 16-bit integer value
        /// </summary>
        public static PlcValue Int(short value) => new PlcValue(value, PlcValueType.Int);

        /// <summary>
        /// Creates a signed 32-bit integer value
        /// </summary>
        public static PlcValue Dint(int value) => new PlcValue(value, PlcValueType.Dint);

        /// <summary>
        /// Creates a signed 64-bit integer value
        /// </summary>
        public static PlcValue Lint(long value) => new PlcValue(value, PlcValueType.Lint);

        /// <summary>
        /// Creates an unsigned 8-bit integer value
        /// </summary>
        public static PlcValue Usint(byte value) => new PlcValue(value, PlcValueType.Usint);

        /// <summary>
        /// Creates an unsigned 16-bit integer value
        /// </summary>
        public static PlcValue Uint(ushort value) => new PlcValue(value, PlcValueType.Uint);

        /// <summary>
        /// Creates an unsigned 32-bit integer value
        /// </summary>
        public static PlcValue Udint(uint value) => new PlcValue(value, PlcValueType.Udint);

        /// <summary>
        /// Creates an unsigned 64-bit integer value
        /// </summary>
        public static PlcValue Ulint(ulong value) => new PlcValue(value, PlcValueType.Ulint);

        /// <summary>
        /// Creates a 32-bit floating point value
        /// </summary>
        public static PlcValue Real(float value) => new PlcValue(value, PlcValueType.Real);

        /// <summary>
        /// Creates a 64-bit floating point value
        /// </summary>
        public static PlcValue Lreal(double value) => new PlcValue(value, PlcValueType.Lreal);

        /// <summary>
        /// Creates a string value
        /// </summary>
        public static PlcValue String(string value) => new PlcValue(value, PlcValueType.String);

        /// <summary>
        /// Creates a UDT (User Defined Type) value with nested support
        /// </summary>
        public static PlcValue Udt(Dictionary<string, PlcValue> value) => new PlcValue(value, PlcValueType.Udt);

        /// <summary>
        /// Creates a UDT value from UdtData (generic UDT with symbol_id and raw bytes)
        /// </summary>
        /// <param name="udtData">The UDT data containing symbol_id and raw bytes</param>
        /// <returns>A PlcValue representing the UDT</returns>
        public static PlcValue UdtFromData(UdtData udtData) => new PlcValue(udtData, PlcValueType.Udt);

        /// <summary>
        /// Gets the value as the specified type
        /// </summary>
        public T As<T>() => (T)_value;

        /// <summary>
        /// Gets the value as the specified type with a default if conversion fails
        /// </summary>
        public T? AsOrDefault<T>(T? defaultValue = default)
        {
            try
            {
                return (T)_value;
            }
            catch
            {
                return defaultValue;
            }
        }

        /// <summary>
        /// Gets the underlying value
        /// </summary>
        public object Value => _value;

        /// <summary>
        /// Gets the value type
        /// </summary>
        public PlcValueType Type => _type;

        /// <summary>
        /// Checks if this is a UDT value
        /// </summary>
        public bool IsUdt => _type == PlcValueType.Udt;

        /// <summary>
        /// Gets the UDT members if this is a UDT value (legacy format)
        /// </summary>
        public Dictionary<string, PlcValue>? UdtMembers
        {
            get
            {
                if (!IsUdt) return null;
                
                // Check if it's UdtData format
                if (_value is UdtData)
                {
                    // UdtData format doesn't have parsed members
                    // Users should use UdtData property instead
                    return null;
                }
                
                // Legacy format: Dictionary<string, PlcValue>
                return As<Dictionary<string, PlcValue>>();
            }
        }

        /// <summary>
        /// Gets the UDT data if this is a UDT value (new generic format)
        /// </summary>
        public UdtData? UdtData => IsUdt && _value is UdtData data ? data : null;

        /// <summary>
        /// Checks if this UDT value uses the new UdtData format
        /// </summary>
        public bool IsUdtDataFormat => IsUdt && _value is UdtData;

        /// <summary>
        /// Gets a nested UDT member by path (e.g., "Status.Running")
        /// </summary>
        public PlcValue? GetNestedValue(string path)
        {
            if (!IsUdt) return null;

            var parts = path.Split('.');
            var current = this;

            foreach (var part in parts)
            {
                if (!current.IsUdt) return null;
                
                var members = current.UdtMembers;
                if (members?.ContainsKey(part) != true) return null;
                
                current = members[part];
            }

            return current;
        }

        /// <summary>
        /// Serializes the value to JSON for transmission to the Rust library
        /// </summary>
        public string ToJson()
        {
            return JsonSerializer.Serialize(_value, new JsonSerializerOptions
            {
                PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
                WriteIndented = false
            });
        }

        /// <summary>
        /// Deserializes a JSON string to a PlcValue
        /// </summary>
        public static PlcValue FromJson(string json)
        {
            try
            {
                var jsonDoc = JsonDocument.Parse(json);
                return FromJsonElement(jsonDoc.RootElement);
            }
            catch
            {
                throw new ArgumentException("Invalid JSON format", nameof(json));
            }
        }

        private static PlcValue FromJsonElement(JsonElement element)
        {
            // Handle Rust enum format: {"Dint": 42}, {"Bool": true}, {"String": "value"}, etc.
            if (element.ValueKind == JsonValueKind.Object)
            {
                var obj = element.EnumerateObject();
                if (obj.Count() == 1)
                {
                    var firstProp = obj.First();
                    var key = firstProp.Name;
                    var value = firstProp.Value;

                    switch (key)
                    {
                        case "Bool":
                            return Bool(value.GetBoolean());
                        case "Sint":
                            return Sint((sbyte)value.GetInt32());
                        case "Int":
                            return Int((short)value.GetInt32());
                        case "Dint":
                            return Dint(value.GetInt32());
                        case "Lint":
                            return Lint(value.GetInt64());
                        case "Usint":
                            return Usint((byte)value.GetUInt32());
                        case "Uint":
                            return Uint((ushort)value.GetUInt32());
                        case "Udint":
                            return Udint(value.GetUInt32());
                        case "Ulint":
                            return Ulint(value.GetUInt64());
                        case "Real":
                            return Real(value.GetSingle());
                        case "Lreal":
                            return Lreal(value.GetDouble());
                        case "String":
                            return String(value.GetString() ?? string.Empty);
                        case "Udt":
                            // UDT format: {"Udt": {"symbol_id": 123, "data": [1,2,3,...]}}
                            if (value.ValueKind == JsonValueKind.Object)
                            {
                                var udtObj = value;
                                if (udtObj.TryGetProperty("symbol_id", out var symbolId) &&
                                    udtObj.TryGetProperty("data", out var data))
                                {
                                    var dataArray = data.EnumerateArray().Select(b => (byte)b.GetUInt32()).ToArray();
                                    var udtData = new UdtData { SymbolId = symbolId.GetInt32(), Data = dataArray };
                                    return UdtFromData(udtData);
                                }
                                // Fallback: try to parse as nested dictionary
                                var udtDict = new Dictionary<string, PlcValue>();
                                foreach (var udtProp in udtObj.EnumerateObject())
                                {
                                    udtDict[udtProp.Name] = FromJsonElement(udtProp.Value);
                                }
                                return Udt(udtDict);
                            }
                            break;
                    }
                }
                // If not a single-key enum, treat as UDT dictionary
                var dict = new Dictionary<string, PlcValue>();
                foreach (var dictProp in element.EnumerateObject())
                {
                    dict[dictProp.Name] = FromJsonElement(dictProp.Value);
                }
                return Udt(dict);
            }

            // Handle simple JSON values (fallback for non-enum format)
            switch (element.ValueKind)
            {
                case JsonValueKind.True:
                case JsonValueKind.False:
                    return Bool(element.GetBoolean());

                case JsonValueKind.Number:
                    if (element.TryGetInt32(out var intVal))
                        return Dint(intVal);
                    if (element.TryGetSingle(out var floatVal))
                        return Real(floatVal);
                    if (element.TryGetDouble(out var doubleVal))
                        return Lreal(doubleVal);
                    break;

                case JsonValueKind.String:
                    return String(element.GetString() ?? string.Empty);
            }

            throw new ArgumentException($"Unsupported JSON value type: {element.ValueKind}");
        }

        public override string ToString()
        {
            return _value?.ToString() ?? "null";
        }

        public override bool Equals(object? obj)
        {
            if (obj is PlcValue other)
            {
                return _type == other._type && Equals(_value, other._value);
            }
            return false;
        }

        public override int GetHashCode()
        {
            return HashCode.Combine(_value, _type);
        }
    }

    /// <summary>
    /// Represents the type of a PLC value
    /// </summary>
    public enum PlcValueType
    {
        Bool,
        Sint,
        Int,
        Dint,
        Lint,
        Usint,
        Uint,
        Udint,
        Ulint,
        Real,
        Lreal,
        String,
        Udt
    }
}
