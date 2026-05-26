# DazPilot Resources

This directory contains bundled resources for the DazPilot application.

## Platform Binaries

### Windows (Pre-built)
- `llama/llama-server.exe` - Local GGUF AI server (Windows x64)
- `llama/llama.dll` - Required llama.cpp runtime library

### macOS/Linux (Build Required)
llama-server must be built natively on each platform.

## SDK Integration

The DAZStudio 4.5+ SDK is proprietary and must be installed separately:

1. **Install via Daz Install Manager (DIM)**:
   - Open DIM
   - Search for "DAZStudio 4.5+ SDK"
   - Install to the default location

2. **Manual Installation**:
   - Download from Daz 3D website
   - Extract to a known location
   - Configure in DazPilot Settings > AI Settings > SDK Path

3. **Auto-Discovery**:
    DazPilot will automatically search for the SDK in common DIM install locations:
    - Windows: `Documents/DAZ 3D/DAZStudio4.5+ SDK`
    - macOS: `~/Library/Application Support/DAZ 3D/DAZStudio4.5+ SDK`
    - Linux: `~/.local/share/DAZ 3D/DAZStudio4.5+ SDK`

Additionally, when used as a dependency in the DazPilot project, the SDK is expected to be located at:
- `thirdparty/DAZStudio4.5+ SDK` (relative to project root)

## What the SDK Provides

The SDK headers are used by the AI indexer to build a knowledge base of Daz Studio's internal API. This enables:
- More accurate DazScript generation
- Better understanding of scene manipulation commands
- Context-aware suggestions for Daz operations

**Note**: Bridge plugin source is at `plugins/daz3d-bridge/` — build it with CMake + Daz Studio SDK.

## Models Directory

The `models/` directory stores downloaded GGUF model weights for the local AI backend.
Default location: `%APPDATA%/com.dazpilot.desktop/models/`

## Configuration

Bridge configuration is stored in:
- Windows: `%APPDATA%/com.dazpilot.desktop/bridge_config.json`
- macOS: `~/Library/Application Support/com.dazpilot.desktop/bridge_config.json`
- Linux: `~/.local/share/com.dazpilot.desktop/bridge_config.json`
