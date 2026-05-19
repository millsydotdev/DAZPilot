#!/usr/bin/env bash
set -euo pipefail

echo "=== DazPilot Clean ==="
cd "$(dirname "$0")/.."

echo "This will remove: dist/, node_modules/, src-tauri/target/"
read -rp "Are you sure? (y/N) " confirm
if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
  echo "Cancelled."
  exit 0
fi

rm -rf dist/
rm -rf node_modules/
rm -rf src-tauri/target/
rm -rf plugins/daz3d-bridge/build/

echo "Clean complete."
