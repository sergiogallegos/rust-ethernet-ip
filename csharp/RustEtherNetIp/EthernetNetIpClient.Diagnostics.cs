using System;
using System.Runtime.InteropServices;
using System.Text.Json;

namespace RustEtherNetIp
{
    public partial class EtherNetIpClient
    {
        public DiagnosticsSnapshot GetDiagnosticsSnapshot()
        {
            return GetDiagnosticsSnapshotInternal(detailed: false);
        }

        public DiagnosticsSnapshot GetDiagnosticsSnapshotDetailed()
        {
            return GetDiagnosticsSnapshotInternal(detailed: true);
        }

        /// <summary>
        /// Invalidates all controller-schema-derived caches without reconnecting.
        /// Pause writes before an edit or download, call this after it completes,
        /// optionally rediscover and verify critical tags, then resume writes.
        /// </summary>
        public void RefreshSchema()
        {
            if (_clientId < 0)
                throw new InvalidOperationException("Not connected to a PLC");

            ExecuteWithLock(() =>
            {
                int rc = eip_refresh_schema(_clientId);
                if (rc != 0)
                    throw OperationFailure($"Failed to refresh controller schema (native code {rc})");
                return true;
            });
        }

        private DiagnosticsSnapshot GetDiagnosticsSnapshotInternal(bool detailed)
        {
            if (_clientId < 0)
                throw new InvalidOperationException("Not connected to a PLC");

            return ExecuteWithLock(() =>
            {
                IntPtr resultPtr = IntPtr.Zero;
                try
                {
                    int rc = eip_get_diagnostics_json(_clientId, detailed ? 1 : 0, out resultPtr);
                    if (rc != 0 || resultPtr == IntPtr.Zero)
                        throw new Exception("Failed to retrieve diagnostics snapshot");

                    string json = Marshal.PtrToStringUTF8(resultPtr) ?? string.Empty;
                    if (string.IsNullOrWhiteSpace(json))
                        throw new Exception("Native diagnostics snapshot was empty");

                    return JsonSerializer.Deserialize<DiagnosticsSnapshot>(json) ?? new DiagnosticsSnapshot();
                }
                catch (JsonException ex)
                {
                    throw new Exception($"Failed to parse diagnostics snapshot JSON: {ex.Message}", ex);
                }
                finally
                {
                    if (resultPtr != IntPtr.Zero)
                        eip_free_string(resultPtr);
                }
            });
        }
    }
}
