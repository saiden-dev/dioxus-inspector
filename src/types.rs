//! Request and response types for the inspector bridge.

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

/// Command sent from HTTP server to Dioxus app for JavaScript evaluation.
pub struct EvalCommand {
    pub script: String,
    pub response_tx: oneshot::Sender<EvalResponse>,
}

/// Request to evaluate JavaScript in the webview.
#[derive(Debug, Deserialize)]
pub struct EvalRequest {
    pub script: String,
}

/// Response from JavaScript evaluation.
#[derive(Debug, Serialize, Clone)]
pub struct EvalResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl EvalResponse {
    pub fn success(result: impl Into<String>) -> Self {
        Self {
            success: true,
            result: Some(result.into()),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            result: None,
            error: Some(message.into()),
        }
    }
}

/// Query request for CSS selector.
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub selector: String,
    #[serde(default)]
    pub property: Option<String>,
}

/// Status response showing bridge health.
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: &'static str,
    pub app: String,
    pub pid: u32,
    pub uptime_secs: u64,
    pub uptime_human: String,
}

/// Request for element inspection.
#[derive(Debug, Deserialize)]
pub struct InspectRequest {
    pub selector: String,
}

/// Request to validate CSS classes.
#[derive(Debug, Deserialize)]
pub struct ValidateClassesRequest {
    pub classes: Vec<String>,
}

/// Screenshot request.
#[derive(Debug, Deserialize, Default)]
pub struct ScreenshotRequest {
    #[serde(default)]
    pub path: Option<String>,
}

/// Screenshot response.
#[derive(Debug, Serialize)]
pub struct ScreenshotResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
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
}
