using System;

namespace RustEtherNetIp
{
    /// <summary>
    /// Represents the quality of a tag value read from the PLC.
    /// Similar to OPC quality indicators, used to indicate data validity and freshness.
    /// </summary>
    public enum DataQuality
    {
        /// <summary>
        /// Quality is good - data is valid and current.
        /// </summary>
        Good = 0,

        /// <summary>
        /// Quality is bad - data is invalid or could not be read.
        /// </summary>
        Bad = 1,

        /// <summary>
        /// Quality is uncertain - data may be stale or questionable.
        /// </summary>
        Uncertain = 2,

        /// <summary>
        /// Quality is not available - quality information is not provided.
        /// </summary>
        NotAvailable = 3
    }

    /// <summary>
    /// Extension methods for DataQuality enum.
    /// </summary>
    public static class DataQualityExtensions
    {
        /// <summary>
        /// Gets a human-readable string representation of the quality.
        /// </summary>
        public static string ToQualityString(this DataQuality quality)
        {
            return quality switch
            {
                DataQuality.Good => "Good",
                DataQuality.Bad => "Bad",
                DataQuality.Uncertain => "Uncertain",
                DataQuality.NotAvailable => "Not Available",
                _ => "Unknown"
            };
        }
    }
}

