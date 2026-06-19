// EtherNetIpClient.cs - Enhanced C# wrapper for Rust EtherNet/IP driver
using System;
using System.Runtime.InteropServices;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using System.Threading;
using System.Text.Json;

namespace RustEtherNetIp
{
    /// <summary>
    /// Enhanced C# wrapper for Rust EtherNet/IP driver to communicate with Allen-Bradley CompactLogix and ControlLogix PLCs.
    /// Provides high-performance, type-safe access to PLC tags via EtherNet/IP protocol with comprehensive data type support.
    /// </summary>
    /// <remarks>
    /// This class manages the connection to a single PLC and provides methods to read/write
    /// all Allen-Bradley native data types. The underlying Rust library handles the EtherNet/IP protocol
    /// implementation, CIP messaging, advanced tag addressing, and network communications.
    /// 
    /// Performance: 1,500+ reads/sec, 800+ writes/sec
    /// Supported PLCs: CompactLogix L1x-L5x, ControlLogix L6x-L8x series
    /// Supported Data Types: BOOL, SINT, INT, DINT, LINT, USINT, UINT, UDINT, ULINT, REAL, LREAL, STRING, UDT
    /// Advanced Features: Program-scoped tags, array addressing, bit operations, UDT member access
    /// 
    /// <para><strong>⚠️ Known Limitations (PLC Firmware Restrictions):</strong></para>
    /// <para>
    /// The following operations are not supported due to PLC firmware restrictions.
    /// These limitations are inherent to the Allen-Bradley PLC firmware and cannot be bypassed at the library level.
    /// </para>
    /// <list type="bullet">
    /// <item><description><strong>STRING Tags:</strong> Cannot write directly to STRING tags (e.g., "gTest_STRING", "Program:TestProgram.gTest_STRING"). 
    /// This is a PLC firmware limitation (CIP Error 0x2107). STRING tags can be read successfully but cannot be written directly.
    /// For STRING members in UDTs, use the workaround: read the entire UDT, modify the STRING member in memory, then write the entire UDT back.</description></item>
    /// <item><description><strong>STRING Members in UDTs:</strong> Cannot write directly to STRING members within UDTs 
    /// (e.g., "gTestUDT.Member5_String"). Must read the entire UDT structure, modify the STRING member in memory, then write the entire UDT back.</description></item>
    /// <item><description><strong>UDT Array Element Members:</strong> Cannot write directly to members of UDT array elements 
    /// (e.g., "gTestUDT_Array[0].Member1_DINT", "Program:TestProgram.gTestUDT_Array[0].Member1_DINT"). 
    /// Must read the entire UDT array element, modify the member in memory, then write the entire element back.</description></item>
    /// </list>
    /// 
    /// <para><strong>✅ What Works:</strong></para>
    /// <list type="bullet">
    /// <item><description>Reading all tag types including STRING tags and UDT members</description></item>
    /// <item><description>Writing DINT, REAL, BOOL, INT, and other numeric types</description></item>
    /// <item><description>Writing UDT members (non-STRING) for non-array UDTs (e.g., "gTestUDT.Member1_DINT")</description></item>
    /// <item><description>Writing entire UDT array elements (e.g., "gTestUDT_Array[0]")</description></item>
    /// <item><description>Writing simple array elements (e.g., "gArray[5]")</description></item>
    /// <item><description>Reading UDT array element members (e.g., "gTestUDT_Array[0].Member1_DINT")</description></item>
    /// </list>
    /// 
    /// <para><strong>💡 Workarounds:</strong></para>
    /// <list type="bullet">
    /// <item><description><strong>UDT Array Element Members:</strong> Read the entire UDT array element, modify the member in memory, then write the entire UDT array element back.</description></item>
    /// <item><description><strong>STRING Members in UDTs:</strong> Read the entire UDT, modify the STRING member in memory, then write the entire UDT back.</description></item>
    /// <item><description><strong>Standalone STRING Tags:</strong> There is no workaround at the communication library level. Alternative approaches may include using PLC ladder logic or other PLC-side mechanisms.</description></item>
    /// </list>
    /// </remarks>
    /// <example>
    /// Basic usage:
    /// <code>
    /// using var client = new EtherNetIpClient();
    /// if (client.Connect("192.168.1.100:44818"))
    /// {
    ///     // Read different data types
    ///     bool startButton = client.ReadBool("StartButton");
    ///     int counter = client.ReadDint("ProductionCount");
    ///     float temperature = client.ReadReal("BoilerTemp");
    ///     
    ///     // Advanced tag addressing
    ///     bool motorStatus = client.ReadBool("Program:MainProgram.Motor.Status");
    ///     int arrayElement = client.ReadDint("DataArray[5]");
    ///     bool bitAccess = client.ReadBool("StatusWord.15");
    ///     
    ///     // Write operations
    ///     client.WriteBool("StartButton", true);
    ///     client.WriteDint("SetPoint", 1500);
    ///     client.WriteReal("TargetTemp", 72.5f);
    /// }
    /// </code>
    /// </example>
    public partial class EtherNetIpClient : IEtherNetIpClient
    {
        private int _clientId = -1;
        private string _currentAddress = string.Empty;
        private RoutePath? _currentRoutePath;
        private readonly object _lock = new();
        private bool _isDisposed;
        private readonly Dictionary<string, TagMetadata> _tagCache = new();
        private readonly SemaphoreSlim _operationLock = new(1, 1);
        private CancellationTokenSource _keepAliveCts = new();
        private Task? _keepAliveTask;
        private readonly Dictionary<string, TagSubscription> _subscriptions = new();
        private readonly Dictionary<string, CancellationTokenSource> _subscriptionTokens = new();
        private readonly object _subscriptionLock = new();
        private readonly Dictionary<string, TagGroupRegistration> _tagGroups = new();
        private readonly object _tagGroupLock = new();
        private readonly ClientStatistics _statistics = new();

        #region Boolean Operations

