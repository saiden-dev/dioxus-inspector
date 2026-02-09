//! MCP tool implementations.

use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::bridge::BridgeClient;

pub async fn call_tool(bridge: &BridgeClient, name: &str, args: Value) -> Result<String> {
    match name {
        "status" => status(bridge).await,
        "get_dom" => get_dom(bridge).await,
        "query_text" => {
            let selector = get_string_arg(&args, "selector")?;
            query_text(bridge, &selector).await
        }
        "query_html" => {
            let selector = get_string_arg(&args, "selector")?;
            query_html(bridge, &selector).await
        }
        "query_all" => {
            let selector = get_string_arg(&args, "selector")?;
            query_all(bridge, &selector).await
        }
        "click" => {
            let selector = get_string_arg(&args, "selector")?;
            click(bridge, &selector).await
        }
        "type_text" => {
            let selector = get_string_arg(&args, "selector")?;
            let text = get_string_arg(&args, "text")?;
            type_text(bridge, &selector, &text).await
        }
        "eval" => {
            let script = get_string_arg(&args, "script")?;
            eval(bridge, &script).await
        }
        "inspect" => {
            let selector = get_string_arg(&args, "selector")?;
            inspect(bridge, &selector).await
        }
        "diagnose" => diagnose(bridge).await,
        "screenshot" => {
            let path = args.get("path").and_then(|v| v.as_str());
            screenshot(bridge, path).await
        }
        "resize" => {
            let width = get_u32_arg(&args, "width")?;
            let height = get_u32_arg(&args, "height")?;
            resize(bridge, width, height).await
        }
        _ => Err(anyhow!("Unknown tool: {}", name)),
    }
}

fn get_string_arg(args: &Value, key: &str) -> Result<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| anyhow!("Missing '{}' argument", key))
}

fn get_u32_arg(args: &Value, key: &str) -> Result<u32> {
    args.get(key)
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .ok_or_else(|| anyhow!("Missing '{}' argument", key))
}

async fn status(bridge: &BridgeClient) -> Result<String> {
    match bridge.status().await {
        Ok(resp) => Ok(format!("Connected: {} ({})", resp.app, resp.status)),
        Err(e) => Ok(format!(
            "Bridge not available: {}. Start app with inspector enabled.",
            e
        )),
    }
}

async fn get_dom(bridge: &BridgeClient) -> Result<String> {
    let resp = bridge.dom().await?;
    let json_str = extract_result(resp)?;
    // Result is double-encoded: parse outer string, then inner JSON
    let inner: String = serde_json::from_str(&json_str)
        .map_err(|e| anyhow!("Failed to unescape: {}", e))?;
    let parsed: Value = serde_json::from_str(&inner)
        .map_err(|e| anyhow!("Invalid DOM JSON: {}", e))?;
    Ok(serde_json::to_string_pretty(&parsed)?)
}

async fn query_text(bridge: &BridgeClient, selector: &str) -> Result<String> {
    let resp = bridge.query(selector, Some("text")).await?;
    extract_result(resp)
}

async fn query_html(bridge: &BridgeClient, selector: &str) -> Result<String> {
    let resp = bridge.query(selector, Some("html")).await?;
    extract_result(resp)
}

async fn query_all(bridge: &BridgeClient, selector: &str) -> Result<String> {
    let script = format!(
        r#"return (() => {{
            const els = document.querySelectorAll({});
            return JSON.stringify(Array.from(els).map((el, i) => ({{
                index: i,
                tag: el.tagName.toLowerCase(),
                id: el.id || null,
                class: el.className || null,
                text: el.textContent?.trim().substring(0, 100) || null
            }})));
        }})()"#,
        serde_json::to_string(selector)?
    );
    let resp = bridge.eval(&script).await?;
    extract_result(resp)
}

async fn click(bridge: &BridgeClient, selector: &str) -> Result<String> {
    let script = format!(
        r#"return (() => {{
            const el = document.querySelector({});
            if (el) {{ el.click(); return 'clicked'; }}
            return 'element not found';
        }})()"#,
        serde_json::to_string(selector)?
    );
    let resp = bridge.eval(&script).await?;
    extract_result(resp)
}

async fn type_text(bridge: &BridgeClient, selector: &str, text: &str) -> Result<String> {
    let script = format!(
        r#"return (() => {{
            const el = document.querySelector({});
            if (el) {{
                el.value = {};
                el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                return 'typed';
            }}
            return 'element not found';
        }})()"#,
        serde_json::to_string(selector)?,
        serde_json::to_string(text)?
    );
    let resp = bridge.eval(&script).await?;
    extract_result(resp)
}

async fn eval(bridge: &BridgeClient, script: &str) -> Result<String> {
    let resp = bridge.eval(script).await?;
    extract_result(resp)
}

async fn inspect(bridge: &BridgeClient, selector: &str) -> Result<String> {
    let resp = bridge.inspect(selector).await?;
    extract_result(resp)
}

async fn diagnose(bridge: &BridgeClient) -> Result<String> {
    let resp = bridge.diagnose().await?;
    extract_result(resp)
}

async fn screenshot(bridge: &BridgeClient, path: Option<&str>) -> Result<String> {
    let resp = bridge.screenshot(path).await?;
    if resp.success {
        Ok(format!("Screenshot saved: {}", resp.path.unwrap_or_default()))
    } else {
        Err(anyhow!(resp.error.unwrap_or_else(|| "Unknown error".to_string())))
    }
}

async fn resize(bridge: &BridgeClient, width: u32, height: u32) -> Result<String> {
    let resp = bridge.resize(width, height).await?;
    if resp.success {
        Ok(format!("Window resized to {}x{}", resp.width, resp.height))
    } else {
        Err(anyhow!(resp.error.unwrap_or_else(|| "Unknown error".to_string())))
    }
}

fn extract_result(resp: crate::bridge::EvalResponse) -> Result<String> {
    if resp.success {
        Ok(resp.result.unwrap_or_else(|| "null".to_string()))
    } else {
        Err(anyhow!(resp.error.unwrap_or_else(|| "Unknown error".to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_string_arg_success() {
        let args = json!({"selector": ".button"});
        let result = get_string_arg(&args, "selector").unwrap();
        assert_eq!(result, ".button");
    }

    #[test]
    fn test_get_string_arg_missing() {
        let args = json!({});
        let result = get_string_arg(&args, "selector");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_result_success() {
        let resp = crate::bridge::EvalResponse {
            success: true,
            result: Some("42".to_string()),
            error: None,
        };
        let result = extract_result(resp).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_extract_result_error() {
        let resp = crate::bridge::EvalResponse {
            success: false,
            result: None,
            error: Some("failed".to_string()),
        };
        let result = extract_result(resp);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_u32_arg_success() {
        let args = json!({"width": 800});
        let result = get_u32_arg(&args, "width").unwrap();
        assert_eq!(result, 800);
    }

    #[test]
    fn test_get_u32_arg_missing() {
        let args = json!({});
        let result = get_u32_arg(&args, "width");
        assert!(result.is_err());
    }
}
