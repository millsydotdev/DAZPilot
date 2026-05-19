#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "Starting DazPilot in development mode..."
npm run tauri dev
