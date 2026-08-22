using System.Text.Json.Serialization;

namespace RustEtherNetIp
{
    [JsonConverter(typeof(JsonStringEnumConverter))]
    public enum DiagnosticsHealthStatus
    {
        Healthy,
        Warning,
        Critical,
        Unknown
    }

    [JsonConverter(typeof(JsonStringEnumConverter))]
    public enum DiagnosticsHealthMode
    {
        Passive,
        Verified
    }

    [JsonConverter(typeof(JsonStringEnumConverter))]
    public enum DiagnosticsErrorCategory
    {
        Network,
        Timeout,
        Session,
        RoutePath,
        CipProtocol,
        BatchEmbeddedService,
        KnownControllerLimitation,
        DataType,
        NotFound,
        Unknown
    }

    public sealed class DiagnosticsSnapshot
    {
        [JsonPropertyName("captured_at_unix_ms")]
        public long? CapturedAtUnixMs { get; set; }

        [JsonPropertyName("system_metrics_are_placeholders")]
        public bool SystemMetricsArePlaceholders { get; set; }

        [JsonPropertyName("connections")]
        public DiagnosticsConnectionMetrics Connections { get; set; } = new();

        [JsonPropertyName("operations")]
        public DiagnosticsOperationMetrics Operations { get; set; } = new();

        [JsonPropertyName("performance")]
        public DiagnosticsPerformanceMetrics Performance { get; set; } = new();

        [JsonPropertyName("errors")]
        public DiagnosticsErrorMetrics Errors { get; set; } = new();

        [JsonPropertyName("health")]
        public DiagnosticsHealthMetrics Health { get; set; } = new();

        [JsonPropertyName("schema_cache")]
        public DiagnosticsSchemaCacheMetrics SchemaCache { get; set; } = new();
    }

    public sealed class DiagnosticsSchemaCacheMetrics
    {
        [JsonPropertyName("generation")]
        public ulong Generation { get; set; }

        [JsonPropertyName("refreshes")]
        public ulong Refreshes { get; set; }

        [JsonPropertyName("array_classification_hits")]
        public ulong ArrayClassificationHits { get; set; }

        [JsonPropertyName("array_classification_misses")]
        public ulong ArrayClassificationMisses { get; set; }

        [JsonPropertyName("array_classification_evictions")]
        public ulong ArrayClassificationEvictions { get; set; }

        [JsonPropertyName("datatype_contradictions")]
        public ulong DatatypeContradictions { get; set; }

        [JsonPropertyName("successful_read_recoveries")]
        public ulong SuccessfulReadRecoveries { get; set; }

        [JsonPropertyName("failed_read_recoveries")]
        public ulong FailedReadRecoveries { get; set; }
    }

    public sealed class DiagnosticsConnectionMetrics
    {
        [JsonPropertyName("active_connections")]
        public int ActiveConnections { get; set; }

        [JsonPropertyName("total_connections")]
        public long TotalConnections { get; set; }

        [JsonPropertyName("failed_connections")]
        public long FailedConnections { get; set; }

        [JsonPropertyName("connection_uptime_avg_seconds")]
        public double ConnectionUptimeAvgSeconds { get; set; }

        [JsonPropertyName("last_connection_time_unix_ms")]
        public long? LastConnectionTimeUnixMs { get; set; }
    }

    public sealed class DiagnosticsOperationMetrics
    {
        [JsonPropertyName("total_reads")]
        public long TotalReads { get; set; }

        [JsonPropertyName("total_writes")]
        public long TotalWrites { get; set; }

        [JsonPropertyName("successful_reads")]
        public long SuccessfulReads { get; set; }

        [JsonPropertyName("successful_writes")]
        public long SuccessfulWrites { get; set; }

        [JsonPropertyName("failed_reads")]
        public long FailedReads { get; set; }

        [JsonPropertyName("failed_writes")]
        public long FailedWrites { get; set; }

        [JsonPropertyName("batch_operations")]
        public long BatchOperations { get; set; }

        [JsonPropertyName("subscription_updates")]
        public long SubscriptionUpdates { get; set; }

        [JsonPropertyName("partial_batch_failures")]
        public long PartialBatchFailures { get; set; }

