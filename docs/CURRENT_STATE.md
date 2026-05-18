# Current State

Updated: May 2026

## What Is Real Now

- The app no longer starts a canned TCP server on port `8765`.
- The Daz plugin now listens on `127.0.0.1:8765`.
- The Tauri bridge is client-only and speaks newline-delimited JSON.
- Bridge commands are validated against a registered schema before dispatch.
- `check_connection_status()` returns `connected` or `disconnected`; production `mock` status is gone.
- Dev bridge mock is explicit through `DazPilot_DEV_MOCK_BRIDGE=1`.
- Local GGUF through bundled `llama-server.exe` is the default AI path.
- Ollama is optional through `DazPilot_AI_BACKEND=ollama`.
- Mock AI is explicit through `DazPilot_DEV_MOCK_AI=1`.
- `sdk_indexer` is the active SDK source of truth; legacy `sdk_rag` is no longer wired into the runtime.
- SDK indexing is recursive, line-aware, and persisted to SQLite.
- Asset scanning reads available Daz JSON metadata and persists discovered assets.
- Fake import/export success responses were removed. Export currently fails honestly until implemented.

## Verified

```powershell
npm run check
cargo test
cmake --build plugins\daz3d-bridge\build2 --config Release
```

All passed after the real-project implementation pass.

## Still Limited

- Plugin-side scene export currently returns an explicit unsupported error.
- Asset loading and pose application use `DzContentMgr::openFile`; live Daz acceptance still needs to verify behavior across `.duf`, `.dsf`, and pose presets.
- Model import uses `DzContentMgr::importFile`; live Daz acceptance still needs to verify format coverage.
- Viewport capture uses the active `Dz3DViewport::captureImage`; live Daz acceptance still needs to verify the output path and UI-thread behavior.
- Manual Daz Studio acceptance still requires installing the freshly built DLL into Daz Studio and testing against a live scene.

## Important Files

- `src-tauri/src/mcp_client.rs`: client-only bridge, command schema, dev mock gate.
- `plugins/daz3d-bridge/DazPilotBridge.cpp`: plugin TCP server and Daz SDK command dispatch.
- `src-tauri/src/ai_action.rs`: structured action planning/validation/execution.
- `src-tauri/src/sdk_indexer.rs`: recursive SDK header indexer.
- `src-tauri/src/library_scanner.rs`: asset metadata scanner.
- `src-tauri/src/advanced.rs`: import/export route to bridge or return honest failure.
