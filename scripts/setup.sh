#!/usr/bin/env bash
set -euo pipefail

echo "=== DazPilot Setup ==="

# Check Node.js
if ! command -v node &>/dev/null; then
  echo "ERROR: Node.js is not installed. Please install Node.js 20+."
  exit 1
fi
NODE_VER=$(node -v | sed 's/v//' | cut -d. -f1)
if [ "$NODE_VER" -lt 20 ]; then
  echo "ERROR: Node.js 20+ required (found v$(node -v))"
  exit 1
fi
echo "Node.js $(node -v) found"

# Check npm
if ! command -v npm &>/dev/null; then
  echo "ERROR: npm not found."
  exit 1
fi
echo "npm $(npm -v) found"

# Check Rust
if ! command -v rustc &>/dev/null; then
  echo "WARNING: Rust not found. Install from https://rustup.rs"
  echo "Frontend-only mode will work, but Tauri builds will fail."
fi

# Check CMake
if ! command -v cmake &>/dev/null; then
  echo "WARNING: CMake not found. Bridge plugin builds will be skipped."
fi

# Install npm dependencies
echo "Installing npm dependencies..."
npm install
echo "Setup complete!"
