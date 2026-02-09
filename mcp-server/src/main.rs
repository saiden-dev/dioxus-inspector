//! Dioxus MCP Server
//!
//! Connects to a running Dioxus app's inspector bridge and exposes
//! MCP tools for DOM inspection and interaction.
//!
//! Configure in Claude Code:
//! ```json
//! {
//!   "mcpServers": {
//!     "dioxus": {
//!       "command": "dioxus-mcp",
//!       "env": { "DIOXUS_BRIDGE_URL": "http://127.0.0.1:9999" }
//!     }
//!   }
//! }
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

mod bridge;
mod tools;

use bridge::BridgeClient;

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

impl JsonRpcResponse {
    fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
            }),
        }
    }
}

fn server_info() -> Value {
    json!({
        "protocolVersion": "2024-11-05",
        "capabilities": { "tools": {} },
        "serverInfo": {
            "name": "dioxus-mcp",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

fn tools_list() -> Value {
    json!({
        "tools": [
            tool_def("status", "Check if the Dioxus app is running", json!({})),
            tool_def("get_dom", "Get simplified DOM tree", json!({})),
            tool_def("query_text", "Get element text by CSS selector", json!({
                "selector": { "type": "string", "description": "CSS selector" }
            })),
            tool_def("query_html", "Get element innerHTML by CSS selector", json!({
                "selector": { "type": "string", "description": "CSS selector" }
            })),
            tool_def("query_all", "List all elements matching a selector", json!({
                "selector": { "type": "string", "description": "CSS selector" }
            })),
            tool_def("click", "Click an element by CSS selector", json!({
                "selector": { "type": "string", "description": "CSS selector" }
            })),
            tool_def("type_text", "Type text into an input", json!({
                "selector": { "type": "string", "description": "CSS selector" },
                "text": { "type": "string", "description": "Text to type" }
            })),
            tool_def("eval", "Execute JavaScript in the webview", json!({
                "script": { "type": "string", "description": "JavaScript code" }
            })),
            tool_def("inspect", "Analyze element visibility", json!({
                "selector": { "type": "string", "description": "CSS selector" }
            })),
            tool_def("diagnose", "Quick UI health check", json!({})),
            tool_def("screenshot", "Capture window screenshot", json!({
                "path": { "type": "string", "description": "Output path (optional)" }
            })),
            tool_def("resize", "Resize the window", json!({
                "width": { "type": "number", "description": "Window width in pixels" },
                "height": { "type": "number", "description": "Window height in pixels" }
            }))
        ]
    })
}

fn tool_def(name: &str, description: &str, properties: Value) -> Value {
    let required: Vec<&str> = properties
        .as_object()
        .map(|obj| obj.keys().map(|k| k.as_str()).collect())
        .unwrap_or_default();

    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": properties,
            "required": required
        }
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .init();

    let bridge_url =
        std::env::var("DIOXUS_BRIDGE_URL").unwrap_or_else(|_| "http://127.0.0.1:9999".to_string());

    tracing::info!("Dioxus MCP server starting, bridge: {}", bridge_url);

    let bridge = BridgeClient::new(&bridge_url);
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to read stdin: {}", e);
                break;
            }
        };

        if line.is_empty() {
            continue;
        }

        tracing::debug!("Received: {}", line);

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to parse request: {}", e);
                continue;
            }
        };

        let response = handle_request(&bridge, request).await;
        let response_json = serde_json::to_string(&response)?;

        tracing::debug!("Sending: {}", response_json);
        writeln!(stdout, "{}", response_json)?;
        stdout.flush()?;
    }

    Ok(())
}

async fn handle_request(bridge: &BridgeClient, request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.clone().unwrap_or(Value::Null);

    match request.method.as_str() {
        "initialize" => JsonRpcResponse::success(id, server_info()),
        "initialized" => JsonRpcResponse::success(id, json!({})),
        "tools/list" => JsonRpcResponse::success(id, tools_list()),
        "tools/call" => {
            let params = request.params.unwrap_or(json!({}));
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            match tools::call_tool(bridge, tool_name, arguments).await {
                Ok(result) => JsonRpcResponse::success(
                    id,
                    json!({ "content": [{ "type": "text", "text": result }] }),
                ),
                Err(e) => JsonRpcResponse::error(id, -32000, e.to_string()),
            }
        }
        _ => JsonRpcResponse::error(id, -32601, format!("Method not found: {}", request.method)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_info() {
        let info = server_info();
        assert_eq!(info["serverInfo"]["name"], "dioxus-mcp");
    }

    #[test]
    fn test_tools_list_contains_status() {
        let list = tools_list();
        let tools = list["tools"].as_array().unwrap();
        assert!(tools.iter().any(|t| t["name"] == "status"));
    }

    #[test]
    fn test_json_rpc_response_success() {
        let resp = JsonRpcResponse::success(json!(1), json!({"ok": true}));
        assert!(resp.error.is_none());
        assert_eq!(resp.result, Some(json!({"ok": true})));
    }

    #[test]
    fn test_json_rpc_response_error() {
        let resp = JsonRpcResponse::error(json!(1), -32600, "Invalid request");
        assert!(resp.result.is_none());
        assert_eq!(resp.error.as_ref().unwrap().code, -32600);
    }
}
