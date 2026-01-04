using System;
using System.Runtime.InteropServices;
using System.Text;

namespace RustEtherNetIp
{
    /// <summary>
    /// Represents an Allen-Bradley Logix STRING tag structure.
    /// Logix strings are stored as structured data with two fields: LEN (DINT) and DATA (SINT array).
    /// </summary>
    /// <remarks>
    /// <para>Allen-Bradley Logix STRING tags are hybrids that can be accessed like atomic types 
    /// but are actually stored as structured data with two fields:</para>
    /// <list type="bullet">
    /// <item><description><strong>LEN:</strong> A DINT (32-bit) type describing the actual string length (not capacity)</description></item>
    /// <item><description><strong>DATA:</strong> An array of SINT (8-bit) bytes. Default is SINT[82], but custom strings may have different sizes.</description></item>
    /// </list>
    /// <para><strong>⚠️ Writing Limitation:</strong> Most PLCs do not support direct writes to STRING tags 
    /// (CIP Error 0x2107). This is a firmware restriction, not a library bug. STRING tags can be read 
    /// successfully, but writes will typically fail.</para>
    /// <para>This class follows the ASComm.NET pattern for handling Logix strings.</para>
    /// </remarks>
    [StructLayout(LayoutKind.Sequential)]
    public class LogixString
    {
        /// <summary>
        /// Logix string length field (DINT - 32-bit integer).
        /// Describes the actual string length, not the string capacity.
        /// </summary>
        public int len;

        /// <summary>
        /// Logix string data field (SINT array).
        /// Default size is 82 bytes (standard Logix STRING), but can be customized for different string sizes.
        /// </summary>
        public byte[] data;

        /// <summary>
        /// Default string capacity (82 bytes for standard Logix STRING).
        /// </summary>
        public const int DefaultCapacity = 82;

        /// <summary>
        /// Initializes a new instance of the LogixString class with default capacity (82 bytes).
        /// </summary>
        public LogixString() : this(DefaultCapacity)
        {
        }

        /// <summary>
        /// Initializes a new instance of the LogixString class with specified capacity.
        /// </summary>
        /// <param name="capacity">The maximum string capacity in bytes.</param>
        public LogixString(int capacity)
        {
            if (capacity < 0)
                throw new ArgumentOutOfRangeException(nameof(capacity), "Capacity must be non-negative");
            
            data = new byte[capacity];
            len = 0;
        }

        /// <summary>
        /// Returns the LogixString as a CLR string.
        /// </summary>
        /// <returns>The string value extracted from the LogixString structure.</returns>
        public override string ToString()
        {
            if (len <= 0 || len > data.Length)
                return string.Empty;
            
            return Encoding.ASCII.GetString(data, 0, len);
        }

        /// <summary>
        /// Stores a CLR string as a LogixString.
        /// </summary>
        /// <param name="value">The string value to store.</param>
        /// <exception cref="ArgumentOutOfRangeException">Thrown if the string length exceeds the data array capacity.</exception>
        public void SetString(string value)
        {
            if (value == null)
            {
                len = 0;
                return;
            }

            if (value.Length > data.Length)
                throw new ArgumentOutOfRangeException(nameof(value), 
                    $"Maximum allowable string length is {data.Length} bytes, but got {value.Length} bytes");

            byte[] bytes = Encoding.ASCII.GetBytes(value);
            Buffer.BlockCopy(bytes, 0, data, 0, bytes.Length);
            len = bytes.Length;
        }

        /// <summary>
        /// Gets the current string value.
        /// </summary>
        /// <returns>The string value.</returns>
        public string GetString()
        {
            return ToString();
        }

        /// <summary>
        /// Gets the maximum capacity of this LogixString.
        /// </summary>
        public int Capacity => data.Length;

        /// <summary>
        /// Gets the current string length.
        /// </summary>
        public int Length => len;
    }
}

