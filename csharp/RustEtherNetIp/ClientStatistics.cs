using System;

namespace RustEtherNetIp
{
    /// <summary>
    /// Performance statistics for EtherNet/IP client operations.
    /// Tracks read/write counts, errors, and average response times for monitoring and diagnostics.
    /// </summary>
    public class ClientStatistics
    {
        private long _readCount = 0;
        private long _writeCount = 0;
        private long _errorCount = 0;
        private long _totalResponseTimeMs = 0;
        private long _totalOperations = 0;

        /// <summary>
        /// Gets the total number of successful read operations.
        /// </summary>
        public long ReadCount => _readCount;

        /// <summary>
        /// Gets the total number of successful write operations.
        /// </summary>
        public long WriteCount => _writeCount;

        /// <summary>
        /// Gets the total number of operations that resulted in errors.
        /// </summary>
        public long ErrorCount => _errorCount;

        /// <summary>
        /// Gets the total number of operations (reads + writes).
        /// </summary>
        public long TotalOperations => _totalOperations;

        /// <summary>
        /// Gets the average response time across all operations.
        /// </summary>
        public TimeSpan AverageResponseTime
        {
            get
            {
                if (_totalOperations == 0)
                    return TimeSpan.Zero;
                return TimeSpan.FromMilliseconds(_totalResponseTimeMs / _totalOperations);
            }
        }

        internal void IncrementRead() => _readCount++;
        internal void IncrementWrite() => _writeCount++;
        internal void IncrementError() => _errorCount++;
        
        internal void AddResponseTime(long milliseconds)
        {
            _totalResponseTimeMs += milliseconds;
            _totalOperations++;
        }

        /// <summary>
        /// Resets all statistics counters to zero.
        /// </summary>
        public void Reset()
        {
            _readCount = 0;
            _writeCount = 0;
            _errorCount = 0;
            _totalResponseTimeMs = 0;
            _totalOperations = 0;
        }

        /// <summary>
        /// Returns a string representation of the statistics.
        /// </summary>
        public override string ToString()
        {
            return $"Reads: {ReadCount}, Writes: {WriteCount}, Errors: {ErrorCount}, " +
                   $"Avg Response: {AverageResponseTime.TotalMilliseconds:F2}ms";
        }
    }
}

