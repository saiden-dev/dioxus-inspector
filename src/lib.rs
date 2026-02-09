//! Dioxus Inspector
//!
//! HTTP bridge for inspecting and debugging Dioxus Desktop apps.
//! Embed this in your app to enable MCP-based debugging from Claude Code.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use dioxus::prelude::*;
//! use dioxus_inspector::start_bridge;
//!
//! fn app() -> Element {
//!     use_inspector_bridge(9999, "my-app");
//!     rsx! { div { "Hello, inspector!" } }
//! }
//! ```
//!
//! # Endpoints
//!
//! - `GET /status` - App status, PID, uptime
//! - `POST /eval` - Execute JavaScript in webview
//! - `POST /query` - Query DOM by CSS selector
//! - `GET /dom` - Get simplified DOM tree
//! - `POST /inspect` - Element visibility analysis
//! - `POST /validate-classes` - Check CSS class availability
//! - `GET /diagnose` - Quick UI health check
//! - `POST /screenshot` - Capture window (macOS)

mod handlers;
mod screenshot;
mod types;

pub use types::{EvalCommand, EvalRequest, EvalResponse, QueryRequest, StatusResponse};

use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Shared state for the HTTP bridge
pub struct BridgeState {
    pub app_name: String,
    pub eval_tx: mpsc::Sender<EvalCommand>,
    pub started_at: std::time::Instant,
    pub pid: u32,
}

/// Start the inspector HTTP bridge.
///
/// Returns a receiver that your Dioxus app should poll to execute JavaScript.
/// The bridge listens on `127.0.0.1:{port}`.
///
/// # Example
///
/// ```rust,ignore
/// let mut eval_rx = start_bridge(9999, "my-app");
/// spawn(async move {
///     while let Some(cmd) = eval_rx.recv().await {
///         let result = document::eval(&cmd.script).await;
///         let response = match result {
///             Ok(val) => EvalResponse::success(val.to_string()),
///             Err(e) => EvalResponse::error(e.to_string()),
///         };
///         let _ = cmd.response_tx.send(response);
///     }
/// });
/// ```
pub fn start_bridge(port: u16, app_name: impl Into<String>) -> mpsc::Receiver<EvalCommand> {
    let (eval_tx, eval_rx) = mpsc::channel::<EvalCommand>(32);

    let state = Arc::new(BridgeState {
        app_name: app_name.into(),
        eval_tx,
        started_at: std::time::Instant::now(),
        pid: std::process::id(),
    });

    let app = Router::new()
        .route("/status", get(handlers::status))
        .route("/eval", axum::routing::post(handlers::eval))
        .route("/query", axum::routing::post(handlers::query))
        .route("/dom", get(handlers::dom))
        .route("/inspect", axum::routing::post(handlers::inspect))
        .route(
            "/validate-classes",
            axum::routing::post(handlers::validate_classes),
        )
        .route("/diagnose", get(handlers::diagnose))
        .route("/screenshot", axum::routing::post(handlers::screenshot))
        .with_state(state);

    tokio::spawn(async move {
        let addr = format!("127.0.0.1:{}", port);
        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to bind inspector bridge on {}: {}", addr, e);
                return;
            }
        };

        tracing::info!("Inspector bridge listening on http://{}", addr);
        let _ = axum::serve(listener, app).await;
    });

    eval_rx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_state_creation() {
        let (tx, _rx) = mpsc::channel(1);
        let state = BridgeState {
            app_name: "test".to_string(),
            eval_tx: tx,
            started_at: std::time::Instant::now(),
            pid: 12345,
        };
        assert_eq!(state.app_name, "test");
        assert_eq!(state.pid, 12345);
    }
}
