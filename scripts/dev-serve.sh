#!/bin/bash
# Dev server launcher with build spinner

MONITOR="${1:-0}"
FULLSCREEN="${2:-}"

cd "$(dirname "$0")/../playground" || exit 1

# Spinner characters
SPINNER='⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏'

# Start spinner in background
spin() {
    local i=0
    while true; do
        printf "\r  ${SPINNER:i++%${#SPINNER}:1} Building playground..."
        sleep 0.1
    done
}

# Start spinner
spin &
SPIN_PID=$!
trap "kill $SPIN_PID 2>/dev/null" EXIT

# Build first (quiet)
cargo build --quiet 2>/dev/null

# Stop spinner
kill $SPIN_PID 2>/dev/null
wait $SPIN_PID 2>/dev/null
printf "\r  ✓ Build complete, launching...     \n"

# Launch dx serve
if [ -n "$FULLSCREEN" ]; then
    DI_FULLSCREEN=1 DI_MONITOR="$MONITOR" exec dx serve
else
    exec dx serve
fi
