use crate::error::{EtherNetIpError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Production configuration for EtherNet/IP library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionConfig {
    /// Connection settings
    pub connection: ConnectionConfig,
    /// Performance settings
    pub performance: PerformanceConfig,
    /// Monitoring settings
    pub monitoring: MonitoringConfig,
    /// Security settings
    pub security: SecurityConfig,
    /// Logging settings
    pub logging: LoggingConfig,
    /// PLC-specific settings
    pub plc_settings: HashMap<String, PlcSpecificConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Default connection timeout
    pub connection_timeout: Duration,
    /// Default read timeout
    pub read_timeout: Duration,
    /// Default write timeout
    pub write_timeout: Duration,
    /// Maximum number of concurrent connections
    pub max_connections: u32,
    /// Connection retry attempts
    pub retry_attempts: u32,
    /// Retry delay between attempts
    pub retry_delay: Duration,
    /// Keep-alive interval
    pub keep_alive_interval: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum packet size
    pub max_packet_size: usize,
    /// Batch operation configuration
    pub batch_config: BatchConfig,
    /// Connection pool settings
    pub connection_pool: ConnectionPoolConfig,
    /// Memory limits
    pub memory_limits: MemoryLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Maximum operations per batch
    pub max_operations_per_batch: usize,
    /// Batch timeout
    pub batch_timeout: Duration,
    /// Continue on error
    pub continue_on_error: bool,
    /// Optimize packet packing
    pub optimize_packet_packing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Initial pool size
    pub initial_size: u32,
    /// Maximum pool size
    pub max_size: u32,
    /// Pool growth increment
    pub growth_increment: u32,
    /// Connection idle timeout
    pub idle_timeout: Duration,
    /// Pool cleanup interval
    pub cleanup_interval: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
    /// Memory warning threshold in MB
    pub warning_threshold_mb: usize,
    /// Enable memory monitoring
    pub enable_monitoring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable production monitoring
    pub enabled: bool,
    /// Metrics collection interval
    pub collection_interval: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Metrics retention period
    pub retention_period: Duration,
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Error rate threshold (0.0 to 1.0)
    pub error_rate_threshold: f64,
    /// Latency threshold in milliseconds
    pub latency_threshold_ms: f64,
    /// Memory usage threshold in MB
    pub memory_threshold_mb: usize,
    /// Connection failure threshold
    pub connection_failure_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable connection encryption (if supported by PLC)
    pub enable_encryption: bool,
    /// Connection validation
    pub validate_connections: bool,
    /// Input validation
    pub validate_inputs: bool,
    /// Rate limiting
    pub rate_limiting: RateLimitingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Maximum requests per second
    pub max_requests_per_second: u32,
    /// Burst capacity
    pub burst_capacity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: LogLevel,
    /// Log format (json, text)
    pub format: LogFormat,
    /// Log file path
    pub file_path: Option<String>,
    /// Enable console logging
    pub enable_console: bool,
    /// Enable structured logging
    pub enable_structured: bool,
    /// Log rotation settings
    pub rotation: LogRotationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// Enable log rotation
    pub enabled: bool,
    /// Maximum file size in MB
    pub max_file_size_mb: usize,
    /// Maximum number of files
    pub max_files: usize,
    /// Rotation schedule (daily, weekly, monthly)
    pub schedule: LogRotationSchedule,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Text,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogRotationSchedule {
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlcSpecificConfig {
    /// PLC model/type
    pub model: String,
    /// Specific connection settings
    pub connection_settings: HashMap<String, String>,
    /// Tag discovery settings
    pub tag_discovery: TagDiscoveryConfig,
    /// Performance tuning
    pub performance_tuning: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDiscoveryConfig {
    /// Enable automatic tag discovery
    pub enabled: bool,
    /// Discovery interval
    pub interval: Duration,
    /// Cache discovered tags
    pub cache_tags: bool,
    /// Maximum tags to discover
    pub max_tags: usize,
}

impl Default for ProductionConfig {
    fn default() -> Self {
        Self {
            connection: ConnectionConfig {
                connection_timeout: Duration::from_secs(10),
                read_timeout: Duration::from_secs(5),
                write_timeout: Duration::from_secs(5),
                max_connections: 10,
                retry_attempts: 3,
                retry_delay: Duration::from_secs(1),
                keep_alive_interval: Duration::from_secs(30),
            },
            performance: PerformanceConfig {
                max_packet_size: 4000,
                batch_config: BatchConfig {
                    max_operations_per_batch: 50,
                    batch_timeout: Duration::from_secs(10),
                    continue_on_error: true,
                    optimize_packet_packing: true,
                },
                connection_pool: ConnectionPoolConfig {
                    initial_size: 2,
                    max_size: 10,
                    growth_increment: 2,
                    idle_timeout: Duration::from_secs(300),
                    cleanup_interval: Duration::from_secs(60),
                },
                memory_limits: MemoryLimits {
                    max_memory_mb: 100,
                    warning_threshold_mb: 80,
                    enable_monitoring: true,
                },
            },
            monitoring: MonitoringConfig {
                enabled: true,
                collection_interval: Duration::from_secs(30),
                health_check_interval: Duration::from_secs(60),
                retention_period: Duration::from_secs(86400), // 24 hours
                enable_profiling: false,
                alert_thresholds: AlertThresholds {
                    error_rate_threshold: 0.05,
                    latency_threshold_ms: 1000.0,
                    memory_threshold_mb: 80,
                    connection_failure_threshold: 5,
                },
            },
            security: SecurityConfig {
                enable_encryption: false,
                validate_connections: true,
                validate_inputs: true,
                rate_limiting: RateLimitingConfig {
                    enabled: true,
                    max_requests_per_second: 100,
                    burst_capacity: 200,
                },
            },
            logging: LoggingConfig {
                level: LogLevel::Info,
                format: LogFormat::Json,
                file_path: Some("logs/ethernet_ip.log".to_string()),
                enable_console: true,
                enable_structured: true,
                rotation: LogRotationConfig {
                    enabled: true,
                    max_file_size_mb: 100,
                    max_files: 10,
                    schedule: LogRotationSchedule::Daily,
                },
            },
            plc_settings: HashMap::new(),
        }
    }
}

impl ProductionConfig {
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: ProductionConfig =
            toml::from_str(&content).map_err(|e| EtherNetIpError::Other(e.to_string()))?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content =
            toml::to_string_pretty(self).map_err(|e| EtherNetIpError::Other(e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> std::result::Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate connection settings
        if self.connection.connection_timeout.as_secs() == 0 {
            errors.push("Connection timeout must be greater than 0".to_string());
        }

        if self.connection.max_connections == 0 {
            errors.push("Maximum connections must be greater than 0".to_string());
        }

        // Validate performance settings
        if self.performance.max_packet_size < 100 {
            errors.push("Maximum packet size must be at least 100 bytes".to_string());
        }

        if self.performance.batch_config.max_operations_per_batch == 0 {
            errors.push("Maximum operations per batch must be greater than 0".to_string());
        }

        // Validate monitoring settings
        if self.monitoring.collection_interval.as_secs() == 0 {
            errors.push("Collection interval must be greater than 0".to_string());
        }

        // Validate security settings
        if self.security.rate_limiting.enabled
            && self.security.rate_limiting.max_requests_per_second == 0
        {
            errors.push(
                "Max requests per second must be greater than 0 when rate limiting is enabled"
                    .to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get PLC-specific configuration
    pub fn get_plc_config(&self, plc_address: &str) -> Option<&PlcSpecificConfig> {
        self.plc_settings.get(plc_address)
    }

    /// Add or update PLC-specific configuration
    pub fn set_plc_config(&mut self, plc_address: String, config: PlcSpecificConfig) {
        self.plc_settings.insert(plc_address, config);
    }

    /// Create a development configuration
    pub fn development() -> Self {
        let mut config = Self::default();
        config.logging.level = LogLevel::Debug;
        config.monitoring.enabled = false;
        config.security.rate_limiting.enabled = false;
        config.performance.memory_limits.enable_monitoring = false;
        config
    }

    /// Create a production configuration
    pub fn production() -> Self {
        let mut config = Self::default();
        config.logging.level = LogLevel::Info;
        config.monitoring.enabled = true;
        config.security.rate_limiting.enabled = true;
        config.performance.memory_limits.enable_monitoring = true;
        config.performance.memory_limits.max_memory_mb = 500;
        config
    }
}
