# DazPilot Bridge Plugin

C++ Daz Studio bridge plugin. The plugin owns the live TCP server on `127.0.0.1:8765`; the Tauri app connects as a client.

## Build

From the repository root:

```powershell
npm run plugin:rebuild
```

Or directly with CMake:

```powershell
cmake -S plugins\daz3d-bridge -B plugins\daz3d-bridge\build
cmake --build plugins\daz3d-bridge\build --config Release
```

Output:

```text
plugins\daz3d-bridge\dist\Release\
```

## Install

Copy the release DLL into the Daz Studio plugin directory, then restart Daz Studio.

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

| Command | Notes |
| --- | --- |
| `get_commands` | Lists registered bridge commands |
| `get_scene_info` | Reads scene summary information |
| `list_nodes` | Lists scene nodes |
| `get_selected_nodes` | Reads selected scene nodes |
| `select_node` | Selects a scene node |
| `get_cameras` | Lists cameras |
| `render_preview` | Runs preview render path |
| `load_asset` | Uses `DzContentMgr::openFile` |
| `apply_pose` | Uses `DzContentMgr::openFile` |
| `capture_viewport` | Uses active `Dz3DViewport::captureImage` |
| `import_model` | Uses `DzContentMgr::importFile` |
| `export_scene` | Uses the Daz export pipeline with configurable export settings |

## Live Validation

Scene export is implemented in the plugin and has a DazScript fallback in the app. Validate the exporter and fallback with installed Daz content before release.
