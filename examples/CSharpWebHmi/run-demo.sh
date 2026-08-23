#!/usr/bin/env bash
set -euo pipefail

EXAMPLE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$EXAMPLE_DIR/../.." && pwd)"

cd "$REPO_ROOT"
cargo build --release --features ffi --locked

cd "$EXAMPLE_DIR/frontend"
npm ci
npm run build

cd "$EXAMPLE_DIR"
dotnet run -c Release
