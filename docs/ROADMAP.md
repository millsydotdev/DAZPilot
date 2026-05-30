# Roadmap

Updated: May 2026

This document tracks planned features, known limitations, and areas where contributions are welcome.

---

## Current Status

All 21 planned implementation phases are complete. The project is in **acceptance and hardening** — remaining work is live Daz Studio validation and release polish.

See [CURRENT_STATE.md](CURRENT_STATE.md) for the implementation snapshot and [PLATFORM_MATRIX.md](PLATFORM_MATRIX.md) for OS/arch release targets.

---

## Known Limitations

| Area | Limitation | Status |
| --- | --- | --- |
| Scene export | Implemented in C++ bridge via DzExportMgr + DazScript fallback | Needs live validation |
| macOS bridge | Source in `plugins/daz3d-bridge/` | Needs macOS Daz SDK to compile |
| Linux bridge | Source in `plugins/daz3d-bridge/` | Wine or headless strategy needed |
| Live acceptance | Not yet validated against real Daz Studio content | Requires local Daz installation |
| CI bridge tests | Unit tests + schema parity; no live Daz in CI | Daz SDK is proprietary |
| Agent tuning | Sub-agent prompts improved; needs real usage validation | Needs live user feedback |

---

## Planned Features

### Short Term

- [ ] **Live acceptance validation** — Verify all bridge commands against a real Daz Studio session
- [ ] **Scene export live test** — Verify C++ DzExportMgr exporter + DazScript fallback work end-to-end
- [x] **Schema parity test** — Rust test + `bridge_commands.manifest` auto-checks C++ ↔ `mcp_client.rs` (CI enforced)
- [x] **Sub-agent hierarchy** — 7 sub-agents in 3-level tree with registry, orchestration, and delegation
- [x] **Platform/arch matrix** — Win/Mac/Linux × x64/ARM64 app builds; Win x64/ARM64 plugin release assets ([PLATFORM_MATRIX.md](PLATFORM_MATRIX.md))
- [x] **Agent prompt tuning** — Synonym-aware capability matching, plural normalization, agent-type/description scoring
- [x] **Viewport capture polish** — UI-thread capture via `CaptureActiveViewport`, temp-path handling, stream/base64 mode
- [ ] **Asset loading coverage** — Validate `.duf`, `.dsf`, pose presets, and content library items (requires live Daz)
- [x] **UI Professionalization** — Design system (`design-system.css`, tokens), PanelShell layout, professional sidebar/title bar, workflow-oriented tabs

### Medium Term

- [x] **Body/material opacity systems** — Individual micro-tools for transparency control:
  - `SetBodyOpacity` action type + `set_body_opacity` bridge command — uniform opacity across all body surfaces
  - `SetSurfaceOpacity` action type + `set_surface_opacity` bridge command — opacity targeting specific material surfaces by name/pattern
  - `GetInternalSurfaces` action type + `get_internal_surfaces` bridge command — discover skeleton/anatomy surface names
  - `ShowAnatomy` action type + `show_anatomy` bridge command — make internal skeleton surfaces fully opaque in one call
- [x] **Interior placement system** — `PlaceAssetInside` action type + `place_asset_inside` bridge command — loads asset and parents it inside a figure at torso position
  - AI composes these independently: `SetBodyOpacity(0.15)` → `SetSurfaceOpacity("Stomach", 0.02)` → `ShowAnatomy` → `PlaceAssetInside("alien")`
- [ ] **macOS bridge plugin** — Build from `plugins/daz3d-bridge/` on macOS
- [ ] **Linux bridge plugin** — Build from `plugins/daz3d-bridge/` for Wine/headless
- [x] **Multi-figure operations** — Scene composer detects multi-figure intent and duplicates `add_figure` steps; batch morph/export commands available
- [x] **Animation timeline** — Keyframe editor, timeline scrubbing, transport controls in viewport panel
- [x] **Render queue** — `queue_render` / `cancel_render` bridge commands + Render Queue panel in viewport
- [x] **Automated conflict resolution pipeline** — Integrated detection, analysis, and auto-fix systems (asset_fixer, vision_service, conflict_resolution agent, pre-load checks) that resolve shell zone, morph ID, UV set, and compatibility conflicts without user intervention
- [x] **Preset management system** — Persist and restore scene configurations via SQLite + Scene Presets tab in Scene panel
- [x] **Agent analytics dashboard** — Execution rates, success rates, response times per agent (`AgentAnalytics` + `get_agent_metrics`)
- [x] **Custom sub-agent plugins** — `register_sub_agent` Tauri command + registration form in Agents panel
- [x] **Asset recommendation engine** — `recommend_scene_assets` command + Suggest button in Asset Browser
- [x] **UI Professionalization Phase 1** — Updated design system with professional color scheme, refined typography, and improved component styling

### Long Term

- [ ] **Plugin marketplace** — Community-contributed bridge commands and AI agents
- [ ] **Cloud AI option** — Optional cloud-based AI for users without capable local hardware
- [ ] **Collaborative sessions** — Multiple users working on the same scene
- [ ] **Custom AI training** — Fine-tune models on user's Daz workflow patterns

---

## Contribution Opportunities

These areas are well-suited for external contributions:

| Area | Difficulty | Notes |
| --- | --- | --- |
| Live acceptance testing | Medium | Requires local Daz Studio + bridge DLL |
| macOS/Linux bridge ports | High | Needs platform Daz SDK access |
| Frontend UI polish | Low | React + Tailwind, well-documented component patterns |
| Documentation | Low | Markdown files in `docs/`, existing conventions |
| Test coverage | Medium | Vitest for frontend, cargo test for backend |
| Sub-agent development | Medium | Add new sub-agents in `agents/sub_agents/`, register in tree |
| Bridge command additions | Medium | Requires Daz SDK access and C++ knowledge |
| AI prompt engineering | Medium | Action planning and validation logic in Rust |

See [CONTRIBUTING.md](../CONTRIBUTING.md) for setup instructions and code style guidelines.

---

## Release Plan

| Version | Focus |
| --- | --- |
| 0.5.x (current) | Schema parity CI, platform matrix, UI professionalization, agent analytics |
| 0.6.0 | Live acceptance validation complete, macOS bridge (.dylib) |
| 0.7.0 | Linux bridge strategy |
| 1.0.0 | Full platform support (Win/Mac/Linux), stable API |

Version numbers and timelines are aspirational and subject to change.
