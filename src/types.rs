//! Request and response types for the inspector bridge.
//!
//! This module contains all the types used for communication between the HTTP bridge
//! and the Dioxus application, as well as the JSON request/response types for the API.

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

/// Command sent from HTTP server to Dioxus app for JavaScript evaluation.
///
/// When the bridge receives an eval request, it creates an `EvalCommand` and sends it
/// through the channel. The Dioxus app should poll this channel and execute the script.
///
/// # Example
///
/// ```rust,ignore
/// while let Some(cmd) = eval_rx.recv().await {
///     let result = document::eval(&cmd.script).await;
///     let response = match result {
///         Ok(val) => EvalResponse::success(val.to_string()),
///         Err(e) => EvalResponse::error(e.to_string()),
///     };
///     let _ = cmd.response_tx.send(response);
/// }
/// ```
pub struct EvalCommand {
    /// The JavaScript code to execute in the webview.
    pub script: String,
    /// Channel to send the evaluation result back to the HTTP handler.
    pub response_tx: oneshot::Sender<EvalResponse>,
}

/// Request to evaluate JavaScript in the webview.
///
/// # JSON Format
///
/// ```json
/// { "script": "return document.title" }
/// ```
#[derive(Debug, Deserialize)]
pub struct EvalRequest {
    /// The JavaScript code to execute. Should return a value.
    pub script: String,
}

/// Response from JavaScript evaluation.
///
/// # JSON Format
///
/// Success:
/// ```json
/// { "success": true, "result": "Page Title" }
/// ```
///
/// Error:
/// ```json
/// { "success": false, "error": "ReferenceError: x is not defined" }
/// ```
#[derive(Debug, Serialize, Clone)]
pub struct EvalResponse {
    /// Whether the evaluation succeeded.
    pub success: bool,
    /// The result of the evaluation, if successful.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    /// The error message, if evaluation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl EvalResponse {
    /// Create a successful response with the given result.
    ///
    /// # Example
    ///
    /// ```
    /// use dioxus_inspector::EvalResponse;
    ///
    /// let resp = EvalResponse::success("42");
    /// assert!(resp.success);
    /// assert_eq!(resp.result, Some("42".to_string()));
    /// ```
    pub fn success(result: impl Into<String>) -> Self {
        Self {
            success: true,
            result: Some(result.into()),
            error: None,
        }
    }

    /// Create an error response with the given message.
    ///
    /// # Example
    ///
    /// ```
    /// use dioxus_inspector::EvalResponse;
    ///
    /// let resp = EvalResponse::error("Script timeout");
    /// assert!(!resp.success);
    /// assert_eq!(resp.error, Some("Script timeout".to_string()));
    /// ```
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            result: None,
            error: Some(message.into()),
        }
    }
}

/// Query request for CSS selector.
///
/// # JSON Format
///
/// ```json
/// { "selector": ".my-button", "property": "text" }
/// ```
///
/// # Supported Properties
///
/// - `text` (default) - Element's `textContent`
/// - `html` - Element's `innerHTML`
/// - `outerHTML` - Element's `outerHTML`
/// - `value` - Element's `value` (for inputs)
/// - Any other string - Treated as an attribute name
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    /// CSS selector to find the element.
    pub selector: String,
    /// Property to extract. Defaults to "text" if not specified.
    #[serde(default)]
    pub property: Option<String>,
}

/// Status response showing bridge health.
///
/// Returned by `GET /status` to check if the bridge is running.
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    /// Always "ok" when the bridge is healthy.
    pub status: &'static str,
    /// The application name provided to [`start_bridge`](crate::start_bridge).
    pub app: String,
    /// Process ID of the Dioxus application.
    pub pid: u32,
    /// Uptime in seconds since the bridge started.
    pub uptime_secs: u64,
    /// Human-readable uptime (e.g., "5m 30s", "2h 15m").
    pub uptime_human: String,
}

/// Request for element inspection.
///
/// Returns detailed visibility and position information for an element.
///
/// # JSON Format
///
/// ```json
/// { "selector": ".modal" }
/// ```
#[derive(Debug, Deserialize)]
pub struct InspectRequest {
    /// CSS selector to find the element to inspect.
    pub selector: String,
}

