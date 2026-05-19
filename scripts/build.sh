#!/usr/bin/env bash
set -euo pipefail

echo "=== DazPilot Build ==="
cd "$(dirname "$0")/.."

npm run build
echo "Frontend build complete."

if command -v cargo &>/dev/null; then
  cd src-tauri
  cargo build --release
  cd ..
  echo "Tauri backend build complete."
else
  echo "Skipping Rust build (cargo not found)."
fi

echo "Build complete!"
