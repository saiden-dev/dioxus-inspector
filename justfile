# dioxus-inspector build commands

# Build all workspace members
build:
    cargo build

# Build release
release:
    cargo build --release

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Check code (format + clippy)
check:
    cargo fmt --check
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Run playground app
playground:
    cargo run -p playground

# List available monitors
monitors:
    DI_LIST_MONITORS=1 cargo run -p playground 2>/dev/null

# Run playground fullscreen on monitor N (default: 0)
playground-fs monitor="0":
    DI_FULLSCREEN=1 DI_MONITOR={{monitor}} cargo run -p playground
