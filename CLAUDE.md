# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Dioxus Inspector is a debugging toolkit for Dioxus desktop applications. It consists of two main components:

1. **dioxus-inspector** (library) - HTTP bridge embedded in your Dioxus app that exposes DOM inspection and JavaScript evaluation endpoints
2. **dioxus-mcp** (binary) - MCP server that connects to the bridge and exposes tools for Claude Code

## Build Commands

```bash
# Build all workspace members
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run a single test
cargo test test_name

# Run playground app (Dioxus desktop demo)
cargo run -p playground
```

## Architecture

### Workspace Structure

- `src/` - Main library (dioxus-inspector)
- `mcp-server/` - MCP server binary (dioxus-mcp)
- `playground/` - Demo Dioxus desktop app for testing

### Data Flow

```
Claude Code <--MCP--> dioxus-mcp <--HTTP--> dioxus-inspector (in Dioxus app) <--eval--> WebView
```

1. Dioxus app calls `start_bridge(port, app_name)` which spawns an Axum HTTP server and returns an `mpsc::Receiver<EvalCommand>`
2. App polls the receiver and executes JavaScript via `document::eval()`, sending responses back through oneshot channels
3. MCP server (`dioxus-mcp`) connects to the bridge via HTTP and translates MCP tool calls to bridge endpoints

### Key Types

- `BridgeState` - Shared state containing app name, eval channel, uptime, PID
- `EvalCommand` - Script + oneshot response channel, sent from HTTP handler to app
- `EvalResponse` - Success/error with result string

### HTTP Endpoints (bridge)

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/status` | GET | Health check, PID, uptime |
| `/eval` | POST | Execute arbitrary JavaScript |
| `/query` | POST | Query DOM by CSS selector (text/html/value/attr) |
| `/dom` | GET | Simplified DOM tree |
| `/inspect` | POST | Element visibility analysis |
| `/validate-classes` | POST | Check CSS class availability |
| `/diagnose` | GET | Quick UI health check |
| `/screenshot` | POST | Capture window (macOS only) |

### MCP Tools (exposed to Claude Code)

`status`, `get_dom`, `query_text`, `query_html`, `query_all`, `click`, `type_text`, `eval`, `inspect`, `diagnose`, `screenshot`

### JavaScript Scripts

Embedded JS for complex operations lives in `src/scripts/`:
- `dom.js` - Builds simplified DOM tree
- `inspect.js` - Element visibility analysis
- `validate_classes.js` - CSS class availability check
- `diagnose.js` - UI health diagnostics

## MCP Server Configuration

Configure in Claude Code settings:
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

## Platform Notes

- Screenshot capture only works on macOS (uses Core Graphics)
- Bridge listens on `127.0.0.1:{port}` (localhost only)
- Default bridge port is 9999