/// Request to validate CSS classes.
///
/// Checks which CSS classes are available in the document's stylesheets.
///
/// # JSON Format
///
/// ```json
/// { "classes": ["flex", "p-4", "bg-white"] }
/// ```
#[derive(Debug, Deserialize)]
pub struct ValidateClassesRequest {
    /// List of CSS class names to validate.
    pub classes: Vec<String>,
}

/// Screenshot request.
///
/// Captures a screenshot of the application window (macOS only).
///
/// # JSON Format
///
/// ```json
/// { "path": "/tmp/screenshot.png" }
/// ```
#[derive(Debug, Deserialize, Default)]
pub struct ScreenshotRequest {
    /// Output path for the screenshot. Defaults to `/tmp/dioxus-screenshot.png`.
    #[serde(default)]
    pub path: Option<String>,
}

/// Screenshot response.
#[derive(Debug, Serialize)]
pub struct ScreenshotResponse {
    /// Whether the screenshot was captured successfully.
    pub success: bool,
    /// Path where the screenshot was saved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Error message if capture failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Request to resize the window.
///
/// # JSON Format
///
/// ```json
/// { "width": 1024, "height": 768 }
/// ```
///
/// # Note
///
/// The application must handle the resize command. The bridge sends a special
/// script pattern that the app can intercept to apply the resize.
#[derive(Debug, Deserialize)]
pub struct ResizeRequest {
    /// Target width in pixels.
    pub width: u32,
    /// Target height in pixels.
    pub height: u32,
}

/// Response from resize operation.
#[derive(Debug, Serialize)]
pub struct ResizeResponse {
    /// Whether the resize command was sent successfully.
    pub success: bool,
    /// The requested width.
    pub width: u32,
    /// The requested height.
    pub height: u32,
    /// Error message if the resize failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_response_success() {
        let resp = EvalResponse::success("42");
        assert!(resp.success);
        assert_eq!(resp.result, Some("42".to_string()));
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_eval_response_error() {
        let resp = EvalResponse::error("failed");
        assert!(!resp.success);
        assert!(resp.result.is_none());
        assert_eq!(resp.error, Some("failed".to_string()));
    }

    #[test]
    fn test_eval_request_deserialize() {
        let json = r#"{"script": "return 1 + 1"}"#;
        let req: EvalRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.script, "return 1 + 1");
    }

    #[test]
    fn test_query_request_deserialize() {
        let json = r#"{"selector": ".button", "property": "text"}"#;
        let req: QueryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.selector, ".button");
        assert_eq!(req.property, Some("text".to_string()));
    }

    #[test]
    fn test_query_request_without_property() {
        let json = r##"{"selector": "#main"}"##;
        let req: QueryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.selector, "#main");
        assert!(req.property.is_none());
    }

    #[test]
    fn test_inspect_request_deserialize() {
        let json = r#"{"selector": ".modal"}"#;
        let req: InspectRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.selector, ".modal");
    }

    #[test]
    fn test_validate_classes_request_deserialize() {
        let json = r#"{"classes": ["flex", "p-4", "bg-white"]}"#;
        let req: ValidateClassesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.classes, vec!["flex", "p-4", "bg-white"]);
    }

    #[test]
    fn test_screenshot_request_default() {
        let req = ScreenshotRequest::default();
        assert!(req.path.is_none());
    }

    #[test]
    fn test_status_response_serialize() {
        let resp = StatusResponse {
            status: "ok",
            app: "test".to_string(),
            pid: 1234,
            uptime_secs: 60,
            uptime_human: "1m 0s".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"app\":\"test\""));
    }

    #[test]
    fn test_resize_request_deserialize() {
        let json = r#"{"width": 800, "height": 600}"#;
        let req: ResizeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.width, 800);
        assert_eq!(req.height, 600);
    }

    #[test]
    fn test_resize_response_serialize() {
        let resp = ResizeResponse {
            success: true,
            width: 1024,
            height: 768,
            error: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"width\":1024"));
        assert!(json.contains("\"height\":768"));
        assert!(!json.contains("error"));
    }
}
