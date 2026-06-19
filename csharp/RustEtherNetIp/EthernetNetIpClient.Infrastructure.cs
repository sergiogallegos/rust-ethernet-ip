using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text.Json;
using System.Threading;

namespace RustEtherNetIp
{
    public partial class EtherNetIpClient
    {
        private void ThrowDetailedWriteException(string tagName, PlcValue value, string fallbackMessage)
        {
            // The scalar write that just failed must NOT be re-issued to the PLC
            // merely to obtain a richer error message: for side-effecting tags
            // (counters, momentary bits, event triggers) that would double-apply
            // the operation, and on a transiently-writable tag it could even
            // mask the original failure. The scalar write FFI only returns a
            // status code, so surface the best heuristic explanation we have.
            // (Richer native error propagation is tracked for a later change.)
            throw new Exception(InferKnownWriteLimitation(tagName, value, fallbackMessage));
        }

        private static string InferKnownWriteLimitation(string tagName, PlcValue value, string fallbackMessage)
        {
            if (LooksLikeDirectUdtArrayMemberPath(tagName))
            {
                return $"Direct writes to UDT array element members are not supported by this PLC/firmware path (commonly surfaced as CIP extended error 0x2107): '{tagName}'. Write the whole UDT element instead.";
            }

            if (value.Type == PlcValueType.String || LooksLikeStringMemberPath(tagName))
            {
                return $"Direct STRING writes are not supported on this PLC/firmware path for '{tagName}' (commonly surfaced as embedded service error 0x1E or extended error 0x2107).";
            }

            return fallbackMessage;
        }

        private static bool LooksLikeDirectUdtArrayMemberPath(string tagName)
        {
            if (string.IsNullOrWhiteSpace(tagName))
                return false;

            int dotIndex = tagName.LastIndexOf('.');
            int bracketStart = tagName.LastIndexOf('[');
            int bracketEnd = tagName.LastIndexOf(']');
            return dotIndex > bracketEnd && bracketStart >= 0 && bracketEnd > bracketStart;
        }

        private static bool LooksLikeStringMemberPath(string tagName)
        {
            return !string.IsNullOrWhiteSpace(tagName) &&
                   (tagName.Contains("STRING", StringComparison.OrdinalIgnoreCase) ||
                    tagName.Contains("String", StringComparison.Ordinal));
        }

        private void CheckConnection()
        {
            if (_clientId < 0)
                throw new InvalidOperationException("Not connected to PLC. Call Connect() first.");
        }

        private T ExecuteWithLock<T>(Func<T> operation)
        {
            _operationLock.Wait();
            try
            {
                if (_isDisposed)
                    throw new ObjectDisposedException(nameof(EtherNetIpClient));

                if (_clientId < 0)
                    throw new InvalidOperationException("Not connected to a PLC");

                return operation();
            }
            finally
            {
                _operationLock.Release();
            }
        }

        private void ExecuteWithLock(Action operation)
        {
            ExecuteWithLock(() =>
            {
                operation();
                return true;
            });
        }

        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        // Finalizer: if Dispose() was never called, still release the native
        // session so the Rust-side client (and its TCP socket) is not leaked for
        // the process lifetime. Only native state is touched here.
        ~EtherNetIpClient()
        {
            Dispose(false);
        }

        private void Dispose(bool disposing)
        {
            lock (_lock)
            {
                if (_isDisposed)
                    return;

                if (disposing)
                {
                    // Managed cleanup must run before _isDisposed is set: several
                    // teardown helpers (e.g. UnsubscribeFromAllTags) short-circuit
                    // once the client is marked disposed.
                    try
                    {
                        UnsubscribeFromAllTags();
                        lock (_tagGroupLock)
                        {
                            foreach (var group in _tagGroups.Values)
                            {
                                group.Group?.Dispose();
                            }
                            _tagGroups.Clear();
                        }
                        StopKeepAlive();
                        _keepAliveCts.Dispose();
                        _operationLock.Dispose();
                    }
                    catch
                    {
                        // Best-effort managed teardown; never throw from Dispose.
                    }
                }

                // Native cleanup always runs (both Dispose and finalizer).
                if (_clientId >= 0)
                {
                    eip_disconnect(_clientId);
                    _clientId = -1;
                }

                _isDisposed = true;
            }
        }

        private sealed class FfiWriteBatchRequestItem
        {
            public string tag_name { get; set; } = string.Empty;
            public string value_type { get; set; } = string.Empty;
            public JsonElement value { get; set; }
        }

        private sealed class FfiReadBatchResultItem
        {
            public string tag_name { get; set; } = string.Empty;
            public bool success { get; set; }
            public JsonElement? value { get; set; }
            public string? error { get; set; }
        }

        private sealed class FfiWriteBatchResultItem
        {
            public string tag_name { get; set; } = string.Empty;
            public bool success { get; set; }
            public string? error { get; set; }
        }

        private sealed class FfiExecuteBatchRequestItem
        {
            public string tag_name { get; set; } = string.Empty;
            public bool is_write { get; set; }
            public string? value_type { get; set; }
            public JsonElement? value { get; set; }
        }

        private sealed class FfiExecuteBatchResultItem
        {
            public int index { get; set; }
            public string tag_name { get; set; } = string.Empty;
            public bool is_write { get; set; }
            public bool success { get; set; }
            public JsonElement? value { get; set; }
            public string? error { get; set; }
            public ulong execution_time_us { get; set; }
        }
    }
}
