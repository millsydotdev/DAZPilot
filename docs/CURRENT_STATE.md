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
| Dev bridge mock | Explicit through `DAZPILOT_DEV_MOCK_BRIDGE=1` |
| Default AI | Bundled local GGUF through `llama-server.exe` |
| Optional AI | Ollama through `DAZPILOT_AI_BACKEND=ollama` |
| Dev AI mock | Explicit through `DAZPILOT_DEV_MOCK_AI=1` |
| SDK knowledge | `sdk_indexer` is the active source of truth |
| SDK persistence | Recursive, line-aware indexing persisted to SQLite |
| Asset knowledge | Daz JSON metadata is read when available and persisted |
| Import honesty | Fake success responses were removed; real Daz SDK import is used |
| Scene export | Implemented via C++ bridge `DzExportMgr/DzExporter::writeFile` with material/animation/texture settings + DazScript fallback |
| Agent hierarchy | 14 agents in 3-level tree with registry, orchestration, and delegation |
| Sub-agent system | 7 sub-agents under 4 parent agents (animation, render, asset_selection, scene_composer) |
| Agent management | 8 Tauri commands for agent querying, testing, registration, and unregistration |
| Agent UI | 3 React components: AgentTreeView, AgentDetailPanel, AgentTester |
| Conflict resolution pipeline | Integrated auto-fix for shell zone, morph ID, and UV set conflicts via `asset_fixer`, `conflict_resolution` agent, `vision_service`, and pre-load checks |
| Pre-load conflict detection | `load_asset_in_daz` auto-runs `check_before_load` and warns about potential conflicts |
| `get_geoshells` integration | Bridge command used for accurate geoshell detection instead of substring matching |
| Agent intent parsing | `conflict_resolution` agent respects input intent (scan/fix/status) and uses heuristic prefix detection |

`check_connection_status()` now reports `connected` or `disconnected`; the old production `mock` status is gone.

## Verified

```powershell
npm run check
cargo test
```

Both pass in the current workspace. `npm run check` completes with lint warnings for existing `no-explicit-any` usage, but no lint errors. 119 Rust tests pass (up from 115).

## Acceptance

**Automated:** `npm run acceptance` runs workspace checks plus `cargo test acceptance_` against the dev mock bridge.

**Manual:** See [ACCEPTANCE.md](ACCEPTANCE.md) for the live Daz Studio checklist (DLL install, asset load, viewport capture, morph/light/render commands).

Remaining product gaps:

- Live end-to-end validation on Windows (build bridge DLL, connect to real Daz Studio, test all 30+ commands).
- Cross-platform port (macOS bridge .dylib, Linux strategy).
- Live verification that schema-covered commands behave correctly in a real Daz Studio session.

## Important Files

| File | Why it matters |
| --- | --- |
| `src-tauri/src/mcp_client.rs` | Client-only bridge, command schema, dev mock gate |
| `plugins/daz3d-bridge/DazPilotBridgePlugin.cpp` | Plugin TCP server and Daz SDK command dispatch |
| `src-tauri/src/ai_action.rs` | Structured action planning, validation, and execution |
| `src-tauri/src/sdk_indexer.rs` | Recursive SDK header indexer |
| `src-tauri/src/library_scanner.rs` | Asset metadata scanner |
| `src-tauri/src/advanced.rs` | Import/export routes (bridge export + DazScript fallback) |
| `src-tauri/src/agents/mod.rs` | Agent types, registry initialization, handler registration |
| `src-tauri/src/agents/registry.rs` | AgentRegistry tree, lookup, capability matching, global singleton |
| `src-tauri/src/agents/orchestrator.rs` | Delegation (parent→child), cycle detection, result aggregation |
| `src-tauri/src/agents/task_planner.rs` | Orchestrator agent — delegates to children based on capability matching |
| `src-tauri/src/agents/sub_agents/` | 7 sub-agent modules (pose, timeline, lighting, camera, morph, material, export) |
| `src-tauri/src/asset_fixer.rs` | File-level conflict scanner and auto-fix for MaterialZone, MorphId, UVSet |
| `src-tauri/src/agents/conflict_resolution.rs` | Intent-aware conflict resolution agent with heuristic prefix detection and get_geoshells integration |
| `src-tauri/src/vision_service.rs` | Scene-level conflict detection using bridge commands |
| `src/components/agents/AgentTreeView.tsx` | Frontend agent hierarchy visualization |
| `src/components/agents/AgentDetailPanel.tsx` | Frontend agent detail display |
| `src/components/agents/AgentTester.tsx` | Frontend agent testing interface |
