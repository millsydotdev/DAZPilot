# DazPilot Vibe Bridge Plugin

C++ Daz Studio bridge plugin. The plugin owns the live TCP server on `127.0.0.1:8765`; the Tauri app connects as a client.

## Build

```powershell
cmake --build plugins\daz3d-bridge\build2 --config Release
```

Output:

```text
plugins\daz3d-bridge\dist\Release\VibeBridgePlugin.dll
```

## Install

Copy `VibeBridgePlugin.dll` into the Daz Studio plugin directory, then restart Daz Studio.

Common path:

```text
C:\Program Files\DAZ 3D\DAZStudio4\plugins\
```

## Protocol

The server accepts newline-delimited JSON.

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

## Implemented Commands

- `get_commands`
- `get_scene_info`
- `list_nodes`
- `get_selected_nodes`
- `select_node`
- `get_cameras`
- `render_preview`
- `load_asset` via `DzContentMgr::openFile`
- `apply_pose` via `DzContentMgr::openFile`
- `capture_viewport` via active `Dz3DViewport::captureImage`
- `import_model` via `DzContentMgr::importFile`

## Honest Unsupported Commands

These commands are registered but currently return explicit errors until the real Daz SDK operation is implemented:

- `export_scene`