        /// <summary>
        /// Reads a BOOL (boolean) tag from the PLC.
        /// Supports advanced tag addressing including program-scoped tags, array elements, bit access,
        /// and UDT array element members (e.g., "gTestUDT_Array[0].Member3_BOOL").
        /// </summary>
        /// <param name="tagName">
        /// Name of the PLC tag to read. Examples:
        /// - Simple tag: "MotorRunning"
        /// - Program-scoped: "Program:MainProgram.StartButton"
        /// - Array element: "StatusArray[5]"
        /// - Bit access: "StatusWord.15"
        /// - UDT array element member: "gTestUDT_Array[0].Member3_BOOL"
        /// </param>
        /// <returns>The boolean value of the tag (true/false).</returns>
        /// <exception cref="InvalidOperationException">Thrown if not connected to PLC.</exception>
        /// <exception cref="Exception">Thrown if tag doesn't exist or communication fails.</exception>
        public bool ReadBool(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                var sw = System.Diagnostics.Stopwatch.StartNew();
                try
                {
                    CheckConnection();
                    IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                    try
                    {
                        // First try the type-specific FFI function
                        int result = eip_read_bool(_clientId, tagPtr, out int value);
                        if (result == 0)
                        {
                            _statistics.IncrementRead();
                            return value != 0;
                        }

                    // If that failed, try the generic read_tag function (handles complex paths better)
                    IntPtr resultPtr = Marshal.AllocHGlobal(4096);
                    try
                    {
                        result = eip_read_tag(_clientId, tagPtr, resultPtr, 4096);
                        if (result == 0)
                        {
                            string jsonResult = Marshal.PtrToStringAnsi(resultPtr) ?? string.Empty;
                            if (!string.IsNullOrEmpty(jsonResult))
                            {
                                var plcValue = PlcValue.FromJson(jsonResult);
                                if (plcValue.Type == PlcValueType.Bool)
                                    return plcValue.As<bool>();
                            }
                        }
                    }
                    finally
                    {
                        Marshal.FreeHGlobal(resultPtr);
                    }

                        throw OperationFailure($"Failed to read BOOL tag '{tagName}'. Check tag exists and is BOOL type.");
                    }
                    finally
                    {
                        Marshal.FreeHGlobal(tagPtr);
                    }
                }
                catch
                {
                    _statistics.IncrementError();
                    throw;
                }
                finally
                {
                    sw.Stop();
                    _statistics.AddResponseTime(sw.ElapsedMilliseconds);
                }
            });
        }

        /// <summary>
        /// Writes a BOOL (boolean) tag to the PLC.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to.</param>
        /// <param name="value">Boolean value to write (true/false).</param>
        /// <exception cref="InvalidOperationException">Thrown if not connected to PLC.</exception>
        /// <exception cref="Exception">Thrown if tag doesn't exist, is read-only, or communication fails.</exception>
        public void WriteBool(string tagName, bool value)
        {
            ExecuteWithLock(() =>
            {
                var sw = System.Diagnostics.Stopwatch.StartNew();
                try
                {
                    CheckConnection();
                    IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                    try
                    {
                        int result = eip_write_bool(_clientId, tagPtr, value ? 1 : 0);
                        if (result != 0)
                            ThrowDetailedWriteException(tagName, PlcValue.Bool(value), $"Failed to write BOOL tag '{tagName}'. Check tag exists and is writable.");
                        _statistics.IncrementWrite();
                    }
                    finally
                    {
                        Marshal.FreeHGlobal(tagPtr);
                    }
                }
                catch
                {
                    _statistics.IncrementError();
                    throw;
                }
                finally
                {
                    sw.Stop();
                    _statistics.AddResponseTime(sw.ElapsedMilliseconds);
                }
            });
        }

        #endregion

        #region Signed Integer Operations

        /// <summary>
        /// Reads a SINT (8-bit signed integer) tag from the PLC.
        /// Range: -128 to 127
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to read.</param>
        /// <returns>The SINT value of the tag.</returns>
        public sbyte ReadSint(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_read_sint(_clientId, tagPtr, out sbyte value);
                    if (result != 0)
                        throw OperationFailure($"Failed to read SINT tag '{tagName}'. Check tag exists and is SINT type.");
                    return value;
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Writes a SINT (8-bit signed integer) tag to the PLC.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to.</param>
        /// <param name="value">SINT value to write (-128 to 127).</param>
        public void WriteSint(string tagName, sbyte value)
        {
            ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_write_sint(_clientId, tagPtr, value);
                    if (result != 0)
                        throw OperationFailure($"Failed to write SINT tag '{tagName}'. Check tag exists and is writable.");
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Reads an INT (16-bit signed integer) tag from the PLC.
        /// Range: -32,768 to 32,767
        /// Supports complex paths like UDT array element members (e.g., "gTestUDT_Array[0].Member4_INT").
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to read.</param>
        /// <returns>The INT value of the tag.</returns>
        public short ReadInt(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    // First try the type-specific FFI function
                    int result = eip_read_int(_clientId, tagPtr, out short value);
                    if (result == 0)
                        return value;

                    // If that failed, try the generic read_tag function (handles complex paths better)
                    IntPtr resultPtr = Marshal.AllocHGlobal(4096);
                    try
                    {
                        result = eip_read_tag(_clientId, tagPtr, resultPtr, 4096);
                        if (result == 0)
                        {
                            string jsonResult = Marshal.PtrToStringAnsi(resultPtr) ?? string.Empty;
                            if (!string.IsNullOrEmpty(jsonResult))
                            {
                                var plcValue = PlcValue.FromJson(jsonResult);
                                if (plcValue.Type == PlcValueType.Int)
                                    return plcValue.As<short>();
                            }
                        }
                    }
                    finally
                    {
                        Marshal.FreeHGlobal(resultPtr);
                    }

                    throw OperationFailure($"Failed to read INT tag '{tagName}'. Check tag exists and is INT type.");
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Writes an INT (16-bit signed integer) tag to the PLC.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to.</param>
        /// <param name="value">INT value to write (-32,768 to 32,767).</param>
        public void WriteInt(string tagName, short value)
        {
            ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_write_int(_clientId, tagPtr, value);
                    if (result != 0)
                        ThrowDetailedWriteException(tagName, PlcValue.Int(value), $"Failed to write INT tag '{tagName}'. Check tag exists and is writable.");
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Reads a DINT (32-bit signed integer) tag from the PLC.
        /// Range: -2,147,483,648 to 2,147,483,647
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to read.</param>
        /// <returns>The DINT value of the tag.</returns>
        public int ReadDint(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_read_dint(_clientId, tagPtr, out int value);
                    if (result != 0)
                        throw OperationFailure($"Failed to read DINT tag '{tagName}'. Check tag exists and is DINT type.");
                    return value;
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Writes a DINT (32-bit signed integer) tag to the PLC.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to.</param>
        /// <param name="value">DINT value to write.</param>
        public void WriteDint(string tagName, int value)
        {
            ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_write_dint(_clientId, tagPtr, value);
                    if (result != 0)
                        ThrowDetailedWriteException(tagName, PlcValue.Dint(value), $"Failed to write DINT tag '{tagName}'. Check tag exists and is writable.");
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Reads a LINT (64-bit signed integer) tag from the PLC.
        /// Range: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to read.</param>
        /// <returns>The LINT value of the tag.</returns>
        public long ReadLint(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_read_lint(_clientId, tagPtr, out long value);
                    if (result != 0)
                        throw OperationFailure($"Failed to read LINT tag '{tagName}'. Check tag exists and is LINT type.");
                    return value;
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Writes a LINT (64-bit signed integer) tag to the PLC.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to.</param>
        /// <param name="value">LINT value to write.</param>
        public void WriteLint(string tagName, long value)
        {
            ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_write_lint(_clientId, tagPtr, value);
                    if (result != 0)
                        throw OperationFailure($"Failed to write LINT tag '{tagName}'. Check tag exists and is writable.");
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        #endregion

        #region Unsigned Integer Operations

        /// <summary>
        /// Reads a USINT (8-bit unsigned integer) tag from the PLC.
        /// Range: 0 to 255
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to read.</param>
        /// <returns>The USINT value of the tag.</returns>
        public byte ReadUsint(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_read_usint(_clientId, tagPtr, out byte value);
                    if (result != 0)
                        throw OperationFailure($"Failed to read USINT tag '{tagName}'. Check tag exists and is USINT type.");
                    return value;
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Writes a USINT (8-bit unsigned integer) tag to the PLC.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to.</param>
        /// <param name="value">USINT value to write (0 to 255).</param>
        public void WriteUsint(string tagName, byte value)
        {
            ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_write_usint(_clientId, tagPtr, value);
                    if (result != 0)
                        throw OperationFailure($"Failed to write USINT tag '{tagName}'. Check tag exists and is writable.");
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Reads a UINT (16-bit unsigned integer) tag from the PLC.
        /// Range: 0 to 65,535
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to read.</param>
        /// <returns>The UINT value of the tag.</returns>
        public ushort ReadUint(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_read_uint(_clientId, tagPtr, out ushort value);
                    if (result != 0)
                        throw OperationFailure($"Failed to read UINT tag '{tagName}'. Check tag exists and is UINT type.");
                    return value;
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Writes a UINT (16-bit unsigned integer) tag to the PLC.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to.</param>
        /// <param name="value">UINT value to write (0 to 65,535).</param>
        public void WriteUint(string tagName, ushort value)
        {
            ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_write_uint(_clientId, tagPtr, value);
                    if (result != 0)
                        throw OperationFailure($"Failed to write UINT tag '{tagName}'. Check tag exists and is writable.");
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Reads a UDINT (32-bit unsigned integer) tag from the PLC.
        /// Range: 0 to 4,294,967,295
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to read.</param>
        /// <returns>The UDINT value of the tag.</returns>
        public uint ReadUdint(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_read_udint(_clientId, tagPtr, out uint value);
                    if (result != 0)
                        throw OperationFailure($"Failed to read UDINT tag '{tagName}'. Check tag exists and is UDINT type.");
                    return value;
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Writes a UDINT (32-bit unsigned integer) tag to the PLC.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to.</param>
        /// <param name="value">UDINT value to write.</param>
        public void WriteUdint(string tagName, uint value)
        {
            ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_write_udint(_clientId, tagPtr, value);
                    if (result != 0)
                        throw OperationFailure($"Failed to write UDINT tag '{tagName}'. Check tag exists and is writable.");
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Reads a ULINT (64-bit unsigned integer) tag from the PLC.
        /// Range: 0 to 18,446,744,073,709,551,615
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to read.</param>
        /// <returns>The ULINT value of the tag.</returns>
        public ulong ReadUlint(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_read_ulint(_clientId, tagPtr, out ulong value);
                    if (result != 0)
                        throw OperationFailure($"Failed to read ULINT tag '{tagName}'. Check tag exists and is ULINT type.");
                    return value;
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Writes a ULINT (64-bit unsigned integer) tag to the PLC.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to.</param>
        /// <param name="value">ULINT value to write.</param>
        public void WriteUlint(string tagName, ulong value)
        {
            ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_write_ulint(_clientId, tagPtr, value);
                    if (result != 0)
                        throw OperationFailure($"Failed to write ULINT tag '{tagName}'. Check tag exists and is writable.");
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        #endregion

        #region Floating Point Operations

        /// <summary>
        /// Reads a REAL (32-bit IEEE 754 float) tag from the PLC.
        /// Range: ±1.18 × 10^-38 to ±3.40 × 10^38
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to read.</param>
        /// <returns>The REAL value of the tag.</returns>
        public float ReadReal(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_read_real(_clientId, tagPtr, out double value);
                    if (result != 0)
                        throw OperationFailure($"Failed to read REAL tag '{tagName}'. Check tag exists and is REAL type.");
                    return (float)value;
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Writes a REAL (32-bit IEEE 754 float) tag to the PLC.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to.</param>
        /// <param name="value">REAL value to write.</param>
        public void WriteReal(string tagName, float value)
        {
            ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_write_real(_clientId, tagPtr, value);
                    if (result != 0)
                        ThrowDetailedWriteException(tagName, PlcValue.Real(value), $"Failed to write REAL tag '{tagName}'. Check tag exists and is writable.");
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Reads an LREAL (64-bit IEEE 754 double) tag from the PLC.
        /// Range: ±2.23 × 10^-308 to ±1.80 × 10^308
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to read.</param>
        /// <returns>The LREAL value of the tag.</returns>
        public double ReadLreal(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_read_lreal(_clientId, tagPtr, out double value);
                    if (result != 0)
                        throw OperationFailure($"Failed to read LREAL tag '{tagName}'. Check tag exists and is LREAL type.");
                    return value;
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Writes an LREAL (64-bit IEEE 754 double) tag to the PLC.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to.</param>
        /// <param name="value">LREAL value to write.</param>
        public void WriteLreal(string tagName, double value)
        {
            ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_write_lreal(_clientId, tagPtr, value);
                    if (result != 0)
                        throw OperationFailure($"Failed to write LREAL tag '{tagName}'. Check tag exists and is writable.");
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        #endregion

        #region String Operations

        /// <summary>
        /// Reads a STRING tag from the PLC.
        /// Supports complex paths like UDT members (e.g., "gTestUDT.Member5_String").
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to read.</param>
        /// <returns>The string value of the tag.</returns>
        public string ReadString(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    // First try the type-specific FFI function with larger buffer
                    IntPtr resultPtr = Marshal.AllocHGlobal(512); // Increased buffer for longer strings
                    try
                    {
                        int result = eip_read_string(_clientId, tagPtr, resultPtr, 512);
                        if (result == 0)
                        {
                            string value = Marshal.PtrToStringAnsi(resultPtr) ?? string.Empty;
                            return value; // Return even if empty (LEN=0 is valid for zeroed/cleared STRING tags)
                        }
                    }
                    finally
                    {
                        Marshal.FreeHGlobal(resultPtr);
                    }

                    // If that failed, try the generic read_tag function (handles complex paths better)
                    IntPtr resultPtr2 = Marshal.AllocHGlobal(4096);
                    try
                    {
                        int result = eip_read_tag(_clientId, tagPtr, resultPtr2, 4096);
                        if (result == 0)
                        {
                            string jsonResult = Marshal.PtrToStringAnsi(resultPtr2) ?? string.Empty;
                            if (!string.IsNullOrEmpty(jsonResult))
                            {
                                var plcValue = PlcValue.FromJson(jsonResult);
                                if (plcValue.Type == PlcValueType.String)
                                    return plcValue.As<string>();

                                // Tag exists and was read successfully, but not parsed as String type.
                                // This happens with zeroed/empty STRING members (LEN=0) in UDTs —
                                // the raw bytes get parsed as a different type. Return empty string.
                                return string.Empty;
                            }
                        }
                    }
                    finally
                    {
                        Marshal.FreeHGlobal(resultPtr2);
                    }

                    throw OperationFailure($"Failed to read STRING tag '{tagName}'. Check tag exists and is STRING type.");
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Writes a STRING tag to the PLC.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to.</param>
        /// <param name="value">String value to write.</param>
        /// <exception cref="Exception">Thrown if the write operation fails. Note: STRING tag writes may fail with CIP Error 0x2107 
        /// due to PLC firmware limitations. STRING tags can be read but not written directly.</exception>
        /// <remarks>
        /// <para><strong>⚠️ PLC Limitation:</strong> Most PLCs do not support direct writes to STRING tags (CIP Error 0x2107). 
        /// This is a firmware restriction, not a library bug. STRING tags can be read successfully, but writes will fail.</para>
        /// <para>If you need to modify STRING values, consider:</para>
        /// <list type="bullet">
        /// <item><description>Using ladder logic or other PLC-side mechanisms to update STRING values</description></item>
        /// <item><description>If the STRING is part of a UDT, write the entire UDT structure (though STRING members in UDTs also have limitations)</description></item>
        /// </list>
        /// </remarks>
        public void WriteString(string tagName, string value)
        {
            ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                IntPtr valuePtr = Marshal.StringToHGlobalAnsi(value);
                try
                {
                    int result = eip_write_string(_clientId, tagPtr, valuePtr);
                    if (result != 0)
                        ThrowDetailedWriteException(tagName, PlcValue.String(value), $"Failed to write STRING tag '{tagName}'. Check tag exists and is writable.");
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                    Marshal.FreeHGlobal(valuePtr);
                }
            });
        }

        /// <summary>
        /// Writes a STRING tag to the PLC using the LogixString structure format.
        /// This method attempts to write the STRING as a UDT structure (len + data fields),
        /// following the ASComm.NET pattern for Logix string handling.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to.</param>
        /// <param name="logixString">LogixString structure containing the string data.</param>
        /// <exception cref="Exception">Thrown if the write operation fails. Note: STRING tag writes may fail 
        /// with CIP Error 0x2107 due to PLC firmware limitations.</exception>
        /// <remarks>
        /// <para><strong>⚠️ PLC Limitation:</strong> Most PLCs do not support direct writes to STRING tags 
        /// (CIP Error 0x2107). This is a firmware restriction, not a library bug. STRING tags can be read 
        /// successfully, but writes will typically fail even when using the structured format.</para>
        /// <para>This method converts the LogixString structure to a UDT format and attempts to write it.
        /// The LogixString structure matches the Allen-Bradley STRING format: len (DINT) + data (SINT array).</para>
        /// </remarks>
        public void WriteStringAsUdt(string tagName, LogixString logixString)
        {
            _ = logixString ?? throw new ArgumentNullException(nameof(logixString));

            // Note: This method attempts to write a STRING tag as a UDT structure.
            // However, due to PLC firmware limitations (CIP Error 0x2107), STRING tag writes
            // typically fail even when using the structured format.
            // 
            // The LogixString structure represents: len (DINT) + data (SINT array)
            // For now, we'll attempt to write the LEN field separately as a workaround.
            // Writing the full structure requires raw byte marshaling which is complex.
            
            try
            {
                // Attempt to write LEN field directly
                WriteDint($"{tagName}.LEN", logixString.len);
                
                // Note: Writing the DATA array field directly also fails due to PLC limitations.
                // The full STRING structure write is not supported by the PLC firmware.
            }
            catch
            {
                // If writing LEN fails, try the full UDT approach (which will also likely fail)
                // Convert LogixString to raw bytes and write as UDT
                // Logix STRING format: len (4 bytes, DINT) + data (variable length, SINT array)
                var totalSize = 4 + logixString.data.Length; // 4 bytes for len + data array
                var rawBytes = new byte[totalSize];
                
                // Write len as DINT (4 bytes, little-endian)
                BitConverter.GetBytes(logixString.len).CopyTo(rawBytes, 0);
                
                // Copy data array
                Array.Copy(logixString.data, 0, rawBytes, 4, logixString.data.Length);
                
                // Create UdtData with raw bytes (symbol_id 0 means unknown, will be determined by PLC)
                var udtData = new UdtData { SymbolId = 0, Data = rawBytes };
                WriteUdtData(tagName, udtData);
            }
        }

        #endregion

        #region UDT Operations

        /// <summary>
        /// Reads a UDT (User Defined Type) tag from the PLC with full nested support.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to read.</param>
        /// <returns>PlcValue containing the UDT with nested structure support.</returns>
        public PlcValue ReadUdt(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                IntPtr resultPtr = Marshal.AllocHGlobal(16384); // Increased buffer for complex UDTs
                try
                {
                    // First try normal UDT reading
                    int result = eip_read_udt(_clientId, tagPtr, resultPtr, 16384);

                    if (result == 0)
                    {
                        // Success - convert the JSON result to PlcValue
                        string jsonResult = Marshal.PtrToStringAnsi(resultPtr) ?? string.Empty;
                        if (!string.IsNullOrEmpty(jsonResult))
                        {
                            // Try to parse as UdtData first (new format)
                            try
                            {
                                var udtData = UdtData.FromJson(jsonResult);
                                return PlcValue.UdtFromData(udtData);
                            }
                            catch
                            {
                                // Fallback to legacy Dictionary format
                                return PlcValue.FromJson(jsonResult);
                            }
                        }
                    }

                    // If normal reading failed, try chunked reading
                    // This handles "Partial transfer" errors for large UDTs
                    return ReadUdtWithChunkedFallback(tagName);
                }
                catch
                {
                    // Handle any UDT reading errors with chunked fallback
                    // This includes "Partial transfer" errors and other UDT issues
                    return ReadUdtWithChunkedFallback(tagName);
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                    Marshal.FreeHGlobal(resultPtr);
                }
            });
        }

        private PlcValue ReadUdtWithChunkedFallback(string tagName)
        {
            // Chunked reading for large UDTs that exceed normal packet size
            // NOTE: This method is called from within ExecuteWithLock, so we don't need another lock
            CheckConnection();
            IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
            IntPtr resultPtr = Marshal.AllocHGlobal(16384);
            try
            {
                int result = eip_read_udt_chunked(_clientId, tagPtr, resultPtr, 16384);

                if (result == 0)
                {
                    string jsonResult = Marshal.PtrToStringAnsi(resultPtr) ?? string.Empty;
                    if (!string.IsNullOrEmpty(jsonResult))
                    {
                        // Try to parse as UdtData first (new format)
                        try
                        {
                            var udtData = UdtData.FromJson(jsonResult);
                            return PlcValue.UdtFromData(udtData);
                        }
                        catch
                        {
                            // Fallback to legacy Dictionary format
                            return PlcValue.FromJson(jsonResult);
                        }
                    }
                    else
                    {
                        throw new Exception($"Empty response when reading UDT tag '{tagName}' with chunked reading.");
                    }
                }
                else
                {
                    throw OperationFailure($"Failed to read UDT tag '{tagName}' with chunked reading. Check tag exists and is UDT type.");
                }
            }
            finally
            {
                Marshal.FreeHGlobal(tagPtr);
                Marshal.FreeHGlobal(resultPtr);
            }
        }

        private PlcValue? ReadTagValue(string tagName)
        {
            // Helper method to read a tag value safely
            try
            {
                // Try different data types to find what works
                try { return PlcValue.Bool(ReadBool(tagName)); } catch { }
                try { return PlcValue.Dint(ReadDint(tagName)); } catch { }
                try { return PlcValue.Real(ReadReal(tagName)); } catch { }
                try { return PlcValue.String(ReadString(tagName)); } catch { }
                try { return PlcValue.Int(ReadInt(tagName)); } catch { }
                try { return PlcValue.Sint(ReadSint(tagName)); } catch { }
                
                return null;
            }
            catch
            {
                return null;
            }
        }

        /// <summary>
        /// Writes a UDT (User Defined Type) tag to the PLC with full nested support.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to.</param>
        /// <param name="value">PlcValue containing the UDT with nested structure support.</param>
        public void WriteUdt(string tagName, PlcValue value)
        {
            _ = value ?? throw new ArgumentNullException(nameof(value));
            
            if (!value.IsUdt)
                throw new ArgumentException("Value must be a UDT type", nameof(value));

            ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    // For large UDTs, use chunked writing approach
                    
                    string jsonValue;
                    
                    // Check if it's UdtData format (new generic format)
                    if (value.IsUdtDataFormat && value.UdtData != null)
                    {
                        // Use UdtData JSON format
                        jsonValue = value.UdtData.ToJson();
                    }
                    else
                    {
                        // Legacy Dictionary format
                        jsonValue = value.ToJson();
                    }
                    
                    IntPtr valuePtr = Marshal.StringToHGlobalAnsi(jsonValue);
                    try
                    {
                        int result = eip_write_udt(_clientId, tagPtr, valuePtr, jsonValue.Length);
                        if (result != 0)
                            throw OperationFailure($"Failed to write UDT tag '{tagName}'. Check tag exists and is writable.");
                    }
                    finally
                    {
                        Marshal.FreeHGlobal(valuePtr);
                    }
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Writes a UDT using UdtData format (generic UDT with symbol_id and raw bytes)
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to</param>
        /// <param name="udtData">The UDT data containing symbol_id and raw bytes</param>
        public void WriteUdtData(string tagName, UdtData udtData)
        {
            if (udtData == null)
                throw new ArgumentNullException(nameof(udtData));

            WriteUdt(tagName, PlcValue.UdtFromData(udtData));
        }

        /// <summary>
        /// Writes a UDT (User Defined Type) tag to the PLC using a dictionary.
        /// This is a convenience method for backward compatibility.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to write to.</param>
        /// <param name="value">Dictionary containing UDT member values.</param>
        public void WriteUdt(string tagName, Dictionary<string, object> value)
        {
            _ = value ?? throw new ArgumentNullException(nameof(value));

            // Convert Dictionary<string, object> to Dictionary<string, PlcValue>
            var udtValue = new Dictionary<string, PlcValue>();
            foreach (var kvp in value)
            {
                udtValue[kvp.Key] = ConvertObjectToPlcValue(kvp.Value);
            }

            WriteUdt(tagName, PlcValue.Udt(udtValue));
        }

        /// <summary>
        /// Reads a UDT (User Defined Type) tag from the PLC and returns it as a dictionary.
        /// This is a convenience method for backward compatibility.
        /// </summary>
        /// <param name="tagName">Name of the PLC tag to read.</param>
        /// <returns>Dictionary containing UDT member values.</returns>
        public Dictionary<string, object> ReadUdtAsDictionary(string tagName)
        {
            var udtValue = ReadUdt(tagName);
            if (!udtValue.IsUdt)
                throw new Exception($"Tag '{tagName}' is not a UDT type.");

            var members = udtValue.UdtMembers;
            if (members == null)
                throw new Exception($"Tag '{tagName}' returned raw UDT data without member definitions.");
            
            return ConvertUdtToDictionary(members);
        }

        /// <summary>
        /// Gets a nested value from a UDT using dot notation (e.g., "Status.Running").
        /// </summary>
        /// <param name="tagName">Name of the UDT tag.</param>
        /// <param name="memberPath">Dot-separated path to the nested member (e.g., "Status.Running").</param>
        /// <returns>PlcValue of the nested member, or null if not found.</returns>
        public PlcValue? GetUdtMember(string tagName, string memberPath)
        {
            var udtValue = ReadUdt(tagName);
            return udtValue.GetNestedValue(memberPath);
        }

        /// <summary>
        /// Sets a nested value in a UDT using dot notation (e.g., "Status.Running").
        /// </summary>
        /// <param name="tagName">Name of the UDT tag.</param>
        /// <param name="memberPath">Dot-separated path to the nested member (e.g., "Status.Running").</param>
        /// <param name="value">Value to set.</param>
        /// <exception cref="Exception">Thrown if the operation fails. Note: Writing to UDT array element members 
        /// (e.g., "gTestUDT_Array[0].Member1_DINT") or STRING members in UDTs will fail with CIP Error 0x2107 due to PLC firmware limitations.</exception>
        /// <remarks>
        /// <para><strong>⚠️ PLC Limitations:</strong></para>
        /// <list type="bullet">
        /// <item><description><strong>UDT Array Element Members:</strong> Cannot write directly to members of UDT array elements 
        /// (e.g., "gTestUDT_Array[0].Member1_DINT"). The PLC returns CIP Error 0x2107. 
        /// Workaround: Read the entire UDT array element, modify the member in memory, then write the entire element back.</description></item>
        /// <item><description><strong>STRING Members in UDTs:</strong> Cannot write directly to STRING members within UDTs 
        /// (e.g., "gTestUDT.Member5_String"). The PLC returns CIP Error 0x2107. 
        /// Workaround: Read the entire UDT, modify the STRING member in memory, then write the entire UDT back.</description></item>
        /// </list>
        /// <para><strong>✅ What Works:</strong> Writing to non-STRING members of non-array UDTs (e.g., "gTestUDT.Member1_DINT").</para>
        /// </remarks>
        public virtual void SetUdtMember(string tagName, string memberPath, PlcValue value)
        {
            var udtValue = ReadUdt(tagName);
            if (udtValue?.IsUdt != true)
                throw new Exception($"Tag '{tagName}' is not a UDT type or could not be read.");

            var members = udtValue.UdtMembers ?? throw new Exception($"Tag '{tagName}' returned raw UDT data without member definitions.");
            var parts = memberPath.Split('.');
            
            // Navigate to the parent of the target member
            for (int i = 0; i < parts.Length - 1; i++)
            {
                if (!members.ContainsKey(parts[i]))
                    throw new Exception($"Member path '{memberPath}' is invalid. '{parts[i]}' not found.");
                
                var nestedValue = members[parts[i]];
                if (!nestedValue.IsUdt)
                    throw new Exception($"Member path '{memberPath}' is invalid. '{parts[i]}' is not a UDT.");

                members = nestedValue.UdtMembers ?? throw new Exception($"Member path '{memberPath}' is invalid. '{parts[i]}' has no member definitions.");
            }

            // Set the final member
            members[parts[parts.Length - 1]] = value;

            // Write the updated UDT back
            WriteUdt(tagName, udtValue);
        }

        /// <summary>
        /// Reads a UDT using chunked reading to handle large structures that exceed packet size limits.
        /// This method automatically handles partial transfer errors by reading the UDT in smaller chunks.
        /// </summary>
        /// <param name="tagName">Name of the UDT tag to read.</param>
        /// <returns>PlcValue containing the UDT with nested structure support.</returns>
        public PlcValue ReadUdtChunked(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                IntPtr resultPtr = Marshal.AllocHGlobal(16384); // Larger buffer for chunked reading
                try
                {
                    int result = eip_read_udt_chunked(_clientId, tagPtr, resultPtr, 16384);
                    if (result != 0)
                        throw OperationFailure($"Failed to read UDT tag '{tagName}' with chunked reading. Check tag exists and is UDT type.");
                    
                    // Convert the JSON result to PlcValue
                    string jsonResult = Marshal.PtrToStringAnsi(resultPtr) ?? string.Empty;
                    if (string.IsNullOrEmpty(jsonResult))
                        throw new Exception($"Empty response when reading UDT tag '{tagName}' with chunked reading.");
                    
                    // Try to parse as UdtData first (new format)
                    try
                    {
                        var udtData = UdtData.FromJson(jsonResult);
                        return PlcValue.UdtFromData(udtData);
                    }
                    catch
                    {
                        // Fallback to legacy Dictionary format
                        return PlcValue.FromJson(jsonResult);
                    }
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                    Marshal.FreeHGlobal(resultPtr);
                }
            });
        }

        /// <summary>
        /// Reads a specific UDT member by offset, size, and data type.
        /// This method allows direct access to UDT members without needing the full UDT structure.
        /// </summary>
        /// <param name="udtName">Name of the UDT tag.</param>
        /// <param name="memberOffset">Byte offset of the member in the UDT.</param>
        /// <param name="memberSize">Size of the member in bytes.</param>
        /// <param name="dataType">CIP data type code (e.g., 0x00C1 for BOOL, 0x00CA for REAL).</param>
        /// <returns>PlcValue containing the member value.</returns>
        public PlcValue ReadUdtMemberByOffset(string udtName, int memberOffset, int memberSize, short dataType)
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr udtPtr = Marshal.StringToHGlobalAnsi(udtName);
                IntPtr resultPtr = Marshal.AllocHGlobal(1024);
                try
                {
                    int result = eip_read_udt_member_by_offset(_clientId, udtPtr, memberOffset, memberSize, dataType, resultPtr, 1024);
                    if (result != 0)
                        throw OperationFailure($"Failed to read UDT member at offset {memberOffset} from '{udtName}'. Check UDT exists and offset is valid.");
                    
                    // Convert the JSON result to PlcValue
                    string jsonResult = Marshal.PtrToStringAnsi(resultPtr) ?? string.Empty;
                    if (string.IsNullOrEmpty(jsonResult))
                        throw new Exception($"Empty response when reading UDT member from '{udtName}' at offset {memberOffset}.");
                    
                    return PlcValue.FromJson(jsonResult);
                }
                finally
                {
                    Marshal.FreeHGlobal(udtPtr);
                    Marshal.FreeHGlobal(resultPtr);
                }
            });
        }

        /// <summary>
        /// Writes a specific UDT member by offset, size, and data type.
        /// This method allows direct writing to UDT members without needing the full UDT structure.
        /// </summary>
        /// <param name="udtName">Name of the UDT tag.</param>
        /// <param name="memberOffset">Byte offset of the member in the UDT.</param>
        /// <param name="memberSize">Size of the member in bytes.</param>
        /// <param name="dataType">CIP data type code (e.g., 0x00C1 for BOOL, 0x00CA for REAL).</param>
        /// <param name="value">PlcValue containing the value to write.</param>
        public void WriteUdtMemberByOffset(string udtName, int memberOffset, int memberSize, short dataType, PlcValue value)
        {
            _ = value ?? throw new ArgumentNullException(nameof(value));

            ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr udtPtr = Marshal.StringToHGlobalAnsi(udtName);
                try
                {
                    // Serialize the value to JSON
                    string jsonValue = value.ToJson();
                    IntPtr valuePtr = Marshal.StringToHGlobalAnsi(jsonValue);
                    try
                    {
                        int result = eip_write_udt_member_by_offset(_clientId, udtPtr, memberOffset, memberSize, dataType, valuePtr, jsonValue.Length);
                        if (result != 0)
                            throw OperationFailure($"Failed to write UDT member at offset {memberOffset} to '{udtName}'. Check UDT exists and offset is valid.");
                    }
                    finally
                    {
                        Marshal.FreeHGlobal(valuePtr);
                    }
                }
                finally
                {
                    Marshal.FreeHGlobal(udtPtr);
                }
            });
        }

        /// <summary>
        /// Retrieves the UDT definition (member layout) for a given UDT tag.
        /// </summary>
        /// <param name="udtName">Name of the UDT tag.</param>
        /// <returns>UDT template describing members, types, and offsets.</returns>
        public UdtTemplate GetUdtDefinition(string udtName)
        {
            if (string.IsNullOrWhiteSpace(udtName))
                throw new ArgumentException("UDT name cannot be null or empty", nameof(udtName));

            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr udtPtr = Marshal.StringToHGlobalAnsi(udtName);
                IntPtr resultPtr = Marshal.AllocHGlobal(Marshal.SizeOf<UdtDefinitionResultNative>());
                try
                {
                    Marshal.StructureToPtr(new UdtDefinitionResultNative(), resultPtr, false);
                    int result = eip_get_udt_definition_by_id(_clientId, udtPtr, resultPtr);
                    var native = Marshal.PtrToStructure<UdtDefinitionResultNative>(resultPtr);

                    if (result != 0 || !native.Success)
                    {
                        string error = PtrToStringAnsiSafe(native.ErrorMessage);
                        if (string.IsNullOrWhiteSpace(error))
                            error = "Unknown error";
                        throw OperationFailure($"Failed to get UDT definition for '{udtName}': {error}");
                    }

                    var template = new UdtTemplate
                    {
                        Name = PtrToStringAnsiSafe(native.Name),
                        Members = new List<UdtMemberTemplate>()
                    };

                    int totalSize = 0;
                    if (native.Members != IntPtr.Zero && native.MemberCount > 0)
                    {
                        int memberSize = Marshal.SizeOf<UdtMemberNative>();
                        for (int i = 0; i < native.MemberCount; i++)
                        {
                            IntPtr memberPtr = IntPtr.Add(native.Members, i * memberSize);
                            var member = Marshal.PtrToStructure<UdtMemberNative>(memberPtr);

                            int endOffset = member.Offset + member.Size;
                            if (endOffset > totalSize)
                                totalSize = endOffset;

                            template.Members.Add(new UdtMemberTemplate
                            {
                                Name = PtrToStringAnsiSafe(member.Name),
                                DataType = CipDataTypeName(member.DataType),
                                Size = member.Size,
                                Offset = member.Offset
                            });
                        }
                    }

                    if (string.IsNullOrWhiteSpace(template.Name))
                        template.Name = udtName;

                    template.TotalSize = totalSize;
                    return template;
                }
                finally
                {
                    eip_free_udt_definition(resultPtr);
                    Marshal.FreeHGlobal(resultPtr);
                    Marshal.FreeHGlobal(udtPtr);
                }
            });
        }

        /// <summary>
        /// Retrieves detailed attributes for a specific tag.
        /// </summary>
        /// <param name="tagName">Name of the tag.</param>
        /// <returns>Tag attributes including type, size, and template instance ID.</returns>
        public TagAttributes GetTagAttributes(string tagName)
        {
            if (string.IsNullOrWhiteSpace(tagName))
                throw new ArgumentException("Tag name cannot be null or empty", nameof(tagName));

            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                IntPtr resultPtr = Marshal.AllocHGlobal(Marshal.SizeOf<TagAttributesResultNative>());
                try
                {
                    Marshal.StructureToPtr(new TagAttributesResultNative(), resultPtr, false);
                    int result = eip_get_tag_attributes_by_id(_clientId, tagPtr, resultPtr);
                    var native = Marshal.PtrToStructure<TagAttributesResultNative>(resultPtr);

                    if (result != 0 || !native.Success)
                    {
                        string error = PtrToStringAnsiSafe(native.ErrorMessage);
                        if (string.IsNullOrWhiteSpace(error))
                            error = "Unknown error";
                        throw OperationFailure($"Failed to get tag attributes for '{tagName}': {error}");
                    }

                    string typeName = PtrToStringAnsiSafe(native.DataTypeName);
                    if (string.IsNullOrWhiteSpace(typeName))
                        typeName = CipDataTypeName(native.DataType);

                    return new TagAttributes
                    {
                        Name = PtrToStringAnsiSafe(native.Name),
                        DataTypeName = typeName,
                        DataType = native.DataType,
                        Size = native.Size,
                        TemplateInstanceId = native.TemplateInstanceId
                    };
                }
                finally
                {
                    eip_free_tag_attributes_result(resultPtr);
                    Marshal.FreeHGlobal(resultPtr);
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        /// <summary>
        /// Discovers tags and returns detailed attributes for each tag.
        /// </summary>
        /// <returns>List of tag attributes discovered on the PLC.</returns>
        public List<TagAttributes> DiscoverTagsDetailed()
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr resultPtr = Marshal.AllocHGlobal(Marshal.SizeOf<TagDiscoveryResultNative>());
                try
                {
                    Marshal.StructureToPtr(new TagDiscoveryResultNative(), resultPtr, false);
                    int result = eip_discover_tags_detailed_by_id(_clientId, resultPtr);
                    var native = Marshal.PtrToStructure<TagDiscoveryResultNative>(resultPtr);

                    if (result != 0 || !native.Success)
                    {
                        string error = PtrToStringAnsiSafe(native.ErrorMessage);
                        if (string.IsNullOrWhiteSpace(error))
                            error = "Unknown error";
                        throw OperationFailure($"Failed to discover tags: {error}");
                    }

                    var tags = new List<TagAttributes>();
                    if (native.Tags != IntPtr.Zero && native.TagCount > 0)
                    {
                        int tagSize = Marshal.SizeOf<TagAttributesNative>();
                        for (int i = 0; i < native.TagCount; i++)
                        {
                            IntPtr tagPtr = IntPtr.Add(native.Tags, i * tagSize);
                            var tag = Marshal.PtrToStructure<TagAttributesNative>(tagPtr);

                            string typeName = PtrToStringAnsiSafe(tag.DataTypeName);
                            if (string.IsNullOrWhiteSpace(typeName))
                                typeName = CipDataTypeName(tag.DataType);

                            tags.Add(new TagAttributes
                            {
                                Name = PtrToStringAnsiSafe(tag.Name),
                                DataTypeName = typeName,
                                DataType = tag.DataType,
                                Size = tag.Size,
                                TemplateInstanceId = tag.TemplateInstanceId
                            });
                        }
                    }

                    return tags;
                }
                finally
                {
                    eip_free_tag_discovery_result(resultPtr);
                    Marshal.FreeHGlobal(resultPtr);
                }
            });
        }

        /// <summary>
        /// Writes a specific UDT member to the PLC.
        /// </summary>
        /// <param name="udtName">Name of the UDT (e.g., "Part_Data").</param>
        /// <param name="memberName">Name of the UDT member (e.g., "oMachine_Running").</param>
        /// <param name="value">Value to write to the UDT member.</param>
        public void WriteUdtMember(string udtName, string memberName, PlcValue value)
        {
            if (string.IsNullOrEmpty(udtName))
                throw new ArgumentException("UDT name cannot be null or empty", nameof(udtName));
            if (string.IsNullOrEmpty(memberName))
                throw new ArgumentException("Member name cannot be null or empty", nameof(memberName));
            _ = value ?? throw new ArgumentNullException(nameof(value));

            ExecuteWithLock(() =>
            {
                CheckConnection();

                // Read the entire UDT, modify the specific member, and write it back
                var udtValue = ReadUdt(udtName);
                if (udtValue.IsUdt)
                {
                    var udtMembers = udtValue.UdtMembers ?? throw new InvalidOperationException($"UDT '{udtName}' returned raw data without member definitions.");
                    // Create a new UDT with the updated member
                    var updatedMembers = new Dictionary<string, PlcValue>(udtMembers);
                    updatedMembers[memberName] = value;

                    var updatedUdt = PlcValue.Udt(updatedMembers);
                    WriteUdt(udtName, updatedUdt);
                }
                else
                {
                    throw new InvalidOperationException($"Failed to read UDT {udtName}");
                }
            });
        }

        #endregion

        #region Helper Methods

        private static string PtrToStringAnsiSafe(IntPtr ptr)
        {
            return ptr == IntPtr.Zero ? string.Empty : Marshal.PtrToStringAnsi(ptr) ?? string.Empty;
        }

        private static string CipDataTypeName(short dataType)
        {
            return dataType switch
            {
                0x00C1 => "BOOL",
                0x00C2 => "SINT",
                0x00C3 => "INT",
                0x00C4 => "DINT",
                0x00C5 => "LINT",
                0x00C6 => "USINT",
                0x00C7 => "UINT",
                0x00C8 => "UDINT",
                0x00C9 => "ULINT",
                0x00CA => "REAL",
                0x00CB => "LREAL",
                0x00CE => "STRING",
                _ => "UNKNOWN"
            };
        }

        private static bool TryParseUdtMemberPath(string tagName, out string baseTag, out string memberPath)
        {
            baseTag = string.Empty;
            memberPath = string.Empty;

            if (string.IsNullOrWhiteSpace(tagName))
                return false;

            if (tagName.Contains(".LEN", StringComparison.OrdinalIgnoreCase) ||
                tagName.Contains(".DATA[", StringComparison.OrdinalIgnoreCase))
                return false;

            int dotIndex = tagName.LastIndexOf('.');
            if (dotIndex <= 0 || dotIndex >= tagName.Length - 1)
                return false;

            var lastSegment = tagName[(dotIndex + 1)..];
            if (lastSegment.All(char.IsDigit))
                return false;

            baseTag = tagName[..dotIndex];
            memberPath = lastSegment;
            return true;
        }

        private static PlcValue ConvertJsonElementToPlcValue(System.Text.Json.JsonElement jsonElement)
        {
            return jsonElement.ValueKind switch
            {
                System.Text.Json.JsonValueKind.True => PlcValue.Bool(true),
                System.Text.Json.JsonValueKind.False => PlcValue.Bool(false),
                System.Text.Json.JsonValueKind.Number => jsonElement.TryGetInt32(out var intValue)
                    ? PlcValue.Dint(intValue)
                    : PlcValue.Real((float)jsonElement.GetDouble()),
                System.Text.Json.JsonValueKind.String => PlcValue.String(jsonElement.GetString() ?? string.Empty),
                _ => throw new ArgumentException($"Unsupported JSON value kind: {jsonElement.ValueKind}")
            };
        }

        /// <summary>
        /// Converts a .NET object to a PlcValue
        /// </summary>
        private PlcValue ConvertObjectToPlcValue(object value)
        {
            return value switch
            {
                bool b => PlcValue.Bool(b),
                sbyte sb => PlcValue.Sint(sb),
                short s => PlcValue.Int(s),
                int i => PlcValue.Dint(i),
                long l => PlcValue.Lint(l),
                byte b => PlcValue.Usint(b),
                ushort us => PlcValue.Uint(us),
                uint ui => PlcValue.Udint(ui),
                ulong ul => PlcValue.Ulint(ul),
                float f => PlcValue.Real(f),
                double d => PlcValue.Lreal(d),
                string str => PlcValue.String(str),
                Dictionary<string, object> dict => PlcValue.Udt(ConvertDictionaryToPlcValueDict(dict)),
                _ => throw new ArgumentException($"Unsupported object type: {value?.GetType()}")
            };
        }

        /// <summary>
        /// Converts a dictionary of objects to a dictionary of PlcValues
        /// </summary>
        private Dictionary<string, PlcValue> ConvertDictionaryToPlcValueDict(Dictionary<string, object> dict)
        {
            var result = new Dictionary<string, PlcValue>();
            foreach (var kvp in dict)
            {
                result[kvp.Key] = ConvertObjectToPlcValue(kvp.Value);
            }
            return result;
        }

        /// <summary>
        /// Converts a UDT dictionary to a regular object dictionary
        /// </summary>
        private Dictionary<string, object> ConvertUdtToDictionary(Dictionary<string, PlcValue> udtMembers)
        {
            var result = new Dictionary<string, object>();
            foreach (var kvp in udtMembers)
            {
                result[kvp.Key] = ConvertPlcValueToObject(kvp.Value);
            }
            return result;
        }

        /// <summary>
        /// Converts a PlcValue to a .NET object
        /// </summary>
        private object ConvertPlcValueToObject(PlcValue value)
        {
            if (value.Type == PlcValueType.Udt)
            {
                var udtMembers = value.UdtMembers;
                if (udtMembers != null)
                    return ConvertUdtToDictionary(udtMembers);
                
                var udtData = value.UdtData;
                if (udtData != null)
                    return udtData;
            }

            return value.Value;
        }

        private static string ToRustValueType(PlcValue value) => value.Type switch
        {
            PlcValueType.Bool => "BOOL",
            PlcValueType.Sint => "SINT",
            PlcValueType.Int => "INT",
            PlcValueType.Dint => "DINT",
            PlcValueType.Lint => "LINT",
            PlcValueType.Usint => "USINT",
            PlcValueType.Uint => "UINT",
            PlcValueType.Udint => "UDINT",
            PlcValueType.Ulint => "ULINT",
            PlcValueType.Real => "REAL",
            PlcValueType.Lreal => "LREAL",
            PlcValueType.String => "STRING",
            PlcValueType.Udt => "UDT",
            _ => throw new ArgumentOutOfRangeException(nameof(value), $"Unsupported PlcValueType: {value.Type}")
        };

        private static object ToRustRawValue(PlcValue value)
        {
            if (value.Type == PlcValueType.Udt)
            {
                var udtData = value.UdtData;
                if (udtData == null)
                    throw new ArgumentException("Batch UDT write requires UdtData format with symbol_id and raw bytes.");

                return new
                {
                    symbol_id = udtData.SymbolId,
                    // Rust expects Vec<u8> as a JSON numeric array, not base64 text.
                    data = Array.ConvertAll(udtData.Data, b => (int)b)
                };
            }

            return value.Value;
        }

        private static JsonElement ToJsonElement(object value) =>
            JsonSerializer.SerializeToElement(value);

        #endregion

        #region Batch Operations

        /// <summary>
        /// Read multiple tags and return per-tag results.
        /// Current implementation executes reads sequentially.
        /// </summary>
        /// <param name="tagNames">Array of tag names to read</param>
        /// <returns>Dictionary of tag names to read results</returns>
        /// <exception cref="ArgumentException">Thrown if tagNames array is null or empty</exception>
        /// <exception cref="InvalidOperationException">Thrown if not connected to PLC</exception>
        public Dictionary<string, TagReadResultBatch> ReadTagsBatch(string[] tagNames)
        {
            if (tagNames == null || tagNames.Length == 0)
                throw new ArgumentException("Tag names array cannot be null or empty", nameof(tagNames));

            var nativeResults = TryReadTagsBatchNative(tagNames);
            if (nativeResults != null)
            {
                var failedTags = nativeResults.Values
                    .Where(v => !v.Success)
                    .Select(v => v.TagName)
                    .ToArray();

                if (failedTags.Length == 0)
                    return nativeResults;

                var fallbackResults = ReadTagsBatchSequential(failedTags);
                foreach (var failedTag in failedTags)
                {
                    if (fallbackResults.TryGetValue(failedTag, out var fallbackResult))
                        nativeResults[failedTag] = fallbackResult;
                }

                return nativeResults;
            }

            return ReadTagsBatchSequential(tagNames);
        }

        private Dictionary<string, TagReadResultBatch> ReadTagsBatchSequential(string[] tagNames)
        {
            var results = new Dictionary<string, TagReadResultBatch>();

            foreach (string tagName in tagNames)
            {
                try
                {
                    // Try multiple data types to find the correct one
                    object? value = null;
                    string dataType = "UNKNOWN";
                    bool success = false;
                    Exception? lastException = null;

                    // Try BOOL first
                    try
                    {
                        value = ReadBool(tagName);
                        dataType = "BOOL";
                        success = true;
                    }
                    catch (Exception ex) { lastException = ex; }

                    // Try DINT if BOOL failed
                    if (!success)
                    {
                        try
                        {
                            value = ReadDint(tagName);
                            dataType = "DINT";
                            success = true;
                        }
                        catch (Exception ex) { lastException = ex; }
                    }

                    // Try INT if DINT failed
                    if (!success)
                    {
                        try
                        {
                            value = ReadInt(tagName);
                            dataType = "INT";
                            success = true;
                        }
                        catch (Exception ex) { lastException = ex; }
                    }

                    // Try REAL if INT failed
                    if (!success)
                    {
                        try
                        {
                            value = ReadReal(tagName);
                            dataType = "REAL";
                            success = true;
                        }
                        catch (Exception ex) { lastException = ex; }
                    }

                    // Try STRING if REAL failed
                    if (!success)
                    {
                        try
                        {
                            value = ReadString(tagName);
                            dataType = "STRING";
                            success = true;
                        }
                        catch (Exception ex) 
                        { 
                            lastException = ex;
                            // STRING reads are supported, but direct STRING writes are limited by PLC firmware.
                        }
                    }

                    // Try SINT if STRING failed
                    if (!success)
                    {
                        try
                        {
                            value = ReadSint(tagName);
                            dataType = "SINT";
                            success = true;
                        }
                        catch (Exception ex) { lastException = ex; }
                    }

                    if (success)
                    {
                        results[tagName] = new TagReadResultBatch
                        {
                            TagName = tagName,
                            Success = true,
                            Value = value,
                            DataType = dataType,
                            ErrorCode = 0,
                            ErrorMessage = null
                        };
                    }
                    else
                    {
                        results[tagName] = new TagReadResultBatch
                        {
                            TagName = tagName,
                            Success = false,
                            Value = null,
                            DataType = "UNKNOWN",
                            ErrorCode = -1,
                            ErrorMessage = lastException?.Message
                        };
                    }
                }
                catch (Exception ex)
                {
                    results[tagName] = new TagReadResultBatch
                    {
                        TagName = tagName,
                        Success = false,
                        Value = null,
                        DataType = "UNKNOWN",
                        ErrorCode = -1,
                        ErrorMessage = ex.Message
                    };
                }
            }

            return results;
        }

        private Dictionary<string, TagReadResultBatch>? TryReadTagsBatchNative(string[] tagNames)
        {
            try
            {
                return ExecuteWithLock(() =>
                {
                    CheckConnection();
                    var tagPtrs = tagNames.Select(Marshal.StringToHGlobalAnsi).ToArray();
                    IntPtr resultPtr = IntPtr.Zero;

                    try
                    {
                        resultPtr = Marshal.AllocHGlobal(131072);
                        int rc = eip_read_tags_batch(_clientId, tagPtrs, tagPtrs.Length, resultPtr, 131072);
                        if (rc != 0)
                            return null;

                        string payload = Marshal.PtrToStringAnsi(resultPtr) ?? string.Empty;
                        if (string.IsNullOrWhiteSpace(payload))
                            return null;

                        return ParseNativeBatchReadResults(payload, tagNames);
                    }
                    finally
                    {
                        if (resultPtr != IntPtr.Zero)
                            Marshal.FreeHGlobal(resultPtr);

                        foreach (var ptr in tagPtrs)
                        {
                            if (ptr != IntPtr.Zero)
                                Marshal.FreeHGlobal(ptr);
                        }
                    }
                });
            }
            catch
            {
                return null;
            }
        }

        private Dictionary<string, TagReadResultBatch>? ParseNativeBatchReadResults(string payload, string[] requestedTags)
        {
            var jsonResults = TryParseNativeBatchReadResultsJson(payload, requestedTags);
            if (jsonResults != null)
                return jsonResults;

            var results = new Dictionary<string, TagReadResultBatch>(requestedTags.Length);
            var entries = payload.Split(';', StringSplitOptions.RemoveEmptyEntries);

            foreach (var entry in entries)
            {
                int separator = entry.IndexOf(':');
                if (separator <= 0)
                    return null;

                string tagName = entry.Substring(0, separator);
                string rawValue = entry.Substring(separator + 1);

                if (rawValue.StartsWith("ERROR:", StringComparison.Ordinal))
                {
                    results[tagName] = new TagReadResultBatch
                    {
                        TagName = tagName,
                        Success = false,
                        Value = null,
                        DataType = "UNKNOWN",
                        ErrorCode = -1,
                        ErrorMessage = rawValue.Substring("ERROR:".Length)
                    };
                    continue;
                }

                if (!TryParseNativeBatchValue(rawValue, out var value, out var dataType))
                    return null;

                results[tagName] = new TagReadResultBatch
                {
                    TagName = tagName,
                    Success = true,
                    Value = value,
                    DataType = dataType,
                    ErrorCode = 0,
                    ErrorMessage = null
                };
            }

            if (results.Count != requestedTags.Length)
                return null;

            return results;
        }

        private static Dictionary<string, TagReadResultBatch>? TryParseNativeBatchReadResultsJson(string payload, string[] requestedTags)
        {
            try
            {
                var entries = JsonSerializer.Deserialize<List<FfiReadBatchResultItem>>(payload);
                if (entries == null || entries.Count != requestedTags.Length)
                    return null;

                var results = new Dictionary<string, TagReadResultBatch>(requestedTags.Length);
                foreach (var entry in entries)
                {
                    if (string.IsNullOrWhiteSpace(entry.tag_name))
                        return null;

                    object? value = null;
                    string dataType = "UNKNOWN";

                    if (entry.success)
                    {
                        if (!TryConvertBatchReadJsonValue(entry.value, out value, out dataType))
                            return null;
                    }

                    results[entry.tag_name] = new TagReadResultBatch
                    {
                        TagName = entry.tag_name,
                        Success = entry.success,
                        Value = value,
                        DataType = entry.success ? dataType : "UNKNOWN",
                        ErrorCode = entry.success ? 0 : -1,
                        ErrorMessage = entry.success ? null : entry.error
                    };
                }

                return results.Count == requestedTags.Length ? results : null;
            }
            catch
            {
                return null;
            }
        }

        private static bool TryConvertBatchReadJsonValue(JsonElement? jsonValue, out object? value, out string dataType)
        {
            value = null;
            dataType = "UNKNOWN";

            if (jsonValue == null)
                return false;

            var plcValue = PlcValue.FromJson(jsonValue.Value.GetRawText());
            value = plcValue.Value;
            dataType = plcValue.Type switch
            {
                PlcValueType.Bool => "BOOL",
                PlcValueType.Sint => "SINT",
                PlcValueType.Int => "INT",
                PlcValueType.Dint => "DINT",
                PlcValueType.Lint => "LINT",
                PlcValueType.Usint => "USINT",
                PlcValueType.Uint => "UINT",
                PlcValueType.Udint => "UDINT",
                PlcValueType.Ulint => "ULINT",
                PlcValueType.Real => "REAL",
                PlcValueType.Lreal => "LREAL",
                PlcValueType.String => "STRING",
                PlcValueType.Udt => "UDT",
                _ => "UNKNOWN"
            };
            return true;
        }

        private static bool TryParseNativeBatchValue(string rawValue, out object? value, out string dataType)
        {
            value = null;
            dataType = "UNKNOWN";

            if (rawValue.StartsWith("Bool(", StringComparison.Ordinal) && rawValue.EndsWith(')'))
            {
                value = bool.Parse(rawValue.Substring(5, rawValue.Length - 6));
                dataType = "BOOL";
                return true;
            }

            if (rawValue.StartsWith("Dint(", StringComparison.Ordinal) && rawValue.EndsWith(')'))
            {
                value = int.Parse(rawValue.Substring(5, rawValue.Length - 6));
                dataType = "DINT";
                return true;
            }

            if (rawValue.StartsWith("Int(", StringComparison.Ordinal) && rawValue.EndsWith(')'))
            {
                value = short.Parse(rawValue.Substring(4, rawValue.Length - 5));
                dataType = "INT";
                return true;
            }

            if (rawValue.StartsWith("Real(", StringComparison.Ordinal) && rawValue.EndsWith(')'))
            {
                value = float.Parse(rawValue.Substring(5, rawValue.Length - 6), System.Globalization.CultureInfo.InvariantCulture);
                dataType = "REAL";
                return true;
            }

            if (rawValue.StartsWith("String(\"", StringComparison.Ordinal) && rawValue.EndsWith("\")", StringComparison.Ordinal))
            {
                value = rawValue.Substring(8, rawValue.Length - 10).Replace("\\\"", "\"");
                dataType = "STRING";
                return true;
            }

            if (rawValue.StartsWith("Udint(", StringComparison.Ordinal) && rawValue.EndsWith(')'))
            {
                value = uint.Parse(rawValue.Substring(6, rawValue.Length - 7));
                dataType = "UDINT";
                return true;
            }

            return false;
        }

        /// <summary>
        /// Write multiple tags using native Rust batch execution and return per-tag results.
        /// </summary>
        /// <param name="tagValues">Dictionary of tag names to values to write</param>
        /// <returns>Dictionary of tag names to write results</returns>
        /// <exception cref="ArgumentException">Thrown if tagValues dictionary is null or empty</exception>
        /// <exception cref="InvalidOperationException">Thrown if not connected to PLC</exception>
        public Dictionary<string, TagWriteResult> WriteTagsBatch(Dictionary<string, object> tagValues)
        {
            if (tagValues == null || tagValues.Count == 0)
                throw new ArgumentException("Tag values dictionary cannot be null or empty", nameof(tagValues));

            var results = new Dictionary<string, TagWriteResult>();

            // Preserve explicit UDT-member behavior (used by tests and by PLC limitation workarounds).
            var ffiOperations = new List<FfiWriteBatchRequestItem>();
            foreach (var entry in tagValues)
            {
                try
                {
                    if (TryParseUdtMemberPath(entry.Key, out var baseTag, out var memberPath))
                    {
                        var plcValue = entry.Value is System.Text.Json.JsonElement jsonElement
                            ? ConvertJsonElementToPlcValue(jsonElement)
                            : ConvertObjectToPlcValue(entry.Value);

                        SetUdtMember(baseTag, memberPath, plcValue);
                        results[entry.Key] = new TagWriteResult
                        {
                            TagName = entry.Key,
                            Success = true,
                            ErrorCode = 0,
                            ErrorMessage = null
                        };
                        continue;
                    }

                    var plcBatchValue = entry.Value is System.Text.Json.JsonElement elem
                        ? ConvertJsonElementToPlcValue(elem)
                        : ConvertObjectToPlcValue(entry.Value);

                    ffiOperations.Add(new FfiWriteBatchRequestItem
                    {
                        tag_name = entry.Key,
                        value_type = ToRustValueType(plcBatchValue),
                        value = ToJsonElement(ToRustRawValue(plcBatchValue))
                    });
                }
                catch (Exception ex)
                {
                    results[entry.Key] = new TagWriteResult
                    {
                        TagName = entry.Key,
                        Success = false,
                        ErrorCode = -1,
                        ErrorMessage = ex.Message
                    };
                }
            }

            if (ffiOperations.Count == 0)
                return results;

            try
            {
                var payload = JsonSerializer.Serialize(ffiOperations);
                ExecuteWithLock(() =>
                {
                    IntPtr payloadPtr = Marshal.StringToHGlobalAnsi(payload);
                    IntPtr resultPtr = Marshal.AllocHGlobal(65536);
                    try
                    {
                        int rc = eip_write_tags_batch(_clientId, payloadPtr, ffiOperations.Count, resultPtr, 65536);
                        var json = Marshal.PtrToStringAnsi(resultPtr) ?? string.Empty;

                        if (string.IsNullOrWhiteSpace(json))
                        {
                            foreach (var op in ffiOperations)
                            {
                                results[op.tag_name] = new TagWriteResult
                                {
                                    TagName = op.tag_name,
                                    Success = false,
                                    ErrorCode = -1,
                                    ErrorMessage = "Empty native batch write response"
                                };
                            }
                            return;
                        }

                        var nativeResults = JsonSerializer.Deserialize<List<FfiWriteBatchResultItem>>(json) ?? new();
                        foreach (var native in nativeResults)
                        {
                            results[native.tag_name] = new TagWriteResult
                            {
                                TagName = native.tag_name,
                                Success = native.success && rc == 0,
                                ErrorCode = native.success && rc == 0 ? 0 : -1,
                                ErrorMessage = native.error ?? (rc == 0 ? null : "Native batch write failed")
                            };
                        }

                        foreach (var op in ffiOperations)
                        {
                            if (results.ContainsKey(op.tag_name))
                                continue;

                            results[op.tag_name] = new TagWriteResult
                            {
                                TagName = op.tag_name,
                                Success = false,
                                ErrorCode = -1,
                                ErrorMessage = "Missing native batch write result"
                            };
                        }
                    }
                    finally
                    {
                        Marshal.FreeHGlobal(payloadPtr);
                        Marshal.FreeHGlobal(resultPtr);
                    }
                });
            }
            catch (Exception ex)
            {
                foreach (var op in ffiOperations)
                {
                    results[op.tag_name] = new TagWriteResult
                    {
                        TagName = op.tag_name,
                        Success = false,
                        ErrorCode = -1,
                        ErrorMessage = ex.Message
                    };
                }
            }

            return results;
        }

        /// <summary>
        /// Configure batch operation behavior for performance optimization.
        /// Currently unsupported in native Rust FFI for this release line.
        /// </summary>
        /// <param name="config">Batch configuration settings</param>
        /// <exception cref="ArgumentNullException">Thrown if config is null</exception>
        /// <exception cref="NotSupportedException">Always thrown until native support is implemented</exception>
        public void ConfigureBatchOperations(BatchConfig config)
        {
            _ = config ?? throw new ArgumentNullException(nameof(config));

            throw new NotSupportedException(
                "Batch configuration is not implemented yet in the native Rust FFI."
            );
        }

        /// <summary>
        /// Get current batch operation configuration.
        /// Currently unsupported in native Rust FFI for this release line.
        /// </summary>
        /// <returns>Current batch configuration</returns>
        /// <exception cref="NotSupportedException">Always thrown until native support is implemented</exception>
        public BatchConfig GetBatchConfig()
        {
            throw new NotSupportedException(
                "Batch configuration is not implemented yet in the native Rust FFI."
            );
        }

        /// <summary>
        /// Execute a mixed set of read and write operations using native Rust batch execution.
        /// </summary>
        /// <param name="operations">Array of batch operations to execute</param>
        /// <returns>Array of batch operation results</returns>
        /// <exception cref="ArgumentException">Thrown if operations array is null or empty</exception>
        /// <exception cref="InvalidOperationException">Thrown if not connected to PLC</exception>
        public BatchOperationResult[] ExecuteBatch(BatchOperation[] operations)
        {
            if (operations == null || operations.Length == 0)
                throw new ArgumentException("Operations array cannot be null or empty", nameof(operations));

            var results = new BatchOperationResult[operations.Length];
            var ffiOperations = new List<FfiExecuteBatchRequestItem>(operations.Length);

            for (int i = 0; i < operations.Length; i++)
            {
                try
                {
                    var op = operations[i];
                    if (op.IsWrite)
                    {
                        if (op.Value == null)
                            throw new ArgumentException($"Write operation '{op.TagName}' is missing a value");

                        var plcValue = op.Value is JsonElement jsonElement
                            ? ConvertJsonElementToPlcValue(jsonElement)
                            : ConvertObjectToPlcValue(op.Value);

                        ffiOperations.Add(new FfiExecuteBatchRequestItem
                        {
                            tag_name = op.TagName,
                            is_write = true,
                            value_type = ToRustValueType(plcValue),
                            value = ToJsonElement(ToRustRawValue(plcValue))
                        });
                    }
                    else
                    {
                        ffiOperations.Add(new FfiExecuteBatchRequestItem
                        {
                            tag_name = op.TagName,
                            is_write = false,
                            value_type = null,
                            value = null
                        });
                    }
                }
                catch (Exception ex)
                {
                    results[i] = new BatchOperationResult
                    {
                        TagName = operations[i].TagName,
                        IsWrite = operations[i].IsWrite,
                        Success = false,
                        Value = null,
                        ExecutionTimeMs = 0,
                        ErrorCode = -1,
                        ErrorMessage = ex.Message
                    };
                }
            }

            if (ffiOperations.Count == 0)
                return results;

            try
            {
                var payload = JsonSerializer.Serialize(ffiOperations);
                ExecuteWithLock(() =>
                {
                    IntPtr payloadPtr = Marshal.StringToHGlobalAnsi(payload);
                    IntPtr resultPtr = Marshal.AllocHGlobal(131072);
                    try
                    {
                        int rc = eip_execute_batch(_clientId, payloadPtr, ffiOperations.Count, resultPtr, 131072);
                        string json = Marshal.PtrToStringAnsi(resultPtr) ?? string.Empty;
                        if (string.IsNullOrWhiteSpace(json))
                            throw new Exception("Empty native execute-batch response");

                        var nativeResults = JsonSerializer.Deserialize<List<FfiExecuteBatchResultItem>>(json) ?? new();
                        foreach (var native in nativeResults)
                        {
                            if (native.index < 0 || native.index >= results.Length)
                                continue;

                            results[native.index] = new BatchOperationResult
                            {
                                TagName = native.tag_name,
                                IsWrite = native.is_write,
                                Success = native.success && rc == 0,
                                Value = native.value.HasValue ? ConvertPlcValueToObject(PlcValue.FromJson(native.value.Value.GetRawText())) : null,
                                ExecutionTimeMs = native.execution_time_us / 1000.0,
                                ErrorCode = native.success && rc == 0 ? 0 : -1,
                                ErrorMessage = native.error ?? (rc == 0 ? null : "Native execute-batch failed")
                            };
                        }
                    }
                    finally
                    {
                        Marshal.FreeHGlobal(payloadPtr);
                        Marshal.FreeHGlobal(resultPtr);
                    }
                });
            }
            catch (Exception ex)
            {
                for (int i = 0; i < results.Length; i++)
                {
                    if (results[i] is { } existing &&
                        (existing.Success || !string.IsNullOrEmpty(existing.ErrorMessage)))
                        continue;
                    results[i] = new BatchOperationResult
                    {
                        TagName = operations[i].TagName,
                        IsWrite = operations[i].IsWrite,
                        Success = false,
                        Value = null,
                        ExecutionTimeMs = 0,
                        ErrorCode = -1,
                        ErrorMessage = ex.Message
                    };
                }
            }

            for (int i = 0; i < results.Length; i++)
            {
                if (results[i] != null)
                    continue;

                results[i] = new BatchOperationResult
                {
                    TagName = operations[i].TagName,
                    IsWrite = operations[i].IsWrite,
                    Success = false,
                    Value = null,
                    ExecutionTimeMs = 0,
                    ErrorCode = -1,
                    ErrorMessage = "Missing execute-batch result"
                };
            }

            return results;
        }

        /// <summary>
        /// Reads a tag of any type and returns it as a PlcValue.
        /// This is a generic read method that automatically detects the tag type.
        /// </summary>
        /// <param name="tagName">Name of the tag to read</param>
        /// <returns>PlcValue containing the tag value</returns>
        private PlcValue ReadTag(string tagName)
        {
            var sw = System.Diagnostics.Stopwatch.StartNew();
            try
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                IntPtr resultPtr = Marshal.AllocHGlobal(4096);
                try
                {
                    int result = eip_read_tag(_clientId, tagPtr, resultPtr, 4096);
                    if (result != 0)
                        throw OperationFailure($"Failed to read tag '{tagName}'");

                    string jsonResult = Marshal.PtrToStringAnsi(resultPtr) ?? string.Empty;
                    if (string.IsNullOrEmpty(jsonResult))
                        throw new Exception($"Empty response for tag '{tagName}'");

                    _statistics.IncrementRead();
                    return PlcValue.FromJson(jsonResult);
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                    Marshal.FreeHGlobal(resultPtr);
                }
            }
            catch
            {
                _statistics.IncrementError();
                throw;
            }
            finally
            {
                sw.Stop();
                _statistics.AddResponseTime(sw.ElapsedMilliseconds);
            }
        }

        /// <summary>
        /// Writes a tag of any type using a PlcValue.
        /// This is a generic write method that automatically handles the tag type.
        /// </summary>
        /// <param name="tagName">Name of the tag to write</param>
        /// <param name="value">PlcValue containing the value to write</param>
        private void WriteTag(string tagName, PlcValue value)
        {
            CheckConnection();
            
            // Use the appropriate write method based on PlcValue type
            switch (value.Type)
            {
                case PlcValueType.Bool:
                    WriteBool(tagName, value.As<bool>());
                    break;
                case PlcValueType.Sint:
                    WriteSint(tagName, value.As<sbyte>());
                    break;
                case PlcValueType.Int:
                    WriteInt(tagName, value.As<short>());
                    break;
                case PlcValueType.Dint:
                    WriteDint(tagName, value.As<int>());
                    break;
                case PlcValueType.Lint:
                    WriteLint(tagName, value.As<long>());
                    break;
                case PlcValueType.Usint:
                    WriteUsint(tagName, value.As<byte>());
                    break;
                case PlcValueType.Uint:
                    WriteUint(tagName, value.As<ushort>());
                    break;
                case PlcValueType.Udint:
                    WriteUdint(tagName, value.As<uint>());
                    break;
                case PlcValueType.Ulint:
                    WriteUlint(tagName, value.As<ulong>());
                    break;
                case PlcValueType.Real:
                    WriteReal(tagName, value.As<float>());
                    break;
                case PlcValueType.Lreal:
                    WriteLreal(tagName, value.As<double>());
                    break;
                case PlcValueType.String:
                    WriteString(tagName, value.As<string>());
                    break;
                case PlcValueType.Udt:
                    WriteUdt(tagName, value);
                    break;
                default:
                    throw new NotSupportedException($"Writing {value.Type} is not supported");
            }
        }

        /// <summary>
        /// Reads a tag and returns detailed result including value, quality, and timestamp.
        /// </summary>
        /// <param name="tagName">Name of the tag to read</param>
        /// <returns>TagReadResult containing value, quality, and timestamp</returns>
        /// <exception cref="InvalidOperationException">Thrown if not connected to PLC</exception>
        public TagReadResult ReadTagWithDetails(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                var timestamp = DateTime.Now;
                try
                {
                    CheckConnection();
                    var value = ReadTag(tagName);
                    return new TagReadResult
                    {
                        TagName = tagName,
                        Value = value,
                        Quality = DataQuality.Good,
                        TimeStamp = timestamp,
                        Success = true
                    };
                }
                catch (Exception ex)
                {
                    return new TagReadResult
                    {
                        TagName = tagName,
                        Value = null!,
                        Quality = DataQuality.Bad,
                        TimeStamp = timestamp,
                        Success = false,
                        ErrorMessage = ex.Message
                    };
                }
            });
        }

        /// <summary>
        /// Reads multiple tags in a single batch operation.
        /// Returns an array of PlcValue objects, one for each tag.
        /// More efficient than individual ReadTag calls.
        /// </summary>
        /// <param name="tagNames">Array of tag names to read</param>
        /// <returns>Array of PlcValue objects corresponding to each tag</returns>
        /// <exception cref="ArgumentException">Thrown if tagNames array is null or empty</exception>
        /// <exception cref="InvalidOperationException">Thrown if not connected to PLC</exception>
        public PlcValue[] ReadTags(string[] tagNames)
        {
            if (tagNames == null || tagNames.Length == 0)
                throw new ArgumentException("Tag names array cannot be null or empty", nameof(tagNames));

            return ExecuteWithLock(() =>
            {
                CheckConnection();
                var results = new PlcValue[tagNames.Length];
                for (int i = 0; i < tagNames.Length; i++)
                {
                    try
                    {
                        results[i] = ReadTag(tagNames[i]);
                    }
                    catch (Exception ex)
                    {
                        throw OperationFailure($"Failed to read tag '{tagNames[i]}': {ex.Message}", ex);
                    }
                }
                return results;
            });
        }

        /// <summary>
        /// Reads multiple tags in a single batch operation with detailed results.
        /// Returns an array of TagReadResult objects, one for each tag, including quality and timestamp.
        /// </summary>
        /// <param name="tagNames">Array of tag names to read</param>
        /// <returns>Array of TagReadResult objects corresponding to each tag</returns>
        /// <exception cref="ArgumentException">Thrown if tagNames array is null or empty</exception>
        /// <exception cref="InvalidOperationException">Thrown if not connected to PLC</exception>
        public TagReadResult[] ReadTagsWithDetails(string[] tagNames)
        {
            if (tagNames == null || tagNames.Length == 0)
                throw new ArgumentException("Tag names array cannot be null or empty", nameof(tagNames));

            return ExecuteWithLock(() =>
            {
                CheckConnection();
                var timestamp = DateTime.Now;
                var results = new TagReadResult[tagNames.Length];
                
                for (int i = 0; i < tagNames.Length; i++)
                {
                    try
                    {
                        var value = ReadTag(tagNames[i]);
                        results[i] = new TagReadResult
                        {
                            TagName = tagNames[i],
                            Value = value,
                            Quality = DataQuality.Good,
                            TimeStamp = timestamp,
                            Success = true
                        };
                    }
                    catch (Exception ex)
                    {
                        results[i] = new TagReadResult
                        {
                            TagName = tagNames[i],
                            Value = null!,
                            Quality = DataQuality.Bad,
                            TimeStamp = timestamp,
                            Success = false,
                            ErrorMessage = ex.Message
                        };
                    }
                }
                
                return results;
            });
        }

        /// <summary>
        /// Reads a contiguous range of array elements from a basic-type PLC array.
        /// </summary>
        /// <param name="baseArrayName">Base array name without index (e.g., "MyArray").</param>
        /// <param name="startIndex">Starting element index.</param>
        /// <param name="elementCount">Number of elements to read.</param>
        /// <returns>List of <see cref="PlcValue"/> values in index order.</returns>
        public List<PlcValue> ReadArrayRange(string baseArrayName, int startIndex, int elementCount)
        {
            if (string.IsNullOrWhiteSpace(baseArrayName))
                throw new ArgumentException("Base array name cannot be null or empty", nameof(baseArrayName));
            if (startIndex < 0)
                throw new ArgumentOutOfRangeException(nameof(startIndex), "Start index must be non-negative");
            if (elementCount <= 0)
                throw new ArgumentOutOfRangeException(nameof(elementCount), "Element count must be greater than zero");

            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr namePtr = Marshal.StringToHGlobalAnsi(baseArrayName);
                IntPtr resultPtr = Marshal.AllocHGlobal(65536);
                try
                {
                    int result = eip_read_array_range(_clientId, namePtr, startIndex, elementCount, resultPtr, 65536);
                    if (result != 0)
                        throw OperationFailure($"Failed to read array range '{baseArrayName}[{startIndex}..{startIndex + elementCount - 1}]'.");

                    string jsonResult = Marshal.PtrToStringAnsi(resultPtr) ?? string.Empty;
                    if (string.IsNullOrWhiteSpace(jsonResult))
                        throw new Exception("Array range response was empty.");

                    var values = new List<PlcValue>();
                    using var doc = JsonDocument.Parse(jsonResult);
                    if (doc.RootElement.ValueKind != JsonValueKind.Array)
                        throw new Exception("Array range response was not a JSON array.");

                    foreach (var item in doc.RootElement.EnumerateArray())
                    {
                        values.Add(PlcValue.FromJson(item.GetRawText()));
                    }

                    if (values.Count != elementCount)
                        throw new Exception($"Array range size mismatch. Requested {elementCount}, got {values.Count}.");

                    return values;
                }
                finally
                {
                    Marshal.FreeHGlobal(resultPtr);
                    Marshal.FreeHGlobal(namePtr);
                }
            });
        }

        /// <summary>
        /// Reads a DINT array range and converts values to <see cref="int"/>.
        /// </summary>
        public int[] ReadDintArrayRange(string baseArrayName, int startIndex, int elementCount)
        {
            var values = ReadArrayRange(baseArrayName, startIndex, elementCount);
            return values.Select(v => v.As<int>()).ToArray();
        }

        /// <summary>
        /// Reads a REAL array range and converts values to <see cref="float"/>.
        /// </summary>
        public float[] ReadRealArrayRange(string baseArrayName, int startIndex, int elementCount)
        {
            var values = ReadArrayRange(baseArrayName, startIndex, elementCount);
            return values.Select(v => v.As<float>()).ToArray();
        }

        /// <summary>
        /// Writes multiple tags in a single batch operation.
        /// Returns an array of success flags, one for each tag.
        /// More efficient than individual WriteTag calls.
        /// </summary>
        /// <param name="tagNames">Array of tag names to write</param>
        /// <param name="values">Array of PlcValue objects to write (must match tagNames length)</param>
        /// <returns>Array of boolean success flags, one for each tag</returns>
        /// <exception cref="ArgumentException">Thrown if arrays are null, empty, or mismatched lengths</exception>
        /// <exception cref="InvalidOperationException">Thrown if not connected to PLC</exception>
        public bool[] WriteTags(string[] tagNames, PlcValue[] values)
        {
            if (tagNames == null || values == null)
                throw new ArgumentNullException("Tag names and values arrays cannot be null");
            if (tagNames.Length != values.Length)
                throw new ArgumentException("Tag names and values arrays must have the same length");

            return ExecuteWithLock(() =>
            {
                CheckConnection();
                var results = new bool[tagNames.Length];
                for (int i = 0; i < tagNames.Length; i++)
                {
                    try
                    {
                        WriteTag(tagNames[i], values[i]);
                        results[i] = true;
                    }
                    catch
                    {
                        results[i] = false;
                    }
                }
                return results;
            });
        }

        #endregion

        #region Tag Management

        /// <summary>
        /// Discovers all tags in the PLC and caches their metadata.
        /// </summary>
        public void DiscoverTags()
        {
            ExecuteWithLock(() =>
            {
                CheckConnection();
                int result = eip_discover_tags(_clientId);
                if (result != 0)
                    throw new Exception("Failed to discover tags from PLC.");
            });
        }

        /// <summary>
        /// Gets metadata for a specific tag.
        /// </summary>
        /// <param name="tagName">Name of the tag to get metadata for.</param>
        /// <returns>Tag metadata including data type, scope, and array information.</returns>
        public TagMetadata GetTagMetadata(string tagName)
        {
            return ExecuteWithLock(() =>
            {
                CheckConnection();
                IntPtr tagPtr = Marshal.StringToHGlobalAnsi(tagName);
                try
                {
                    int result = eip_get_tag_metadata(_clientId, tagPtr, out TagMetadata metadata);
                    if (result != 0)
                        throw OperationFailure($"Failed to get metadata for tag '{tagName}'. Check tag exists.");
                    return metadata;
                }
                finally
                {
                    Marshal.FreeHGlobal(tagPtr);
                }
            });
        }

        #endregion

        #region Configuration

        /// <summary>
        /// Sets the maximum packet size for communication with the PLC.
        /// </summary>
        /// <param name="size">Maximum packet size in bytes (recommended: 4000).</param>
        public void SetMaxPacketSize(int size)
        {
            ExecuteWithLock(() =>
            {
                CheckConnection();
                eip_set_max_packet_size(_clientId, size);
            });
        }

        /// <summary>
        /// Checks the health of the connection to the PLC.
        /// </summary>
        /// <returns>True if connection is healthy, false otherwise.</returns>
        public bool CheckHealth()
        {
            if (_clientId < 0) return false;
            
            int result = eip_check_health(_clientId, out int isHealthy);
            return result == 0 && isHealthy != 0;
        }

        /// <summary>
        /// Performs a detailed health check by actually communicating with the PLC.
        /// This method sends a keep-alive message to verify connectivity.
        /// </summary>
        /// <returns>True if connection is healthy, false otherwise.</returns>
        public bool CheckHealthDetailed()
        {
            if (_clientId < 0) return false;
            
            int result = eip_check_health_detailed(_clientId, out int isHealthy);
            return result == 0 && isHealthy != 0;
        }

        #endregion

    }
    // =========================================================================
    // BATCH OPERATIONS DATA STRUCTURES
    // =========================================================================
    
    /// <summary>
    /// Represents a batch operation (read or write) to be executed.
    /// </summary>
    public class BatchOperation(
        string tagName = "",
        bool isWrite = false,
        object? value = null)
    {
        /// <summary>
        /// Name of the PLC tag to operate on.
        /// </summary>
        public string TagName { get; set; } = tagName;
        
        /// <summary>
        /// True for write operations, false for read operations.
        /// </summary>
        public bool IsWrite { get; set; } = isWrite;
        
        /// <summary>
        /// Value to write (only used for write operations).
        /// </summary>
        public object? Value { get; set; } = value;
        
        /// <summary>
        /// Creates a read operation for the specified tag.
        /// </summary>
        /// <param name="tagName">Name of the tag to read</param>
        /// <returns>Read batch operation</returns>
        public static BatchOperation Read(string tagName)
        {
            return new BatchOperation(tagName, false, null);
        }
        
        /// <summary>
        /// Creates a write operation for the specified tag and value.
        /// </summary>
        /// <param name="tagName">Name of the tag to write</param>
        /// <param name="value">Value to write to the tag</param>
        /// <returns>Write batch operation</returns>
        public static BatchOperation Write(string tagName, object value)
        {
            return new BatchOperation(tagName, true, value);
        }
    }
    
    /// <summary>
    /// Result of a batch operation execution.
    /// </summary>
    public class BatchOperationResult(
        string tagName = "",
        bool isWrite = false,
        bool success = false,
        object? value = null,
        double executionTimeMs = 0.0,
        int errorCode = 0,
        string? errorMessage = null)
    {
        /// <summary>
        /// Name of the tag that was operated on.
        /// </summary>
        public string TagName { get; set; } = tagName;
        
        /// <summary>
        /// True if this was a write operation, false for read.
        /// </summary>
        public bool IsWrite { get; set; } = isWrite;
        
        /// <summary>
        /// True if the operation completed successfully.
        /// </summary>
        public bool Success { get; set; } = success;
        
        /// <summary>
        /// Value read from the tag (only for successful read operations).
        /// </summary>
        public object? Value { get; set; } = value;
        
        /// <summary>
        /// Execution time for this operation in milliseconds.
        /// </summary>
        public double ExecutionTimeMs { get; set; } = executionTimeMs;
        
        /// <summary>
        /// Error code (0 for success, negative for errors).
        /// </summary>
        public int ErrorCode { get; set; } = errorCode;
        
        /// <summary>
        /// Error message (null for successful operations).
        /// </summary>
        public string? ErrorMessage { get; set; } = errorMessage;
    }
    
    /// <summary>
    /// Result of a tag read operation in a batch (legacy format for batch operations).
    /// Note: For detailed read results with quality and timestamp, use the TagReadResult class from TagReadResult.cs
    /// </summary>
    public class TagReadResultBatch(
        string tagName = "",
        bool success = false,
        object? value = null,
        string dataType = "",
        int errorCode = 0,
        string? errorMessage = null)
    {
        /// <summary>
        /// Name of the tag that was read.
        /// </summary>
        public string TagName { get; set; } = tagName;
        
        /// <summary>
        /// True if the read was successful.
        /// </summary>
        public bool Success { get; set; } = success;
        
        /// <summary>
        /// Value read from the tag (null if read failed).
        /// </summary>
        public object? Value { get; set; } = value;
        
        /// <summary>
        /// Data type of the tag (e.g., "DINT", "REAL", "BOOL").
        /// </summary>
        public string DataType { get; set; } = dataType;
        
        /// <summary>
        /// Error code (0 for success, negative for errors).
        /// </summary>
        public int ErrorCode { get; set; } = errorCode;
        
        /// <summary>
        /// Error message (null for successful reads).
        /// </summary>
        public string? ErrorMessage { get; set; } = errorMessage;
    }
    
    /// <summary>
    /// Result of a tag write operation in a batch.
    /// </summary>
    public class TagWriteResult(
        string tagName = "",
        bool success = false,
        int errorCode = 0,
        string? errorMessage = null)
    {
        /// <summary>
        /// Name of the tag that was written.
        /// </summary>
        public string TagName { get; set; } = tagName;
        
        /// <summary>
        /// True if the write was successful.
        /// </summary>
        public bool Success { get; set; } = success;
        
        /// <summary>
        /// Error code (0 for success, negative for errors).
        /// </summary>
        public int ErrorCode { get; set; } = errorCode;
        
        /// <summary>
        /// Error message (null for successful writes).
        /// </summary>
        public string? ErrorMessage { get; set; } = errorMessage;
    }
    
    /// <summary>
    /// Configuration settings for batch operations.
    /// </summary>
    public class BatchConfig
    {
        /// <summary>
        /// Maximum number of operations to include in a single CIP packet.
        /// Larger values improve performance but may exceed PLC packet size limits.
        /// Typical range: 10-50 operations per packet.
        /// </summary>
        public int MaxOperationsPerPacket { get; set; } = 20;
        
        /// <summary>
        /// Maximum packet size in bytes for batch operations.
        /// Should not exceed the PLC's maximum packet size capability.
        /// Typical values: 504 bytes (default), up to 4000 bytes for modern PLCs.
        /// </summary>
        public int MaxPacketSize { get; set; } = 504;
        
        /// <summary>
        /// Timeout for individual batch packets (in milliseconds).
        /// This is per-packet timeout, not per-operation.
        /// Typical range: 1000-5000 milliseconds.
        /// </summary>
        public long PacketTimeoutMs { get; set; } = 3000;
        
        /// <summary>
        /// Whether to continue processing other operations if one fails.
        /// If true, failed operations are reported but don't stop the batch.
        /// If false, the first error stops the entire batch processing.
        /// </summary>
        public bool ContinueOnError { get; set; } = true;
        
        /// <summary>
        /// Whether to optimize packet packing by grouping similar operations.
        /// If true, reads and writes are grouped separately for better performance.
        /// If false, operations are processed in the order provided.
        /// </summary>
        public bool OptimizePacketPacking { get; set; } = true;
        
        /// <summary>
        /// Creates a default batch configuration optimized for typical usage.
        /// </summary>
        /// <returns>Default batch configuration</returns>
        public static BatchConfig Default()
        {
            return new BatchConfig();
        }
        
        /// <summary>
        /// Creates a high-performance batch configuration for modern PLCs.
        /// </summary>
        /// <returns>High-performance batch configuration</returns>
        public static BatchConfig HighPerformance()
        {
            return new BatchConfig
            {
                MaxOperationsPerPacket = 50,
                MaxPacketSize = 4000,
                PacketTimeoutMs = 1000,
                ContinueOnError = true,
                OptimizePacketPacking = true
            };
        }
        
        /// <summary>
        /// Creates a conservative batch configuration for older PLCs or unreliable networks.
        /// </summary>
        /// <returns>Conservative batch configuration</returns>
        public static BatchConfig Conservative()
        {
            return new BatchConfig
            {
                MaxOperationsPerPacket = 10,
                MaxPacketSize = 504,
                PacketTimeoutMs = 5000,
                ContinueOnError = false,
                OptimizePacketPacking = false
            };
        }
    }
    
    // Native structures for FFI (placeholder for future implementation)
    [StructLayout(LayoutKind.Sequential)]
    internal struct UdtMemberNative
    {
        public IntPtr Name;
        public short DataType;
        public int Offset;
        public int Size;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct UdtDefinitionResultNative
    {
        [MarshalAs(UnmanagedType.I1)]
        public bool Success;
        public IntPtr ErrorMessage;
        public IntPtr Name;
        public IntPtr Members;
        public int MemberCount;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct TagAttributesNative
    {
        public IntPtr Name;
        public IntPtr DataTypeName;
        public short DataType;
        public int Size;
        public int TemplateInstanceId;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct TagAttributesResultNative
    {
        [MarshalAs(UnmanagedType.I1)]
        public bool Success;
        public IntPtr ErrorMessage;
        public IntPtr Name;
        public IntPtr DataTypeName;
        public short DataType;
        public int Size;
        public int TemplateInstanceId;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct TagDiscoveryResultNative
    {
        [MarshalAs(UnmanagedType.I1)]
        public bool Success;
        public IntPtr ErrorMessage;
        public IntPtr Tags;
        public int TagCount;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct BatchConfigNative
    {
        public int MaxOperationsPerPacket;
        public int MaxPacketSize;
        public long PacketTimeoutMs;
        public int ContinueOnError;
        public int OptimizePacketPacking;
    }
    
    [StructLayout(LayoutKind.Sequential)]
    internal struct TagReadResultNative
    {
        public IntPtr TagName;
        public int Success;
        public int ErrorCode;
        public IntPtr ErrorMessage;
        public int DataType;
        // Value fields (union-like)
        public int ValueBool;
        public int ValueDint;
        public float ValueReal;
        public IntPtr ValueString;
    }
    
    [StructLayout(LayoutKind.Sequential)]
    internal struct TagWriteValueNative
    {
        public IntPtr TagName;
        public int DataType;
        // Value fields (union-like)
        public int ValueBool;
        public int ValueDint;
        public float ValueReal;
        public IntPtr ValueString;
    }
    
    [StructLayout(LayoutKind.Sequential)]
    internal struct TagWriteResultNative
    {
        public IntPtr TagName;
        public int Success;
        public int ErrorCode;
        public IntPtr ErrorMessage;
    }
    
    [StructLayout(LayoutKind.Sequential)]
    internal struct BatchOperationNative
    {
        public IntPtr TagName;
        public int IsWrite;
        public int DataType;
        // Value fields (union-like)
        public int ValueBool;
        public int ValueDint;
        public float ValueReal;
        public IntPtr ValueString;
    }
    
    [StructLayout(LayoutKind.Sequential)]
    internal struct BatchOperationResultNative
    {
        public IntPtr TagName;
        public int IsWrite;
        public int Success;
        public long ExecutionTimeUs;
        public int ErrorCode;
        public IntPtr ErrorMessage;
        public int DataType;
        // Value fields (union-like)
        public int ValueBool;
        public int ValueDint;
        public float ValueReal;
        public IntPtr ValueString;
    }
}
