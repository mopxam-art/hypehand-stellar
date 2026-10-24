use std::str::FromStr;
use tracing::Level;
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*, util::SubscriberInitExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Compact,
    Json,
    Pretty,
}

impl FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(LogFormat::Json),
            "compact" => Ok(LogFormat::Compact),
            "pretty" => Ok(LogFormat::Pretty),
            _ => {
                Err(format!("Unknown log format '{}', expected 'compact', 'json', or 'pretty'", s))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub default_level: Level,
    pub format: LogFormat,
    pub env_var_name: &'static str,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self { default_level: Level::INFO, format: LogFormat::Compact, env_var_name: "RUST_LOG" }
    }
}

impl TelemetryConfig {
    pub fn new(default_level: Level, format: LogFormat) -> Self {
        Self { default_level, format, env_var_name: "RUST_LOG" }
    }

    pub fn with_env_var(mut self, var_name: &'static str) -> Self {
        self.env_var_name = var_name;
        self
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TelemetryError {
    AlreadyInitialized,
    ConfigurationError(String),
}

impl core::fmt::Display for TelemetryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TelemetryError::AlreadyInitialized => {
                write!(f, "Tracing subscriber is already initialized in this process")
            }
            TelemetryError::ConfigurationError(msg) => {
                write!(f, "Telemetry configuration error: {}", msg)
            }
        }
    }
}

impl std::error::Error for TelemetryError {}

/// Initializes the global tracing subscriber with EnvFilter and selected output format.
/// Returns `Ok(())` on success or `Err(TelemetryError::AlreadyInitialized)` if already active.
pub fn init_telemetry(config: &TelemetryConfig) -> Result<(), TelemetryError> {
    let env_filter = EnvFilter::try_from_env(config.env_var_name)
        .unwrap_or_else(|_| EnvFilter::new(config.default_level.as_str().to_lowercase()));

    match config.format {
        LogFormat::Json => {
            let subscriber =
                tracing_subscriber::registry().with(env_filter).with(fmt::layer().json());
            subscriber.try_init().map_err(|_| TelemetryError::AlreadyInitialized)
        }
        LogFormat::Pretty => {
            let subscriber =
                tracing_subscriber::registry().with(env_filter).with(fmt::layer().pretty());
            subscriber.try_init().map_err(|_| TelemetryError::AlreadyInitialized)
        }
        LogFormat::Compact => {
            let subscriber =
                tracing_subscriber::registry().with(env_filter).with(fmt::layer().compact());
            subscriber.try_init().map_err(|_| TelemetryError::AlreadyInitialized)
        }
    }
}

/// Convenience initializer using default configuration (INFO level, compact format).
pub fn init_default() -> Result<(), TelemetryError> {
    init_telemetry(&TelemetryConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_from_str() {
        assert_eq!("compact".parse::<LogFormat>().unwrap(), LogFormat::Compact);
        assert_eq!("json".parse::<LogFormat>().unwrap(), LogFormat::Json);
        assert_eq!("pretty".parse::<LogFormat>().unwrap(), LogFormat::Pretty);
        assert_eq!("JSON".parse::<LogFormat>().unwrap(), LogFormat::Json);
        assert!("invalid".parse::<LogFormat>().is_err());
    }

    #[test]
    fn test_telemetry_config_defaults() {
        let cfg = TelemetryConfig::default();
        assert_eq!(cfg.default_level, Level::INFO);
        assert_eq!(cfg.format, LogFormat::Compact);
        assert_eq!(cfg.env_var_name, "RUST_LOG");

        let custom = TelemetryConfig::new(Level::DEBUG, LogFormat::Json).with_env_var("CUSTOM_LOG");
        assert_eq!(custom.default_level, Level::DEBUG);
        assert_eq!(custom.format, LogFormat::Json);
        assert_eq!(custom.env_var_name, "CUSTOM_LOG");
    }

    #[test]
    fn test_idempotent_subscriber_init_no_panic() {
        let cfg = TelemetryConfig::new(Level::INFO, LogFormat::Compact);
        // The first call might succeed or fail if another test initialized it first
        let _ = init_telemetry(&cfg);

        // Subsequent call must return Err(TelemetryError::AlreadyInitialized) and NOT panic!
        let second_call = init_telemetry(&cfg);
        assert_eq!(second_call, Err(TelemetryError::AlreadyInitialized));
    }

    #[test]
    fn test_env_filter_creation_levels() {
        for level in [Level::ERROR, Level::WARN, Level::INFO, Level::DEBUG, Level::TRACE] {
            let filter = EnvFilter::new(level.as_str().to_lowercase());
            assert_eq!(filter.to_string(), level.as_str().to_lowercase());
        }
    }

    #[test]
    fn test_tracing_spans_execution() {
        // Ensure spans can be entered and exit cleanly without panicking
        let span = tracing::info_span!("contract_call", contract_id = "CA123", method = "deposit");
        let _enter = span.enter();
        tracing::info!(amount = 500, "Simulating gift deposit within span");
    }
}
