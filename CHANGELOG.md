# Changelog

All notable changes to DazPilot will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.5] - 2026-05-30

DazPilot is an AI-assisted desktop companion for DAZ Studio. This release adds **agent tooling in the UI**, a **cleaner app shell**, **reliable Windows bridge builds**, and **fixed GitHub release publishing**.

### Highlights

- **Agents panel** — browse the agent tree, test routing, register custom sub-agents.
- **Daz ↔ app handoff** — send viewport context from the bridge pane to the desktop app (`127.0.0.1:8766`).
- **Multi-platform installers** — Windows (x64 / ARM64), macOS (Apple Silicon / Intel), Linux (AppImage, deb, rpm).

### Added

- Agents panel with custom sub-agent form, render queue panel, and consolidated scripts tab.
- Bridge command manifest (`bridge_commands.manifest`) kept in sync with Rust MCP schemas.
- Platform matrix documentation (`docs/PLATFORM_MATRIX.md`).
- App inbox handoff from the Daz Studio pane (viewport context to desktop app on port 8766).
- Scene presets in the Scene panel; shared `PanelShell` UI component.

### Changed

- UI shell consolidation and learning dashboard styling.
- Release CI stages only installable bundle files; macOS updater archives disambiguated per CPU.
- Windows bridge plugin release is **x64 only** (DAZ SDK limitation; ARM64 Windows uses the same DLL).

### Fixed

- DazPilot bridge pane compile errors (Qt4-compatible pane; removed incomplete ImGui integration).
- Release workflows locate MSVC-built `DazPilotBridge.dll` under `dist/` / `dist/Release/`.
- GitHub release upload no longer attaches empty `target/` directories or duplicate macOS asset names.
- App/installer version bumped to **0.5.5** so release asset names match the git tag.
- Removed obsolete CI updater JSON upload path and Node 24 Actions override warnings.

### Install

**Desktop:** download the installer for your OS/arch from this release (MSI/NSIS on Windows, DMG on macOS, AppImage/deb/rpm on Linux). Auto-update uses `latest.json` on the release page.

**Bridge plugin:** install from [plugin-v0.5.5](https://github.com/millsydotdev/DAZPilot/releases/tag/plugin-v0.5.5) — copy `DazPilotBridge-x64.dll` into Daz’s plugins folder and restart Daz. Bridge listens on `127.0.0.1:8765`.

Full release notes: [docs/releases/v0.5.5.md](docs/releases/v0.5.5.md)

## [0.1.1] - 2026-05-19

### Added

- Husky + lint-staged git hooks: pre-commit runs lint-staged on staged files, pre-push runs full validation pipeline
- Documentation overhaul: badges, table of contents, features section, quick start guide
- `docs/GETTING_STARTED.md`: step-by-step setup with prerequisites and troubleshooting
- `docs/ROADMAP.md`: planned features, known limitations, and contribution areas
- CI now enforces lint and format checks alongside typecheck and tests
- `process:default` Tauri capability for auto-relaunch after updates
- `.orchids/` added to `.gitignore`

### Fixed

- Rust compiler warnings: unused imports in `asset_selection.rs` and `vector_store.rs`, unnecessary `mut` in `asset_selection.rs` and `library_scanner.rs`
- Auto-update relaunch: added missing `process:default` capability so the app restarts after installing an update
- Dynamic version: `get_app_version()` now reads from `CARGO_PKG_VERSION` instead of hardcoded string

### Changed

- `CONTRIBUTING.md`: added git hooks documentation
- `CHANGELOG.md`: updated to Keep a Changelog standard with comparison links

## [0.1.0] - 2024-01-01

### Added

- Initial DazPilot application with Tauri 2 + React + TypeScript
- Daz Studio bridge plugin (DazPilotBridge) for TCP communication on `127.0.0.1:8765`
- AI Scripting Co-Pilot with multi-provider support (OpenAI, Anthropic, Gemini, Ollama, Local GGUF)
- Local GGUF AI backend via bundled `llama-server.exe`
- Viewport sync with Daz Studio (screenshot-based polling)
- Face tracking Live Link via MediaPipe
- Asset browser with recursive library scanning and SQLite indexing
- SDK header indexer for Daz API knowledge base
- Scene management and animation controls
- Physics simulation (dForce) integration
- Script approval system for safe AI execution
- Database-backed settings and permission persistence
- 25 atomic UI component design system
- 17 Zustand state management stores
- 8 specialized AI agent modules
- First-launch setup wizard
- GitHub Actions CI/CD for multi-platform builds (Windows, macOS, Linux)
- Separate release workflows for app bundles and bridge plugin
- Environment variable configuration for dev mocks and AI backend selection

[Unreleased]: https://github.com/millsydotdev/DazPilot/compare/v0.5.5...HEAD
[0.5.5]: https://github.com/millsydotdev/DazPilot/compare/v0.1.1...v0.5.5
[0.1.1]: https://github.com/millsydotdev/DazPilot/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/millsydotdev/DazPilot/releases/tag/v0.1.0
