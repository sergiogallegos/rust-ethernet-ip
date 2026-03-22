using System;
using System.Collections.Generic;
using System.Threading.Tasks;

namespace RustEtherNetIp
{
    /// <summary>
    /// High-level classification for tag-group polling events.
    /// </summary>
    public enum TagGroupEventKind
    {
        Data,
        PartialError,
        ReadFailure
    }

    /// <summary>
    /// Structured category for tag-group read failures.
    /// </summary>
    public enum TagGroupFailureCategory
    {
        Network,
        Timeout,
        PlcStatus,
        Protocol,
        Permission,
        Tag,
        Data,
        Other
    }

    /// <summary>
    /// Structured diagnostics for read failures during tag-group polling.
    /// </summary>
    public sealed class TagGroupFailureDiagnostic
    {
        public TagGroupFailureCategory Category { get; init; } = TagGroupFailureCategory.Other;
        public bool Retriable { get; init; }
        public int? StatusCode { get; init; }

        public static TagGroupFailureDiagnostic FromException(Exception exception)
        {
            if (exception is TimeoutException
                || exception is OperationCanceledException
                || exception is TaskCanceledException)
            {
                return new TagGroupFailureDiagnostic
                {
                    Category = TagGroupFailureCategory.Timeout,
                    Retriable = true
                };
            }

            if (exception is UnauthorizedAccessException)
            {
                return new TagGroupFailureDiagnostic
                {
                    Category = TagGroupFailureCategory.Permission,
                    Retriable = false
                };
            }

            if (exception is ArgumentException
                || exception is FormatException
                || exception is InvalidCastException)
            {
                return new TagGroupFailureDiagnostic
                {
                    Category = TagGroupFailureCategory.Data,
                    Retriable = false
                };
            }

            if (exception is InvalidOperationException invalidOp
                && invalidOp.Message.Contains("Not connected", StringComparison.OrdinalIgnoreCase))
            {
                return new TagGroupFailureDiagnostic
                {
                    Category = TagGroupFailureCategory.Network,
                    Retriable = true
                };
            }

            return new TagGroupFailureDiagnostic
            {
                Category = TagGroupFailureCategory.Other,
                Retriable = false
            };
        }
    }

    /// <summary>
    /// Event payload for tag-group polling with classification and diagnostics.
    /// </summary>
    public sealed class TagGroupPollingEventArgs : EventArgs
    {
        public TagGroupEventKind Kind { get; init; }
        public string[] ChangedTags { get; init; } = Array.Empty<string>();
        public Dictionary<string, PlcValue> AllValues { get; init; } = new();
        public Dictionary<string, string> Errors { get; init; } = new();
        public string? ErrorMessage { get; init; }
        public TagGroupFailureDiagnostic? Failure { get; init; }
    }
}
