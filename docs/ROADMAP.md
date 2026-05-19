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
| Scene export | Returns explicit unsupported response | Waiting on real Daz SDK exporter |
| macOS/Linux bridge | Native bridge plugin is Windows-only | Needs platform-specific plugin builds |
| Live acceptance | Not yet validated against real Daz Studio content | Requires local Daz installation |
| CI bridge tests | Only test against mock bridge | Daz SDK is proprietary, cannot run in CI |

---

## Planned Features

### Short Term

- [ ] **Scene export** — Implement real Daz SDK scene exporter to replace the unsupported response
- [ ] **Live acceptance validation** — Verify all bridge commands against a real Daz Studio session
- [ ] **Viewport capture polish** — Ensure capture paths and UI-thread behavior work reliably
- [ ] **Asset loading coverage** — Validate `.duf`, `.dsf`, pose presets, and content library items

### Medium Term

- [ ] **macOS bridge plugin** — Build `libDazPilotBridge.dylib` for macOS Daz Studio
- [ ] **Linux bridge plugin** — Build `libDazPilotBridge.so` for Linux Daz Studio
- [ ] **Multi-figure operations** — Batch scene operations across multiple figures
- [ ] **Animation timeline** — Enhanced keyframe editing and timeline scrubbing
- [ ] **Render queue** — Queue and manage multiple render jobs
- [ ] **Asset conflict resolver UI** — Visual interface for resolving asset conflicts

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
| 0.2.0 | Scene export, live acceptance hardening |
| 0.3.0 | macOS/Linux bridge support |
| 1.0.0 | Stable API, full platform support, marketplace readiness |

Version numbers and timelines are aspirational and subject to change.
