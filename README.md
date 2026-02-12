<p align="center">
  <img src="https://raw.githubusercontent.com/saiden-dev/dioxus-inspector/master/logo.png" alt="dioxus-inspector" width="120" />
</p>

<h1 align="center">dioxus-inspector</h1>

<p align="center">
  <a href="https://crates.io/crates/dioxus-inspector"><img src="https://img.shields.io/crates/v/dioxus-inspector.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/dioxus-inspector"><img src="https://docs.rs/dioxus-inspector/badge.svg" alt="Documentation"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT"></a>
</p>

<p align="center">
  HTTP bridge for inspecting and debugging Dioxus Desktop apps.<br/>
  Embed this in your app to enable MCP-based debugging from Claude Code.
</p>

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
