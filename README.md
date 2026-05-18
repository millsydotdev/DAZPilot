# DazPilot

AI-assisted Daz Studio scene control through a Tauri desktop app, a local GGUF model, a Daz SDK header index, and a C++ Daz bridge plugin.

## Current Runtime Shape

- Daz Studio plugin owns the bridge server on `127.0.0.1:8765`.
- Tauri connects as a TCP client and sends newline-delimited JSON.
- Production runtime has no silent bridge mock. Dev bridge mock requires `DazPilot_DEV_MOCK_BRIDGE=1`.
- Default AI path is bundled local GGUF through `llama-server.exe`.
- Ollama is optional with `DazPilot_AI_BACKEND=ollama`.
- Mock AI is dev-only with `DazPilot_DEV_MOCK_AI=1`.

## Bridge Protocol

Request:

```json
{ "id": "request-id", "command": "list_nodes", "args": {} }
```

Success:

```json
{ "id": "request-id", "status": "ok", "data": {} }
```

Failure:

```json
{ "id": "request-id", "status": "error", "error": "message" }
```

Registered commands include `get_scene_info`, `list_nodes`, `get_selected_nodes`, `select_node`, `get_cameras`, `load_asset`, `apply_pose`, `render_preview`, `capture_viewport`, `import_model`, and `export_scene`.

## Build And Run

### 1. Daz Studio SDK Setup (Required)
The Daz Studio C++ SDK is proprietary and **cannot be hosted on GitHub**. To compile the bridge plugin, you must install the SDK locally:
1. Open the **Daz Install Manager (DIM)**.
2. Search for "Daz Studio SDK" and install it.
3. The SDK will typically be installed to a folder like `C:\Users\Public\Documents\My DAZ 3D Library\DAZStudio4.5+ SDK` or inside your default content library.
4. Copy or link the `DAZStudio4.5+ SDK` folder directly into the root of this repository (i.e., `E:\DazPilot\DAZStudio4.5+ SDK\`). 
   *Note: Our `.gitignore` is configured to ensure these files are never accidentally committed to git.*

### 2. Compiling the C++ Bridge Plugin
Once the SDK is in place, build the Daz Studio bridge plugin:
```powershell
cd plugins\daz3d-bridge
cmake -B build -S .
cmake --build build --config Release
cd ..\..
```

The plugin DLL is produced at:
```text
E:\DazPilot\plugins\daz3d-bridge\dist\Release\DazPilotBridge.dll
```
Install that DLL into your Daz Studio plugin folder (e.g., `C:\Program Files\DAZ 3D\DAZStudio4\plugins\`), start Daz Studio, and enable the Vibe Bridge.

### 3. Compiling the Tauri App
Ensure you have Node.js and Rust installed, then run:
```powershell
npm install
npm run check
npm run tauri build
```

The plugin DLL is produced at:

```text
E:\DazPilot\plugins\daz3d-bridge\dist\Release\DazPilotBridge.dll
```

Install that DLL into the Daz Studio plugin folder, start Daz Studio, then connect from the app Settings panel to `127.0.0.1:8765`.

## SDK And Asset Knowledge

- `src-tauri/src/sdk_indexer.rs` is the source of truth for SDK knowledge.
- It recursively scans the configured SDK include path, defaulting to `E:\DazPilot\DAZStudio4.5+ SDK\include`.
- SDK class, method, enum, parent, file, and line metadata are persisted to SQLite.
- Asset scanning reads Daz file metadata where available and falls back to content-path/name classification.

## Known Limitations

Scene export is still an intentionally honest unsupported error until the Daz SDK export implementation is completed in the plugin. It no longer fakes success.
