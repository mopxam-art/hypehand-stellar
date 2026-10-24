use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

#[derive(Clone)]
pub struct AppState {
    pub start_time: Instant,
    pub rpc_healthy: Arc<AtomicBool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            rpc_healthy: Arc::new(AtomicBool::new(true)),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_rpc_healthy(&self, healthy: bool) {
        self.rpc_healthy.store(healthy, Ordering::SeqCst);
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthzResponse {
    pub status: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencyStatus {
    pub name: String,
    pub status: String,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReadyzResponse {
    pub status: String,
    pub checks: Vec<DependencyStatus>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionResponse {
    pub service: String,
    pub version: String,
    pub git_commit: String,
    pub build_timestamp: String,
}

/// Liveness probe: returns HTTP 200 with uptime when server process is active
pub async fn healthz_handler(State(state): State<AppState>) -> impl IntoResponse {
    let uptime = state.start_time.elapsed().as_secs();
    (
        StatusCode::OK,
        Json(HealthzResponse {
            status: "ok".to_string(),
            uptime_seconds: uptime,
        }),
    )
}

/// Readiness probe: returns HTTP 200 if dependencies (Soroban RPC / horizon) are healthy,
/// or HTTP 503 if downstream RPC is unreachable
pub async fn readyz_handler(State(state): State<AppState>) -> impl IntoResponse {
    let rpc_ok = state.rpc_healthy.load(Ordering::SeqCst);

    let checks = vec![
        DependencyStatus {
            name: "soroban_rpc".to_string(),
            status: if rpc_ok { "healthy".to_string() } else { "unreachable".to_string() },
            required: true,
        },
        DependencyStatus {
            name: "persistent_storage".to_string(),
            status: "healthy".to_string(),
            required: false,
        },
    ];

    if rpc_ok {
        (
            StatusCode::OK,
            Json(ReadyzResponse {
                status: "ready".to_string(),
                checks,
            }),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ReadyzResponse {
                status: "degraded".to_string(),
                checks,
            }),
        )
    }
}

/// Version endpoint: returns build metadata and git commit
pub async fn version_handler() -> impl IntoResponse {
    Json(VersionResponse {
        service: "hypehand-api".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: option_env!("GIT_COMMIT").unwrap_or("development").to_string(),
        build_timestamp: option_env!("BUILD_TIMESTAMP").unwrap_or("2026-10-24T18:00:00Z").to_string(),
    })
}

/// Builds the Axum router with health, readiness, and version routes
pub fn app_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz_handler))
        .route("/readyz", get(readyz_handler))
        .route("/version", get(version_handler))
        .with_state(state)
}

/// Listens for OS shutdown signals (SIGINT / SIGTERM)
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_healthz_endpoint_success() {
        let state = AppState::new();
        let app = app_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let payload: HealthzResponse =
            serde_json::from_slice(&body_bytes).expect("Deserialize healthz");

        assert_eq!(payload.status, "ok");
    }

    #[tokio::test]
    async fn test_readyz_healthy_status() {
        let state = AppState::new();
        state.set_rpc_healthy(true);
        let app = app_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/readyz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let payload: ReadyzResponse =
            serde_json::from_slice(&body_bytes).expect("Deserialize readyz");

        assert_eq!(payload.status, "ready");
        assert_eq!(payload.checks[0].name, "soroban_rpc");
        assert_eq!(payload.checks[0].status, "healthy");
    }

    #[tokio::test]
    async fn test_readyz_degraded_status_503() {
        let state = AppState::new();
        state.set_rpc_healthy(false); // Simulate Soroban RPC down
        let app = app_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/readyz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("Request failed");

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let payload: ReadyzResponse =
            serde_json::from_slice(&body_bytes).expect("Deserialize degraded readyz");

        assert_eq!(payload.status, "degraded");
        assert_eq!(payload.checks[0].status, "unreachable");
    }

    #[tokio::test]
    async fn test_version_endpoint() {
        let state = AppState::new();
        let app = app_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/version")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let payload: VersionResponse =
            serde_json::from_slice(&body_bytes).expect("Deserialize version");

        assert_eq!(payload.service, "hypehand-api");
        assert_eq!(payload.version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_not_found_route() {
        let state = AppState::new();
        let app = app_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/unknown-route")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("Request failed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
