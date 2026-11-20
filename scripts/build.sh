#!/bin/bash
set -euo pipefail

# Script: build.sh
# Builds nxm-tracker in release mode and copies artifacts to builds/.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILDS_DIR="$PROJECT_DIR/builds"

echo "==> Building nxm-tracker in release mode"
cd "$PROJECT_DIR"
cargo build --release

echo "==> Copying artifacts to $BUILDS_DIR"
mkdir -p "$BUILDS_DIR"
cp "target/release/nxm-tracker" "$BUILDS_DIR/"

echo "==> Build complete: $BUILDS_DIR/nxm-tracker"