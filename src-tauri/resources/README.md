# DazPilot Resources

This directory contains bundled resources for the DazPilot application.

## Platform Binaries

### Windows (Pre-built)
- `DazPilotBridge.dll` - Daz Studio bridge plugin
- `llama/llama-server.exe` - Local GGUF AI server (Windows x64)
- `llama/llama.dll` - Required llama.cpp runtime library

### macOS/Linux (Build Required)
The bridge plugin and llama-server must be built natively on each platform.

For macOS:
- Build `libDazPilotBridge.dylib` using the Daz SDK on macOS
- Build `llama-server` for macOS (arm64 or x86_64)

For Linux:
- Build `libDazPilotBridge.so` using the Daz SDK on Linux
- Build `llama-server` for Linux (x86_64)

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

## What the SDK Provides

The SDK headers are used by the AI indexer to build a knowledge base of Daz Studio's internal API. This enables:
- More accurate DazScript generation
- Better understanding of scene manipulation commands
- Context-aware suggestions for Daz operations

**Note**: The SDK is only needed for AI indexing. The bridge plugin DLL is pre-compiled and bundled with DazPilot.

## Bridge Plugin

The bridge plugin (`DazPilotBridge.dll`) is a proper Daz Studio plugin that:
- Exports `getSDKVersion` and `getPluginDefinition` (required by Daz Studio)
- Creates a TCP server on `127.0.0.1:8765` for communication with DazPilot
- Handles Daz Studio's single-threaded SDK via Qt event marshaling

## Models Directory

The `models/` directory stores downloaded GGUF model weights for the local AI backend.
Default location: `%APPDATA%/com.dazpilot.app/models/`

## Configuration

Bridge configuration is stored in:
- Windows: `%APPDATA%/com.dazpilot.app/bridge_config.json`
- macOS: `~/Library/Application Support/com.dazpilot.app/bridge_config.json`
- Linux: `~/.local/share/com.dazpilot.app/bridge_config.json`
