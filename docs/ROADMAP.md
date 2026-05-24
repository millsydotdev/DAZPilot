# Roadmap

Updated: May 2026

This document tracks planned features, known limitations, and areas where contributions are welcome.

---

## Current Status

All 20 planned implementation phases are complete. The project is in **acceptance and hardening** — remaining work is live Daz Studio validation and release polish.

See [CURRENT_STATE.md](CURRENT_STATE.md) for the implementation snapshot.

---

## Known Limitations

| Area | Limitation | Status |
| --- | --- | --- |
| Scene export | Implemented in C++ bridge via DzExportMgr + DazScript fallback | Needs live validation |
| macOS bridge | CMake/apple branches exist, untested | Needs macOS Daz SDK to compile |
| Linux bridge | CMake/linux branches exist, no Daz Studio on Linux | Wine or headless strategy needed |
| Live acceptance | Not yet validated against real Daz Studio content | Requires local Daz installation |
| CI bridge tests | Only test against mock bridge | Daz SDK is proprietary, cannot run in CI |

---

## Planned Features

### Short Term

- [ ] **Live acceptance validation** — Verify all bridge commands against a real Daz Studio session
- [ ] **Scene export live test** — Verify C++ DzExportMgr exporter + DazScript fallback work end-to-end
- [ ] **Schema parity test** — Auto-check C++ bridge commands match Rust mcp_client.rs schema
- [ ] **Viewport capture polish** — Ensure capture paths and UI-thread behavior work reliably
- [ ] **Asset loading coverage** — Validate `.duf`, `.dsf`, pose presets, and content library items

### Medium Term

- [ ] **macOS bridge plugin** — Build `libDazPilotBridge.dylib` for macOS Daz Studio (CMake scaffolding exists)
- [ ] **Linux bridge plugin** — Build `libDazPilotBridge.so` or Wine DLL for Linux (strategy TBD)
- [ ] **Multi-figure operations** — Batch scene operations across multiple figures
- [ ] **Animation timeline** — Enhanced keyframe editing and timeline scrubbing
- [ ] **Render queue** — Queue and manage multiple render jobs
- [ ] **Asset conflict resolver UI** — Visual interface for resolving asset conflicts
- [ ] **Preset management system** — Save and reuse common scene configurations (lighting, camera, figure arrangements)
- [ ] **Asset recommendation engine** — Suggest complementary assets based on scene context and user intent

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
| Bridge command additions | Medium | Requires Daz SDK access and C++ knowledge |
| AI prompt engineering | Medium | Action planning and validation logic in Rust |
| Platform bridge ports | High | Needs macOS/Linux Daz SDK access |

See [CONTRIBUTING.md](../CONTRIBUTING.md) for setup instructions and code style guidelines.

---

## Release Plan

| Version | Focus |
| --- | --- |
| 0.1.0 | Initial release — core features, Windows bridge, local AI |
| 0.2.0 | Live acceptance validation, schema parity, polish |
| 0.3.0 | macOS bridge (.dylib) + Linux strategy |
| 1.0.0 | Full platform support (Win/Mac/Linux), stable API |

Version numbers and timelines are aspirational and subject to change.
