# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Overview

Dioxus Inspector is a debugging toolkit for Dioxus desktop applications:

1. **dioxus-inspector** (library) - HTTP bridge embedded in your Dioxus app that exposes DOM inspection and JavaScript evaluation endpoints
2. **dioxus-mcp** (binary) - MCP server that connects to the bridge and exposes tools for Claude Code

## Quick Reference

```bash
just build        # Build all workspace members
just test         # Run tests
just check        # Format check + clippy
just playground   # Run demo app
just fmt          # Format code
```

## Architecture

### Workspace Structure

```
dioxus-inspector/
├── src/              # Library (dioxus-inspector) - HTTP bridge
│   ├── lib.rs        # Public API: start_bridge()
│   ├── handlers.rs   # Axum route handlers
│   └── scripts/      # Embedded JavaScript
├── mcp-server/       # Binary (dioxus-mcp) - MCP server
│   └── src/main.rs   # MCP protocol implementation
└── playground/       # Demo Dioxus desktop app
    └── src/main.rs   # Example integration
```

### Data Flow

```
Claude Code <--MCP--> dioxus-mcp <--HTTP--> dioxus-inspector (in app) <--eval--> WebView
```

1. Dioxus app calls `start_bridge(port, app_name)` → spawns Axum server, returns `mpsc::Receiver<EvalCommand>`
2. App polls receiver, executes JavaScript via `document::eval()`, sends responses through oneshot channels
3. MCP server translates tool calls to HTTP requests against the bridge

### Key Types

| Type | Purpose |
|------|---------|
| `BridgeState` | Shared state: app name, eval channel, uptime, PID |
| `EvalCommand` | Script + oneshot response channel |
| `EvalResponse` | Success/error with result string |

### HTTP Endpoints (bridge)

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/status` | GET | Health check, PID, uptime |
| `/eval` | POST | Execute arbitrary JavaScript |
| `/query` | POST | Query DOM by CSS selector |
| `/dom` | GET | Simplified DOM tree |
| `/inspect` | POST | Element visibility analysis |
| `/validate-classes` | POST | Check CSS class availability |
| `/diagnose` | GET | Quick UI health check |
| `/screenshot` | POST | Capture window (macOS only) |

### MCP Tools

`status`, `get_dom`, `query_text`, `query_html`, `query_all`, `click`, `type_text`, `eval`, `inspect`, `diagnose`, `screenshot`

## Code Style

### Which Rules Apply Where

| Crate | Rules | Notes |
|-------|-------|-------|
| `src/` (library) | Rust CLI rules | No GUI, pure async HTTP |
| `mcp-server/` | Rust CLI rules | Thin binary, MCP protocol |
| `playground/` | Rust GUI rules | Dioxus desktop patterns |

### Key Conventions

- **File size**: 300-500 lines max
- **Function size**: 50 lines max
- **No `.unwrap()`** in library code (use `.unwrap_or()`, `.expect()`, or `?`)
- **Flat module exports**: `pub use` at module level, hide submodules
- **Two-level run pattern**: `run()` creates deps, `run_with()` accepts traits for testing

## Testing

```bash
just test                    # Run all tests
just test-verbose            # With output
cargo test test_name         # Single test
cargo tarpaulin              # Coverage report
```

### Guidelines

- Test JSON serialization, request building, response parsing
- Mock HTTP boundaries with test servers
- Separate logic from I/O for testability
- Use `#[cfg(not(tarpaulin_include))]` for untestable integration code

## Dependencies

### Library (dioxus-inspector)
- `axum` - HTTP server
- `tokio` - Async runtime
- `serde`, `serde_json` - Serialization
- `tracing` - Logging
- `core-graphics`, `image` - macOS screenshot (platform-specific)

### MCP Server (dioxus-mcp)
- `reqwest` - HTTP client
- `anyhow` - Error handling
- `tracing-subscriber` - Log formatting

### Playground
- `dioxus` (desktop feature) - GUI framework

## Development Workflow

1. **Make changes** to library or MCP server
2. **Run `just check`** - format + clippy must pass
3. **Run `just test`** - all tests must pass
4. **Test with playground** - `just playground` in one terminal
5. **Test MCP** - Use Claude Code with dioxus-mcp configured

## MCP Server Configuration

Add to Claude Code settings (`~/.claude/settings.json`):

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

- Screenshot capture: macOS only (Core Graphics)
- Bridge binds to `127.0.0.1:{port}` (localhost only)
- Default port: 9999
