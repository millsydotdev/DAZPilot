#!/usr/bin/env bash
set -euo pipefail

echo "=== DazPilot Bridge Plugin Build ==="
cd "$(dirname "$0")/../plugins/daz3d-bridge"

if ! command -v cmake &>/dev/null; then
  echo "ERROR: CMake is required to build the bridge plugin."
  exit 1
fi

BUILD_DIR="build"
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

cmake .. -DCMAKE_BUILD_TYPE=Release
cmake --build . --config Release

echo "Bridge plugin built successfully."
