//! HTTP request handlers for the inspector bridge.

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
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

/// Query parameters for DOM endpoint.
#[derive(Debug, Default, Deserialize)]
pub struct DomQuery {
    pub depth: Option<u32>,
    pub max_nodes: Option<u32>,
    pub selector: Option<String>,
}

/// GET /dom - Get simplified DOM tree.
pub async fn dom(
    State(state): State<Arc<BridgeState>>,
    Query(query): Query<DomQuery>,
) -> Result<Json<EvalResponse>, StatusCode> {
    let depth = query.depth.unwrap_or(10);
    let max_nodes = query.max_nodes.unwrap_or(500);
    let selector_json = query
        .selector
        .as_ref()
        .map(|s| serde_json::to_string(s).unwrap_or_else(|_| "null".to_string()))
        .unwrap_or_else(|| "null".to_string());

    let script = include_str!("scripts/dom.js")
        .replace("{MAX_DEPTH}", &depth.to_string())
        .replace("{MAX_NODES}", &max_nodes.to_string())
        .replace("{SELECTOR}", &selector_json);

    let response = send_eval(&state, script).await?;
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
#[cfg(not(tarpaulin_include))]
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

    // format_uptime tests
    #[test]
    fn test_format_uptime_zero() {
        assert_eq!(format_uptime(0), "0s");
    }

    #[test]
    fn test_format_uptime_seconds() {
        assert_eq!(format_uptime(45), "45s");
    }

    #[test]
    fn test_format_uptime_exactly_one_minute() {
        assert_eq!(format_uptime(60), "1m 0s");
    }

    #[test]
    fn test_format_uptime_minutes() {
        assert_eq!(format_uptime(125), "2m 5s");
    }

    #[test]
    fn test_format_uptime_exactly_one_hour() {
        assert_eq!(format_uptime(3600), "1h 0m");
    }

    #[test]
    fn test_format_uptime_hours() {
        assert_eq!(format_uptime(3665), "1h 1m");
    }

    #[test]
    fn test_format_uptime_many_hours() {
        assert_eq!(format_uptime(86400), "24h 0m"); // 24 hours
    }

    // build_query_script tests
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
    fn test_build_query_script_outer_html() {
        let script = build_query_script("div", "outerHTML");
        assert!(script.contains("outerHTML"));
        assert!(script.contains("\"div\""));
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

    #[test]
    fn test_build_query_script_data_attribute() {
        let script = build_query_script("[data-id]", "data-id");
        assert!(script.contains("getAttribute"));
        assert!(script.contains("\"data-id\""));
    }

    #[test]
    fn test_build_query_script_escapes_selector() {
        let script = build_query_script("div[data-name=\"test\"]", "text");
        // Selector should be JSON-escaped
        assert!(script.contains("\\\"test\\\""));
    }

    // DomQuery tests
    #[test]
    fn test_dom_query_default() {
        let query = DomQuery::default();
        assert!(query.depth.is_none());
        assert!(query.max_nodes.is_none());
        assert!(query.selector.is_none());
    }

    #[test]
    fn test_dom_query_deserialize_empty() {
        let json = "{}";
        let query: DomQuery = serde_json::from_str(json).unwrap();
        assert!(query.depth.is_none());
        assert!(query.max_nodes.is_none());
        assert!(query.selector.is_none());
    }

    #[test]
    fn test_dom_query_deserialize_full() {
        let json = r#"{"depth": 5, "max_nodes": 100, "selector": ".container"}"#;
        let query: DomQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.depth, Some(5));
        assert_eq!(query.max_nodes, Some(100));
        assert_eq!(query.selector, Some(".container".to_string()));
    }

    #[test]
    fn test_dom_query_deserialize_partial() {
        let json = r#"{"depth": 3}"#;
        let query: DomQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.depth, Some(3));
        assert!(query.max_nodes.is_none());
        assert!(query.selector.is_none());
    }

    // Handler integration tests
    mod integration {
        use super::*;
        use axum::{body::Body, http::Request, routing::get, Router};
        use http_body_util::BodyExt;
        use tokio::sync::mpsc;
        use tower::ServiceExt;

        fn create_test_state() -> (Arc<BridgeState>, mpsc::Receiver<EvalCommand>) {
            let (eval_tx, eval_rx) = mpsc::channel(32);
            let state = Arc::new(BridgeState {
                app_name: "test-app".to_string(),
                eval_tx,
                started_at: std::time::Instant::now(),
                pid: 12345,
            });
            (state, eval_rx)
        }

        #[tokio::test]
        async fn test_status_handler() {
            let (state, _rx) = create_test_state();
            let app = Router::new()
                .route("/status", get(status))
                .with_state(state);

            let response = app
                .oneshot(Request::get("/status").body(Body::empty()).unwrap())
                .await
                .unwrap();

            assert_eq!(response.status(), 200);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            assert_eq!(json["status"], "ok");
            assert_eq!(json["app"], "test-app");
            assert_eq!(json["pid"], 12345);
            assert!(json["uptime_secs"].is_number());
            assert!(json["uptime_human"].is_string());
        }

        #[tokio::test]
        async fn test_eval_handler_success() {
            let (state, mut rx) = create_test_state();
            let app = Router::new()
                .route("/eval", axum::routing::post(eval))
                .with_state(state);

            // Spawn responder
            tokio::spawn(async move {
                if let Some(cmd) = rx.recv().await {
                    let _ = cmd.response_tx.send(EvalResponse::success("42"));
                }
            });

            let response = app
                .oneshot(
                    Request::post("/eval")
                        .header("content-type", "application/json")
                        .body(Body::from(r#"{"script": "return 42"}"#))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), 200);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            assert_eq!(json["success"], true);
            assert_eq!(json["result"], "42");
        }

        #[tokio::test]
        async fn test_eval_handler_error() {
            let (state, mut rx) = create_test_state();
            let app = Router::new()
                .route("/eval", axum::routing::post(eval))
                .with_state(state);

            // Spawn responder with error
            tokio::spawn(async move {
                if let Some(cmd) = rx.recv().await {
                    let _ = cmd.response_tx.send(EvalResponse::error("Script failed"));
                }
            });

            let response = app
                .oneshot(
                    Request::post("/eval")
                        .header("content-type", "application/json")
                        .body(Body::from(r#"{"script": "invalid"}"#))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), 200);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            assert_eq!(json["success"], false);
            assert_eq!(json["error"], "Script failed");
        }

        #[tokio::test]
        async fn test_query_handler() {
            let (state, mut rx) = create_test_state();
            let app = Router::new()
                .route("/query", axum::routing::post(query))
                .with_state(state);

            // Spawn responder that validates the script
            tokio::spawn(async move {
                if let Some(cmd) = rx.recv().await {
                    // Verify the script contains textContent (default property)
                    assert!(cmd.script.contains("textContent"));
                    assert!(cmd.script.contains(".btn"));
                    let _ = cmd.response_tx.send(EvalResponse::success("Click me"));
                }
            });

            let response = app
                .oneshot(
                    Request::post("/query")
                        .header("content-type", "application/json")
                        .body(Body::from(r#"{"selector": ".btn"}"#))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), 200);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            assert_eq!(json["success"], true);
            assert_eq!(json["result"], "Click me");
        }

        #[tokio::test]
        async fn test_query_handler_with_property() {
            let (state, mut rx) = create_test_state();
            let app = Router::new()
                .route("/query", axum::routing::post(query))
                .with_state(state);

            // Spawn responder that validates the script
            tokio::spawn(async move {
                if let Some(cmd) = rx.recv().await {
                    // Verify the script contains innerHTML
                    assert!(cmd.script.contains("innerHTML"));
                    let _ = cmd
                        .response_tx
                        .send(EvalResponse::success("<span>Content</span>"));
                }
            });

            let response = app
                .oneshot(
                    Request::post("/query")
                        .header("content-type", "application/json")
                        .body(Body::from(r##"{"selector": "#main", "property": "html"}"##))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), 200);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            assert_eq!(json["result"], "<span>Content</span>");
        }

        #[tokio::test]
        async fn test_dom_handler() {
            let (state, mut rx) = create_test_state();
            let app = Router::new().route("/dom", get(dom)).with_state(state);

            // Spawn responder
            tokio::spawn(async move {
                if let Some(cmd) = rx.recv().await {
                    // Verify script has default values
                    assert!(cmd.script.contains("MAX_DEPTH = 10"));
                    assert!(cmd.script.contains("MAX_NODES = 500"));
                    let _ = cmd
                        .response_tx
                        .send(EvalResponse::success(r#"{"tag":"body"}"#));
                }
            });

            let response = app
                .oneshot(Request::get("/dom").body(Body::empty()).unwrap())
                .await
                .unwrap();

            assert_eq!(response.status(), 200);
        }

        #[tokio::test]
        async fn test_dom_handler_with_params() {
            let (state, mut rx) = create_test_state();
            let app = Router::new().route("/dom", get(dom)).with_state(state);

            // Spawn responder
            tokio::spawn(async move {
                if let Some(cmd) = rx.recv().await {
                    // Verify script has custom values
                    assert!(cmd.script.contains("MAX_DEPTH = 5"));
                    assert!(cmd.script.contains("MAX_NODES = 100"));
                    assert!(cmd.script.contains("\".container\""));
                    let _ = cmd
                        .response_tx
                        .send(EvalResponse::success(r#"{"tag":"div"}"#));
                }
            });

            let response = app
                .oneshot(
                    Request::get("/dom?depth=5&max_nodes=100&selector=.container")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), 200);
        }

        #[tokio::test]
        async fn test_inspect_handler() {
            let (state, mut rx) = create_test_state();
            let app = Router::new()
                .route("/inspect", axum::routing::post(inspect))
                .with_state(state);

            // Spawn responder
            tokio::spawn(async move {
                if let Some(cmd) = rx.recv().await {
                    assert!(cmd.script.contains(".modal"));
                    let _ = cmd
                        .response_tx
                        .send(EvalResponse::success(r#"{"visible": true, "rect": {}}"#));
                }
            });

            let response = app
                .oneshot(
                    Request::post("/inspect")
                        .header("content-type", "application/json")
                        .body(Body::from(r#"{"selector": ".modal"}"#))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), 200);
        }

        #[tokio::test]
        async fn test_validate_classes_handler() {
            let (state, mut rx) = create_test_state();
            let app = Router::new()
                .route("/validate-classes", axum::routing::post(validate_classes))
                .with_state(state);

            // Spawn responder
            tokio::spawn(async move {
                if let Some(cmd) = rx.recv().await {
                    assert!(cmd.script.contains("flex"));
                    assert!(cmd.script.contains("p-4"));
                    let _ = cmd.response_tx.send(EvalResponse::success(
                        r#"{"available": ["flex"], "missing": ["p-4"]}"#,
                    ));
                }
            });

            let response = app
                .oneshot(
                    Request::post("/validate-classes")
                        .header("content-type", "application/json")
                        .body(Body::from(r#"{"classes": ["flex", "p-4"]}"#))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), 200);
        }

        #[tokio::test]
        async fn test_diagnose_handler() {
            let (state, mut rx) = create_test_state();
            let app = Router::new()
                .route("/diagnose", get(diagnose))
                .with_state(state);

            // Spawn responder
            tokio::spawn(async move {
                if let Some(cmd) = rx.recv().await {
                    // diagnose.js should be embedded
                    assert!(!cmd.script.is_empty());
                    let _ = cmd
                        .response_tx
                        .send(EvalResponse::success(r#"{"healthy": true}"#));
                }
            });

            let response = app
                .oneshot(Request::get("/diagnose").body(Body::empty()).unwrap())
                .await
                .unwrap();

            assert_eq!(response.status(), 200);
        }

        #[tokio::test]
        async fn test_resize_handler_success() {
            let (state, mut rx) = create_test_state();
            let app = Router::new()
                .route("/resize", axum::routing::post(resize))
                .with_state(state);

            // Spawn responder
            tokio::spawn(async move {
                if let Some(cmd) = rx.recv().await {
                    assert!(cmd.script.contains("800x600"));
                    let _ = cmd.response_tx.send(EvalResponse::success(
                        "__DIOXUS_INSPECTOR_RESIZE__800x600__",
                    ));
                }
            });

            let response = app
                .oneshot(
                    Request::post("/resize")
                        .header("content-type", "application/json")
                        .body(Body::from(r#"{"width": 800, "height": 600}"#))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), 200);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            assert_eq!(json["success"], true);
            assert_eq!(json["width"], 800);
            assert_eq!(json["height"], 600);
        }

        #[tokio::test]
        async fn test_resize_handler_error() {
            let (state, mut rx) = create_test_state();
            let app = Router::new()
                .route("/resize", axum::routing::post(resize))
                .with_state(state);

            // Spawn responder with error
            tokio::spawn(async move {
                if let Some(cmd) = rx.recv().await {
                    let _ = cmd
                        .response_tx
                        .send(EvalResponse::error("Window not found"));
                }
            });

            let response = app
                .oneshot(
                    Request::post("/resize")
                        .header("content-type", "application/json")
                        .body(Body::from(r#"{"width": 800, "height": 600}"#))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), 200);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            assert_eq!(json["success"], false);
            assert_eq!(json["error"], "Window not found");
        }

        #[tokio::test]
        async fn test_eval_handler_channel_closed() {
            let (state, rx) = create_test_state();
            // Drop receiver to simulate closed channel
            drop(rx);

            let app = Router::new()
                .route("/eval", axum::routing::post(eval))
                .with_state(state);

            let response = app
                .oneshot(
                    Request::post("/eval")
                        .header("content-type", "application/json")
                        .body(Body::from(r#"{"script": "return 42"}"#))
                        .unwrap(),
                )
                .await
                .unwrap();

            // Should return 503 Service Unavailable when channel is closed
            assert_eq!(response.status(), 503);
        }
    }
}
