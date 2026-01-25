using System;

namespace RustEtherNetIp
{
    /// <summary>
    /// Represents the result of reading a tag, including value, quality, and timestamp.
    /// </summary>
    public class TagReadResult(
        string tagName = "",
        PlcValue? value = null,
        DataQuality quality = DataQuality.Good,
        DateTime? timeStamp = null,
        bool success = true,
        string? errorMessage = null)
    {
        /// <summary>
        /// Gets the tag name that was read.
        /// </summary>
        public string TagName { get; set; } = tagName;

        /// <summary>
        /// Gets the value read from the PLC.
        /// </summary>
        public PlcValue? Value { get; set; } = value;

        /// <summary>
        /// Gets the quality of the read operation.
        /// </summary>
        public DataQuality Quality { get; set; } = quality;

        /// <summary>
        /// Gets the timestamp when the tag was read.
        /// </summary>
        public DateTime TimeStamp { get; set; } = timeStamp ?? DateTime.Now;

        /// <summary>
        /// Gets whether the read operation was successful.
        /// </summary>
        public bool Success { get; set; } = success;

        /// <summary>
        /// Gets the error message if the read failed.
        /// </summary>
        public string? ErrorMessage { get; set; } = errorMessage;
    }
}

