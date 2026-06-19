using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

namespace RustEtherNetIp
{
    /// <summary>
    /// Asynchronous wrappers over the synchronous operations.
    /// </summary>
    /// <remarks>
    /// The native FFI is blocking, so these methods offload the blocking call to a
    /// thread-pool thread via <see cref="Task.Run(Action, CancellationToken)"/>.
    /// This keeps UI threads (WinForms/WPF) responsive and lets callers use
    /// <c>await</c>, but it does not make the underlying socket I/O non-blocking.
    /// The cancellation token prevents the work from starting if already cancelled;
    /// it cannot interrupt an in-flight native call.
    /// </remarks>
    public partial class EtherNetIpClient
    {
        // Connection
        public Task<bool> ConnectAsync(string address, CancellationToken cancellationToken = default)
            => Task.Run(() => Connect(address), cancellationToken);

        // Boolean
        public Task<bool> ReadBoolAsync(string tagName, CancellationToken cancellationToken = default)
            => Task.Run(() => ReadBool(tagName), cancellationToken);

        public Task WriteBoolAsync(string tagName, bool value, CancellationToken cancellationToken = default)
            => Task.Run(() => WriteBool(tagName, value), cancellationToken);

        // Signed integers
        public Task<sbyte> ReadSintAsync(string tagName, CancellationToken cancellationToken = default)
            => Task.Run(() => ReadSint(tagName), cancellationToken);

        public Task WriteSintAsync(string tagName, sbyte value, CancellationToken cancellationToken = default)
            => Task.Run(() => WriteSint(tagName, value), cancellationToken);

        public Task<short> ReadIntAsync(string tagName, CancellationToken cancellationToken = default)
            => Task.Run(() => ReadInt(tagName), cancellationToken);

        public Task WriteIntAsync(string tagName, short value, CancellationToken cancellationToken = default)
            => Task.Run(() => WriteInt(tagName, value), cancellationToken);

        public Task<int> ReadDintAsync(string tagName, CancellationToken cancellationToken = default)
            => Task.Run(() => ReadDint(tagName), cancellationToken);

        public Task WriteDintAsync(string tagName, int value, CancellationToken cancellationToken = default)
            => Task.Run(() => WriteDint(tagName, value), cancellationToken);

        public Task<long> ReadLintAsync(string tagName, CancellationToken cancellationToken = default)
            => Task.Run(() => ReadLint(tagName), cancellationToken);

        public Task WriteLintAsync(string tagName, long value, CancellationToken cancellationToken = default)
            => Task.Run(() => WriteLint(tagName, value), cancellationToken);

        // Unsigned integers
        public Task<byte> ReadUsintAsync(string tagName, CancellationToken cancellationToken = default)
            => Task.Run(() => ReadUsint(tagName), cancellationToken);

        public Task WriteUsintAsync(string tagName, byte value, CancellationToken cancellationToken = default)
            => Task.Run(() => WriteUsint(tagName, value), cancellationToken);

        public Task<ushort> ReadUintAsync(string tagName, CancellationToken cancellationToken = default)
            => Task.Run(() => ReadUint(tagName), cancellationToken);

        public Task WriteUintAsync(string tagName, ushort value, CancellationToken cancellationToken = default)
            => Task.Run(() => WriteUint(tagName, value), cancellationToken);

        public Task<uint> ReadUdintAsync(string tagName, CancellationToken cancellationToken = default)
            => Task.Run(() => ReadUdint(tagName), cancellationToken);

        public Task WriteUdintAsync(string tagName, uint value, CancellationToken cancellationToken = default)
            => Task.Run(() => WriteUdint(tagName, value), cancellationToken);

        public Task<ulong> ReadUlintAsync(string tagName, CancellationToken cancellationToken = default)
            => Task.Run(() => ReadUlint(tagName), cancellationToken);

        public Task WriteUlintAsync(string tagName, ulong value, CancellationToken cancellationToken = default)
            => Task.Run(() => WriteUlint(tagName, value), cancellationToken);

        // Floating point
        public Task<float> ReadRealAsync(string tagName, CancellationToken cancellationToken = default)
            => Task.Run(() => ReadReal(tagName), cancellationToken);

        public Task WriteRealAsync(string tagName, float value, CancellationToken cancellationToken = default)
            => Task.Run(() => WriteReal(tagName, value), cancellationToken);

        public Task<double> ReadLrealAsync(string tagName, CancellationToken cancellationToken = default)
            => Task.Run(() => ReadLreal(tagName), cancellationToken);

        public Task WriteLrealAsync(string tagName, double value, CancellationToken cancellationToken = default)
            => Task.Run(() => WriteLreal(tagName, value), cancellationToken);

        // String
        public Task<string> ReadStringAsync(string tagName, CancellationToken cancellationToken = default)
            => Task.Run(() => ReadString(tagName), cancellationToken);

        public Task WriteStringAsync(string tagName, string value, CancellationToken cancellationToken = default)
            => Task.Run(() => WriteString(tagName, value), cancellationToken);

        // UDT
        public Task<PlcValue> ReadUdtAsync(string tagName, CancellationToken cancellationToken = default)
            => Task.Run(() => ReadUdt(tagName), cancellationToken);

        public Task WriteUdtAsync(string tagName, PlcValue value, CancellationToken cancellationToken = default)
            => Task.Run(() => WriteUdt(tagName, value), cancellationToken);

        // Batch
        public Task<Dictionary<string, TagReadResultBatch>> ReadTagsBatchAsync(
            string[] tagNames, CancellationToken cancellationToken = default)
            => Task.Run(() => ReadTagsBatch(tagNames), cancellationToken);

        public Task<Dictionary<string, TagWriteResult>> WriteTagsBatchAsync(
            Dictionary<string, object> tagValues, CancellationToken cancellationToken = default)
            => Task.Run(() => WriteTagsBatch(tagValues), cancellationToken);

        public Task<BatchOperationResult[]> ExecuteBatchAsync(
            BatchOperation[] operations, CancellationToken cancellationToken = default)
            => Task.Run(() => ExecuteBatch(operations), cancellationToken);

        // Diagnostics
        public Task<bool> CheckHealthAsync(CancellationToken cancellationToken = default)
            => Task.Run(() => CheckHealth(), cancellationToken);
    }
}
