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
            try
            {
                var request = new List<FfiWriteBatchRequestItem>
                {
                    new()
                    {
                        tag_name = tagName,
                        value_type = ToRustValueType(value),
                        value = ToJsonElement(ToRustRawValue(value))
                    }
                };

                string payload = JsonSerializer.Serialize(request);
                IntPtr payloadPtr = Marshal.StringToHGlobalAnsi(payload);
                IntPtr resultPtr = IntPtr.Zero;

                try
                {
                    resultPtr = Marshal.AllocHGlobal(65536);
                    int rc = eip_write_tags_batch(_clientId, payloadPtr, 1, resultPtr, 65536);
                    string json = Marshal.PtrToStringAnsi(resultPtr) ?? string.Empty;
                    if (!string.IsNullOrWhiteSpace(json))
                    {
                        var nativeResults = JsonSerializer.Deserialize<List<FfiWriteBatchResultItem>>(json) ?? new();
                        var nativeResult = nativeResults.FirstOrDefault(r => r.tag_name == tagName);
                        if (nativeResult != null)
                        {
                            if (!string.IsNullOrWhiteSpace(nativeResult.error))
                                throw new Exception(nativeResult.error);

                            if (!nativeResult.success)
                                throw new Exception(InferKnownWriteLimitation(tagName, value, fallbackMessage));

                            if (rc != 0)
                                throw new Exception(fallbackMessage);
                        }
                    }
                }
                finally
                {
                    Marshal.FreeHGlobal(payloadPtr);
                    if (resultPtr != IntPtr.Zero)
                        Marshal.FreeHGlobal(resultPtr);
                }
            }
            catch (Exception ex) when (!string.IsNullOrWhiteSpace(ex.Message))
            {
                throw new Exception(ex.Message);
            }

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
            if (!_isDisposed)
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
                if (_clientId >= 0)
                {
                    eip_disconnect(_clientId);
                    _clientId = -1;
                }
                _operationLock.Dispose();
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
