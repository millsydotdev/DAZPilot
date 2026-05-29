# Roadmap

Updated: May 2026

This document tracks planned features, known limitations, and areas where contributions are welcome.

---

## Current Status

All 21 planned implementation phases are complete. The project is in **acceptance and hardening** — remaining work is live Daz Studio validation and release polish.

See [CURRENT_STATE.md](CURRENT_STATE.md) for the implementation snapshot.

---

## Known Limitations

| Area | Limitation | Status |
| --- | --- | --- |
| Scene export | Implemented in C++ bridge via DzExportMgr + DazScript fallback | Needs live validation |
| macOS bridge | Source in `plugins/daz3d-bridge/` | Needs macOS Daz SDK to compile |
| Linux bridge | Source in `plugins/daz3d-bridge/` | Wine or headless strategy needed |
| Live acceptance | Not yet validated against real Daz Studio content | Requires local Daz installation |
| CI bridge tests | Only test against mock bridge | Daz SDK is proprietary, cannot run in CI |
| Agent tuning | Sub-agent prompts not yet validated against real user input | Needs real usage data |

---

## Planned Features

### Short Term

- [ ] **Live acceptance validation** — Verify all bridge commands against a real Daz Studio session
- [ ] **Scene export live test** — Verify C++ DzExportMgr exporter + DazScript fallback work end-to-end
- [ ] **Schema parity test** — Rust test auto-checks C++ bridge commands match `mcp_client.rs`
- [ ] **Sub-agent hierarchy** — 7 sub-agents in 3-level tree with registry, orchestration, and delegation
- [ ] **Agent prompt tuning** — Refine sub-agent keyword matching and response formatting
- [ ] **Viewport capture polish** — Ensure capture paths and UI-thread behavior work reliably
- [ ] **Asset loading coverage** — Validate `.duf`, `.dsf`, pose presets, and content library items
- [ ] **UI Professionalization** — Complete visual redesign with professional 3D software aesthetics, refined design system, and workflow-oriented interface improvements

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
- [ ] **Multi-figure operations** — Batch scene operations across multiple figures
- [ ] **Animation timeline** — Enhanced keyframe editing and timeline scrubbing
- [ ] **Render queue** — Queue and manage multiple render jobs
- [x] **Automated conflict resolution pipeline** — Integrated detection, analysis, and auto-fix systems (asset_fixer, vision_service, conflict_resolution agent, pre-load checks) that resolve shell zone, morph ID, UV set, and compatibility conflicts without user intervention
- [x] **Preset management system** — Persist and restore scene configurations (lighting, camera, figure arrangements)
- [ ] **Agent analytics dashboard** — Track execution rates, success rates, and delegation patterns per agent
- [ ] **Custom sub-agent plugins** — Allow users to register custom sub-agents from the UI
- [ ] **Asset recommendation engine** — Suggest complementary assets based on scene context and user intent
- [ ] **UI Professionalization Phase 1** — Updated design system with professional color scheme, refined typography, and improved component styling

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
| Frontend UI polish | Low | React + Tailwind, well-documented component patterns |
| Documentation | Low | Markdown files in `docs/`, existing conventions |
| Test coverage | Medium | Vitest for frontend, cargo test for backend |
| Sub-agent development | Medium | Add new sub-agents in `agents/sub_agents/`, register in tree |
| Agent UI components | Medium | React components for agent management (tree, detail, tester) |
| Bridge command additions | Medium | Requires Daz SDK access and C++ knowledge |
| Body/material opacity micro-tools | Medium | `SetBodyOpacity`, `SetSurfaceOpacity`, `ShowAnatomy`, `PlaceAssetInside` — each is an independent bridge command + action type |
| AI prompt engineering | Medium | Action planning and validation logic in Rust |
| Platform bridge ports | High | Needs macOS/Linux Daz SDK access |

See [CONTRIBUTING.md](../CONTRIBUTING.md) for setup instructions and code style guidelines.

---

## Release Plan

| Version | Focus |
| --- | --- |
| 0.5.x (current) | Acceptance validation, schema parity, release hardening |
| 0.6.0 | Live acceptance validation complete, macOS bridge (.dylib) |
| 0.7.0 | Linux bridge strategy |
| 1.0.0 | Full platform support (Win/Mac/Linux), stable API |

Version numbers and timelines are aspirational and subject to change.
