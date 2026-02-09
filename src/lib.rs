//! # Dioxus Inspector
//!
//! HTTP bridge for inspecting and debugging Dioxus Desktop apps.
//! Embed this in your app to enable MCP-based debugging from Claude Code.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use dioxus::prelude::*;
//! use dioxus_inspector::{start_bridge, EvalResponse};
//!
//! fn main() {
//!     dioxus::launch(app);
//! }
//!
//! fn app() -> Element {
//!     use_inspector_bridge(9999, "my-app");
//!     rsx! { div { "Hello, inspector!" } }
//! }
//!
//! fn use_inspector_bridge(port: u16, app_name: &str) {
//!     use_hook(|| {
//!         let mut eval_rx = start_bridge(port, app_name);
//!         spawn(async move {
//!             while let Some(cmd) = eval_rx.recv().await {
//!                 let result = document::eval(&cmd.script).await;
//!                 let response = match result {
//!                     Ok(val) => EvalResponse::success(val.to_string()),
//!                     Err(e) => EvalResponse::error(e.to_string()),
//!                 };
//!                 let _ = cmd.response_tx.send(response);
//!             }
//!         });
//!     });
//! }
//! ```
//!
//! ## Architecture
//!
//! ```text
//! Claude Code <--MCP--> dioxus-mcp <--HTTP--> dioxus-inspector <--eval--> WebView
//! ```
//!
//! 1. Call [`start_bridge`] with a port and app name
//! 2. Poll the returned receiver for [`EvalCommand`]s
//! 3. Execute JavaScript via `document::eval()` and send responses back
//!
//! ## HTTP Endpoints
//!
//! | Endpoint | Method | Purpose |
//! |----------|--------|---------|
//! | `/status` | GET | App status, PID, uptime |
//! | `/eval` | POST | Execute JavaScript in webview |
//! | `/query` | POST | Query DOM by CSS selector |
//! | `/dom` | GET | Get simplified DOM tree |
//! | `/inspect` | POST | Element visibility analysis |
//! | `/validate-classes` | POST | Check CSS class availability |
//! | `/diagnose` | GET | Quick UI health check |
//! | `/screenshot` | POST | Capture window (macOS only) |
//! | `/resize` | POST | Resize window (requires app handling) |
//!
//! ## Platform Support
//!
//! - **Screenshot capture**: macOS only (uses Core Graphics)
//! - **All other features**: Cross-platform

mod handlers;
mod screenshot;
mod types;

pub use types::{
    EvalCommand, EvalRequest, EvalResponse, QueryRequest, ResizeRequest, ResizeResponse,
    StatusResponse,
};

use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Shared state for the HTTP bridge.
///
/// This struct holds the internal state used by the Axum handlers.
/// You don't need to interact with this directly.
pub struct BridgeState {
    /// The application name, used in status responses.
    pub app_name: String,
    /// Channel to send eval commands to the Dioxus app.
    pub eval_tx: mpsc::Sender<EvalCommand>,
    /// When the bridge was started, for uptime calculation.
    pub started_at: std::time::Instant,
    /// Process ID of the running application.
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
        .route("/resize", axum::routing::post(handlers::resize))
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
