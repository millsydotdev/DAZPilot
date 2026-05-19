#!/usr/bin/env bash
set -euo pipefail

echo "=== DazPilot Lint ==="
cd "$(dirname "$0")/.."

echo "--- ESLint ---"
npm run lint

echo "--- Prettier Check ---"
npm run format:check

if command -v cargo &>/dev/null; then
  echo "--- Clippy ---"
  cd src-tauri
  cargo clippy -- -D warnings 2>/dev/null || echo "Clippy found warnings (non-fatal)"
  cd ..
fi

echo "Lint complete!"
