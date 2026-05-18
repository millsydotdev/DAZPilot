# Current State

Updated: May 2026

## Snapshot

All planned implementation phases are complete. Remaining work is acceptance validation against a live Daz Studio installation and release hardening.

## What Is Real Now

| Area | Status |
| --- | --- |
| Bridge ownership | The Daz plugin listens on `127.0.0.1:8765`; Tauri is client-only |
| Bridge protocol | Newline-delimited JSON with registered command validation |
| Production mocks | Removed from the default runtime path |
| Dev bridge mock | Explicit through `DazPilot_DEV_MOCK_BRIDGE=1` |
| Default AI | Bundled local GGUF through `llama-server.exe` |
| Optional AI | Ollama through `DazPilot_AI_BACKEND=ollama` |
| Dev AI mock | Explicit through `DazPilot_DEV_MOCK_AI=1` |
| SDK knowledge | `sdk_indexer` is the active source of truth |
| SDK persistence | Recursive, line-aware indexing persisted to SQLite |
| Asset knowledge | Daz JSON metadata is read when available and persisted |
| Import/export honesty | Fake success responses were removed |

`check_connection_status()` now reports `connected` or `disconnected`; the old production `mock` status is gone.

Scene export currently returns an explicit unsupported response until a real Daz SDK exporter is prioritized.

## Verified

```powershell
npm run check
cargo test
```

Both pass in the current workspace. `npm run check` completes with lint warnings for existing `no-explicit-any` usage, but no lint errors.

## Acceptance Follow-Up

These items require Daz Studio, the installed bridge DLL, and representative local content:

- Install the freshly built DLL into Daz Studio and test against a live scene.
- Verify asset loading and pose application across `.duf`, `.dsf`, and pose presets.
- Verify model import format coverage through `DzContentMgr::importFile`.
- Verify viewport capture output paths and UI-thread behavior through `Dz3DViewport::captureImage`.
- Implement a real plugin-side scene exporter when export becomes a prioritized product feature.

## Important Files

| File | Why it matters |
| --- | --- |
| `src-tauri/src/mcp_client.rs` | Client-only bridge, command schema, dev mock gate |
| `plugins/daz3d-bridge/DazPilotBridge.cpp` | Plugin TCP server and Daz SDK command dispatch |
| `src-tauri/src/ai_action.rs` | Structured action planning, validation, and execution |
| `src-tauri/src/sdk_indexer.rs` | Recursive SDK header indexer |
| `src-tauri/src/library_scanner.rs` | Asset metadata scanner |
| `src-tauri/src/advanced.rs` | Import/export routes and honest unsupported responses |
