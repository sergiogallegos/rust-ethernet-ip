using System;

namespace RustEtherNetIp
{
    /// <summary>
    /// Represents the result of reading a tag, including value, quality, and timestamp.
    /// </summary>
    public class TagReadResult
    {
        /// <summary>
        /// Gets the tag name that was read.
        /// </summary>
        public string TagName { get; set; } = string.Empty;

        /// <summary>
        /// Gets the value read from the PLC.
        /// </summary>
        public PlcValue? Value { get; set; }

        /// <summary>
        /// Gets the quality of the read operation.
        /// </summary>
        public DataQuality Quality { get; set; } = DataQuality.Good;

        /// <summary>
        /// Gets the timestamp when the tag was read.
        /// </summary>
        public DateTime TimeStamp { get; set; } = DateTime.Now;

        /// <summary>
        /// Gets whether the read operation was successful.
        /// </summary>
        public bool Success { get; set; } = true;

        /// <summary>
        /// Gets the error message if the read failed.
        /// </summary>
        public string? ErrorMessage { get; set; }
    }
}

