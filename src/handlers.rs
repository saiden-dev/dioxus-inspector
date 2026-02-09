//! HTTP request handlers for the inspector bridge.

use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::Json};
use tokio::sync::oneshot;

use crate::screenshot::capture_screenshot;
use crate::types::{
    EvalCommand, EvalRequest, EvalResponse, InspectRequest, QueryRequest, ResizeRequest,
    ResizeResponse, ScreenshotRequest, ScreenshotResponse, StatusResponse, ValidateClassesRequest,
};
use crate::BridgeState;

/// GET /status - Check bridge health.
pub async fn status(State(state): State<Arc<BridgeState>>) -> Json<StatusResponse> {
    let uptime = state.started_at.elapsed();
    let secs = uptime.as_secs();
    let uptime_human = format_uptime(secs);

    Json(StatusResponse {
        status: "ok",
        app: state.app_name.clone(),
        pid: state.pid,
        uptime_secs: secs,
        uptime_human,
    })
}

fn format_uptime(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// POST /eval - Execute JavaScript.
pub async fn eval(
    State(state): State<Arc<BridgeState>>,
    Json(req): Json<EvalRequest>,
) -> Result<Json<EvalResponse>, StatusCode> {
    let response = send_eval(&state, req.script).await?;
    Ok(Json(response))
}

/// POST /query - Query DOM by CSS selector.
pub async fn query(
    State(state): State<Arc<BridgeState>>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<EvalResponse>, StatusCode> {
    let property = req.property.as_deref().unwrap_or("text");
    let script = build_query_script(&req.selector, property);
    let response = send_eval(&state, script).await?;
    Ok(Json(response))
}

fn build_query_script(selector: &str, property: &str) -> String {
    let selector_json = serde_json::to_string(selector).unwrap_or_else(|_| "\"\"".to_string());

    match property {
        "text" => format!(
            r#"return (() => {{
                const el = document.querySelector({});
                return el ? el.textContent : null;
            }})()"#,
            selector_json
        ),
        "html" => format!(
            r#"return (() => {{
                const el = document.querySelector({});
                return el ? el.innerHTML : null;
            }})()"#,
            selector_json
        ),
        "outerHTML" => format!(
            r#"return (() => {{
                const el = document.querySelector({});
                return el ? el.outerHTML : null;
            }})()"#,
            selector_json
        ),
        "value" => format!(
            r#"return (() => {{
                const el = document.querySelector({});
                return el ? el.value : null;
            }})()"#,
            selector_json
        ),
        attr => {
            let attr_json = serde_json::to_string(attr).unwrap_or_else(|_| "\"\"".to_string());
            format!(
                r#"return (() => {{
                    const el = document.querySelector({});
                    return el ? el.getAttribute({}) : null;
                }})()"#,
                selector_json, attr_json
            )
        }
    }
}

/// GET /dom - Get simplified DOM tree.
pub async fn dom(State(state): State<Arc<BridgeState>>) -> Result<Json<EvalResponse>, StatusCode> {
    let script = include_str!("scripts/dom.js");
    let response = send_eval(&state, script.to_string()).await?;
    Ok(Json(response))
}

/// POST /inspect - Element visibility analysis.
pub async fn inspect(
    State(state): State<Arc<BridgeState>>,
    Json(req): Json<InspectRequest>,
) -> Result<Json<EvalResponse>, StatusCode> {
    let selector_json = serde_json::to_string(&req.selector).unwrap_or_else(|_| "\"\"".to_string());
    let script = include_str!("scripts/inspect.js").replace("{SELECTOR}", &selector_json);
    let response = send_eval(&state, script).await?;
    Ok(Json(response))
}

/// POST /validate-classes - Check CSS class availability.
pub async fn validate_classes(
    State(state): State<Arc<BridgeState>>,
    Json(req): Json<ValidateClassesRequest>,
) -> Result<Json<EvalResponse>, StatusCode> {
    let classes_json = serde_json::to_string(&req.classes).unwrap_or_else(|_| "[]".to_string());
    let script = include_str!("scripts/validate_classes.js").replace("{CLASSES}", &classes_json);
    let response = send_eval(&state, script).await?;
    Ok(Json(response))
}

/// GET /diagnose - Quick UI health check.
pub async fn diagnose(
    State(state): State<Arc<BridgeState>>,
) -> Result<Json<EvalResponse>, StatusCode> {
    let script = include_str!("scripts/diagnose.js");
    let response = send_eval(&state, script.to_string()).await?;
    Ok(Json(response))
}

/// POST /screenshot - Capture window.
pub async fn screenshot(
    State(state): State<Arc<BridgeState>>,
    body: Option<Json<ScreenshotRequest>>,
) -> Json<ScreenshotResponse> {
    let req = body.map(|j| j.0).unwrap_or_default();
    let output_path = req
        .path
        .unwrap_or_else(|| "/tmp/dioxus-screenshot.png".to_string());

    match capture_screenshot(&state.app_name, &output_path) {
        Ok(()) => Json(ScreenshotResponse {
            success: true,
            path: Some(output_path),
            error: None,
        }),
        Err(e) => Json(ScreenshotResponse {
            success: false,
            path: None,
            error: Some(e),
        }),
    }
}

/// POST /resize - Resize the window.
///
/// Sends a resize command via eval. The app must handle the special
/// `__DIOXUS_INSPECTOR_RESIZE__` script pattern to apply the resize.
pub async fn resize(
    State(state): State<Arc<BridgeState>>,
    Json(req): Json<ResizeRequest>,
) -> Result<Json<ResizeResponse>, StatusCode> {
    // Send a special script that the app can intercept
    // Format: __DIOXUS_INSPECTOR_RESIZE__{width},{height}__
    let script = format!(
        "return '__DIOXUS_INSPECTOR_RESIZE__{}x{}__'",
        req.width, req.height
    );

    let response = send_eval(&state, script).await?;

    if response.success {
        Ok(Json(ResizeResponse {
            success: true,
            width: req.width,
            height: req.height,
            error: None,
        }))
    } else {
        Ok(Json(ResizeResponse {
            success: false,
            width: req.width,
            height: req.height,
            error: response.error,
        }))
    }
}

async fn send_eval(state: &BridgeState, script: String) -> Result<EvalResponse, StatusCode> {
    let (response_tx, response_rx) = oneshot::channel();

    let cmd = EvalCommand {
        script,
        response_tx,
    };

    state
        .eval_tx
        .send(cmd)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    response_rx
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime_seconds() {
        assert_eq!(format_uptime(45), "45s");
    }

    #[test]
    fn test_format_uptime_minutes() {
        assert_eq!(format_uptime(125), "2m 5s");
    }

    #[test]
    fn test_format_uptime_hours() {
        assert_eq!(format_uptime(3665), "1h 1m");
    }

    #[test]
    fn test_build_query_script_text() {
        let script = build_query_script(".btn", "text");
        assert!(script.contains("textContent"));
        assert!(script.contains("\".btn\""));
    }

    #[test]
    fn test_build_query_script_html() {
        let script = build_query_script("#main", "html");
        assert!(script.contains("innerHTML"));
    }

    #[test]
    fn test_build_query_script_value() {
        let script = build_query_script("input", "value");
        assert!(script.contains(".value"));
    }

    #[test]
    fn test_build_query_script_attribute() {
        let script = build_query_script("a", "href");
        assert!(script.contains("getAttribute"));
        assert!(script.contains("\"href\""));
    }
}