        [JsonPropertyName("last_successful_read_time_unix_ms")]
        public long? LastSuccessfulReadTimeUnixMs { get; set; }

        [JsonPropertyName("last_failed_read_time_unix_ms")]
        public long? LastFailedReadTimeUnixMs { get; set; }

        [JsonPropertyName("last_successful_write_time_unix_ms")]
        public long? LastSuccessfulWriteTimeUnixMs { get; set; }

        [JsonPropertyName("last_failed_write_time_unix_ms")]
        public long? LastFailedWriteTimeUnixMs { get; set; }
    }

    public sealed class DiagnosticsPerformanceMetrics
    {
        [JsonPropertyName("avg_read_latency_ms")]
        public double AvgReadLatencyMs { get; set; }

        [JsonPropertyName("avg_write_latency_ms")]
        public double AvgWriteLatencyMs { get; set; }

        [JsonPropertyName("max_read_latency_ms")]
        public double MaxReadLatencyMs { get; set; }

        [JsonPropertyName("max_write_latency_ms")]
        public double MaxWriteLatencyMs { get; set; }

        [JsonPropertyName("reads_per_second")]
        public double ReadsPerSecond { get; set; }

        [JsonPropertyName("writes_per_second")]
        public double WritesPerSecond { get; set; }

        [JsonPropertyName("memory_usage_mb")]
        public double MemoryUsageMb { get; set; }

        [JsonPropertyName("cpu_usage_percent")]
        public double CpuUsagePercent { get; set; }
    }

    public sealed class DiagnosticsErrorMetrics
    {
        [JsonPropertyName("network_errors")]
        public long NetworkErrors { get; set; }

        [JsonPropertyName("protocol_errors")]
        public long ProtocolErrors { get; set; }

        [JsonPropertyName("timeout_errors")]
        public long TimeoutErrors { get; set; }

        [JsonPropertyName("tag_not_found_errors")]
        public long TagNotFoundErrors { get; set; }

        [JsonPropertyName("data_type_errors")]
        public long DataTypeErrors { get; set; }

        [JsonPropertyName("session_errors")]
        public long SessionErrors { get; set; }

        [JsonPropertyName("route_path_errors")]
        public long RoutePathErrors { get; set; }

        [JsonPropertyName("embedded_service_errors")]
        public long EmbeddedServiceErrors { get; set; }

        [JsonPropertyName("known_controller_limitation_errors")]
        public long KnownControllerLimitationErrors { get; set; }

        [JsonPropertyName("retriable_errors")]
        public long RetriableErrors { get; set; }

        [JsonPropertyName("non_retriable_errors")]
        public long NonRetriableErrors { get; set; }

        [JsonPropertyName("last_error_time_unix_ms")]
        public long? LastErrorTimeUnixMs { get; set; }

        [JsonPropertyName("last_error_message")]
        public string? LastErrorMessage { get; set; }

        [JsonPropertyName("last_error_category")]
        public DiagnosticsErrorCategory? LastErrorCategory { get; set; }

        [JsonPropertyName("last_retriable_error_time_unix_ms")]
        public long? LastRetriableErrorTimeUnixMs { get; set; }
    }

    public sealed class DiagnosticsHealthMetrics
    {
        [JsonPropertyName("overall_health")]
        public DiagnosticsHealthStatus OverallHealth { get; set; }

        [JsonPropertyName("health_mode")]
        public DiagnosticsHealthMode HealthMode { get; set; }

        [JsonPropertyName("last_health_check_unix_ms")]
        public long? LastHealthCheckUnixMs { get; set; }

        [JsonPropertyName("last_verified_health_check_unix_ms")]
        public long? LastVerifiedHealthCheckUnixMs { get; set; }

        [JsonPropertyName("consecutive_failures")]
        public int ConsecutiveFailures { get; set; }

        [JsonPropertyName("recovery_attempts")]
        public int RecoveryAttempts { get; set; }

        [JsonPropertyName("system_uptime_seconds")]
        public double SystemUptimeSeconds { get; set; }

        [JsonPropertyName("last_success_time_unix_ms")]
        public long? LastSuccessTimeUnixMs { get; set; }

        [JsonPropertyName("last_failure_time_unix_ms")]
        public long? LastFailureTimeUnixMs { get; set; }
    }
}
