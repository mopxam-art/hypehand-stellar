use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::{Mutex, OnceLock};

static PROMETHEUS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();
static INIT_MUTEX: Mutex<()> = Mutex::new(());

#[derive(Debug, PartialEq, Eq)]
pub enum MetricsError {
    AlreadyInitialized,
    InstallationError(String),
}

impl core::fmt::Display for MetricsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MetricsError::AlreadyInitialized => {
                write!(f, "Prometheus metrics recorder is already initialized")
            }
            MetricsError::InstallationError(msg) => {
                write!(f, "Failed to install Prometheus recorder: {}", msg)
            }
        }
    }
}

impl std::error::Error for MetricsError {}

/// Initializes the global Prometheus metrics recorder safely with mutual exclusion.
/// Idempotent: returns the existing handle if already initialized.
pub fn init_metrics() -> Result<PrometheusHandle, MetricsError> {
    let _guard = INIT_MUTEX.lock().unwrap();
    if let Some(handle) = PROMETHEUS_HANDLE.get() {
        return Ok(handle.clone());
    }

    let builder = PrometheusBuilder::new();
    match builder.install_recorder() {
        Ok(handle) => {
            let _ = PROMETHEUS_HANDLE.set(handle.clone());
            Ok(handle)
        }
        Err(e) => {
            // If already installed by another subsystem, return handle if cached
            if let Some(handle) = PROMETHEUS_HANDLE.get() {
                Ok(handle.clone())
            } else {
                Err(MetricsError::InstallationError(e.to_string()))
            }
        }
    }
}

/// Retrieves the active Prometheus handle if initialized
pub fn get_handle() -> Option<PrometheusHandle> {
    PROMETHEUS_HANDLE.get().cloned()
}

/// Renders current metrics in Prometheus plain text format
pub fn render_metrics() -> String {
    let handle = init_metrics()
        .unwrap_or_else(|_| get_handle().expect("Prometheus handle should be available"));
    handle.render()
}

/// Records a contract invocation: increments counter and tracks latency histogram
pub fn record_invocation(method: &str, duration_seconds: f64) {
    counter!("contract_invocations_total", "method" => method.to_string()).increment(1);
    histogram!("contract_invocation_duration_seconds", "method" => method.to_string())
        .record(duration_seconds);
}

/// Records a failed invocation with the corresponding error code
pub fn record_error(error_code: &str, method: &str) {
    counter!(
        "error_count_total",
        "code" => error_code.to_string(),
        "method" => method.to_string()
    )
    .increment(1);
}

/// Sets the current number of active registered accounts in the gauge
pub fn set_active_accounts(count: f64) {
    gauge!("active_accounts_total").set(count);
}

/// Adjusts the active accounts gauge by a relative delta
pub fn increment_active_accounts(delta: f64) {
    gauge!("active_accounts_total").increment(delta);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_and_render_metrics() {
        let handle = init_metrics().expect("Metrics initialization failed");

        // Record invocations across contract methods
        record_invocation("create_account", 0.045);
        record_invocation("deposit", 0.012);
        record_invocation("get_balance", 0.003);

        // Record errors
        record_error("InsufficientBalance", "deposit");
        record_error("InvalidAmount", "create_account");

        // Update active accounts gauge
        set_active_accounts(42.0);
        increment_active_accounts(1.0);

        let rendered = handle.render();

        // Verify metrics output format
        assert!(rendered.contains("contract_invocations_total"));
        assert!(rendered.contains(r#"method="create_account""#));
        assert!(rendered.contains(r#"method="deposit""#));
        assert!(rendered.contains(r#"method="get_balance""#));

        assert!(rendered.contains("contract_invocation_duration_seconds"));
        assert!(rendered.contains("error_count_total"));
        assert!(rendered.contains(r#"code="InsufficientBalance""#));

        assert!(rendered.contains("active_accounts_total"));
        assert!(rendered.contains("43"));
    }

    #[test]
    fn test_idempotent_init_returns_handle() {
        let handle1 = init_metrics();
        assert!(handle1.is_ok());

        let handle2 = init_metrics();
        assert!(handle2.is_ok());
    }

    #[test]
    fn test_render_metrics_function() {
        let _ = init_metrics();
        record_invocation("status_probe", 0.005);
        let output = render_metrics();
        assert!(!output.is_empty());
        assert!(output.contains("contract_invocations_total"));
    }
}
