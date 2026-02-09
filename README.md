# dioxus-inspector

[![Crates.io](https://img.shields.io/crates/v/dioxus-inspector.svg)](https://crates.io/crates/dioxus-inspector)
[![Documentation](https://docs.rs/dioxus-inspector/badge.svg)](https://docs.rs/dioxus-inspector)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

HTTP bridge for inspecting and debugging Dioxus Desktop apps. Embed this in your app to enable MCP-based debugging from Claude Code.

## Quick Start

```rust
use dioxus::prelude::*;
use dioxus_inspector::{start_bridge, EvalResponse};

fn main() {
    dioxus::launch(app);
}

fn app() -> Element {
    use_inspector_bridge(9999, "my-app");
    rsx! { div { "Hello, inspector!" } }
}

fn use_inspector_bridge(port: u16, app_name: &str) {
    use_hook(|| {
        let mut eval_rx = start_bridge(port, app_name);
        spawn(async move {
            while let Some(cmd) = eval_rx.recv().await {
                let result = document::eval(&cmd.script).await;
                let response = match result {
                    Ok(val) => EvalResponse::success(val.to_string()),
                    Err(e) => EvalResponse::error(e.to_string()),
                };
                let _ = cmd.response_tx.send(response);
            }
        });
    });
}
```

## Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/status` | GET | App status, PID, uptime |
| `/eval` | POST | Execute JavaScript in webview |
| `/query` | POST | Query DOM by CSS selector |
| `/dom` | GET | Get simplified DOM tree |
| `/inspect` | POST | Element visibility analysis |
| `/validate-classes` | POST | Check CSS class availability |
| `/diagnose` | GET | Quick UI health check |
| `/screenshot` | POST | Capture window (macOS only) |
| `/resize` | POST | Resize window |

## MCP Server

Use with [dioxus-mcp](https://github.com/saiden-dev/dioxus-inspector/tree/master/mcp-server) for Claude Code integration:

```json
{
  "mcpServers": {
    "dioxus": {
      "command": "dioxus-mcp",
      "env": { "DIOXUS_BRIDGE_URL": "http://127.0.0.1:9999" }
    }
  }
}
```

## License

MIT
