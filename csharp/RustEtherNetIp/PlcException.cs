using System;

namespace RustEtherNetIp
{
    /// <summary>
    /// Raised when a PLC operation fails. When available, <see cref="NativeError"/>
    /// carries the human-readable reason reported by the native library
    /// (retrieved via <c>eip_get_last_error</c>), which is also appended to
    /// <see cref="Exception.Message"/>.
    /// </summary>
    public class PlcException : Exception
    {
        /// <summary>
        /// The underlying error message reported by the native library, if any.
        /// </summary>
        public string? NativeError { get; }

        public PlcException(string message, string? nativeError = null)
            : base(Compose(message, nativeError))
        {
            NativeError = nativeError;
        }

        public PlcException(string message, string? nativeError, Exception innerException)
            : base(Compose(message, nativeError), innerException)
        {
            NativeError = nativeError;
        }

        private static string Compose(string message, string? nativeError)
            => string.IsNullOrWhiteSpace(nativeError) ? message : $"{message} (native: {nativeError})";
    }
}
