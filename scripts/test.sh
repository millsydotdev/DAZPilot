#!/usr/bin/env bash
set -euo pipefail

echo "=== DazPilot Tests ==="
cd "$(dirname "$0")/.."

echo "--- Frontend Tests ---"
npm test

if command -v cargo &>/dev/null; then
  echo "--- Rust Backend Tests ---"
  cd src-tauri
  cargo test
  cd ..
else
  echo "Skipping Rust tests (cargo not found)."
fi

echo "All tests complete!"
