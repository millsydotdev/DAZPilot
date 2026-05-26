# DazPilot Documentation

> **Disclaimer:** DazPilot is an independent, third-party project and is **not affiliated with, authorized, or endorsed by Daz 3D.** All product names, logos, and brands are property of their respective owners.

Use this index as the starting point for project, runtime, release, and publishing documentation.

---

## Start Here

| I want to... | Document |
| --- | --- |
| Set up the project for the first time | [Getting Started](GETTING_STARTED.md) |
| Understand how the app works at runtime | [Architecture](ARCHITECTURE.md) |
| See what is built and verified right now | [Current State](CURRENT_STATE.md) |
| Review completed implementation phases | [Implementation](IMPLEMENTATION.md) |
| Understand the AI agent hierarchy (14 agents) | [Agents](AGENTS.md) |
| Learn about the permission model | [Permissions](PERMISSIONS.md) |
| Tag and publish a release | [Release Guide](RELEASE_GUIDE.md) |
| Prepare GitHub or Daz Marketplace publishing | [Publishing](PUBLISHING.md) |
| Run acceptance tests | [Acceptance](ACCEPTANCE.md) |
| See what is planned next | [Roadmap](ROADMAP.md) |

---

## Quick Reference

### Key Commands

```powershell
npm install               # Install frontend dependencies
npm run dev               # Start Vite dev server
npm run check             # Rust clippy + typecheck + lint + format check + Rust fmt + test
npm run tauri build       # Build production Tauri app
npm run plugin:build      # Build bridge plugin (plugins/daz3d-bridge/)
npm test                  # Frontend tests
cargo test                # Rust backend tests
# C++ bridge plugin tests (doctest) — at plugins/daz3d-bridge/tests/
```

### Key Environment Variables

| Variable | Effect |
| --- | --- |
| `DAZPILOT_DEV_MOCK_BRIDGE=1` | Enable the bridge mock for development |
| `DAZPILOT_DEV_MOCK_AI=1` | Enable the AI mock for development |
| `DAZPILOT_AI_BACKEND=ollama` | Use Ollama instead of bundled local GGUF |
| `DAZ_SDK_PATH=...` | Override the SDK include path |

### Quick Runtime Facts

- The Daz Studio plugin owns the bridge server on `127.0.0.1:8765`.
- The Tauri app connects as a TCP client and sends newline-delimited JSON.
- The default AI path is local GGUF through `llama-server.exe`.
- Development mocks are opt-in with environment flags.
- Scene export is implemented via C++ bridge DzExportMgr with DazScript fallback.

---

## All Documents

| Document | Description |
| --- | --- |
| [Getting Started](GETTING_STARTED.md) | Prerequisites, step-by-step setup, first build, and troubleshooting |
| [Architecture](ARCHITECTURE.md) | Runtime layers, bridge ownership, AI flow, knowledge sources, session summaries |
| [Current State](CURRENT_STATE.md) | Implementation snapshot, verification status, and acceptance notes |
| [Implementation](IMPLEMENTATION.md) | 20-phase completion status and important files |
| [Agents](AGENTS.md) | 14-agent hierarchy, registry, orchestration, sub-agents, Tauri commands, and UI components |
| [Permissions](PERMISSIONS.md) | Database-driven permission model, roles, audit logging, and default policies |
| [Release Guide](RELEASE_GUIDE.md) | Tagging, GitHub Actions workflows, signing, and bridge DLL releases |
| [Publishing](PUBLISHING.md) | GitHub publication and Daz 3D Marketplace submission guide |
| [Acceptance](ACCEPTANCE.md) | Automated CI acceptance and manual live Daz Studio test checklists |
| [Roadmap](ROADMAP.md) | Planned features, known limitations, and contribution opportunities |
| [Third-Party](THIRD-PARTY.md) | Overview of third-party libraries, licenses, and usage status |

---

## Project Structure

```text
DazPilot/
├── src/                          # React + TypeScript frontend
│   ├── components/               # UI components (24 atomic + feature panels)
│   ├── store/                    # Zustand state management (17 stores)
│   ├── hooks/                    # Custom React hooks
│   ├── utils/                    # Utilities
│   └── types/                    # TypeScript type definitions
├── src-tauri/                    # Rust + Tauri 2 backend
│   ├── src/                      # Backend source (AI, bridge, agents, database)
│   │   ├── agents/               # 14-agent hierarchy (registry, orchestrator, sub-agents)
│   │   │   └── sub_agents/       # 7 sub-agent modules (pose, timeline, lighting, etc.)
│   ├── resources/                # Bundled binaries (DLL, llama-server, models)
│   └── capabilities/             # Tauri security capabilities
├── plugins/
│   └── daz3d-bridge/             # C++ bridge plugin source
├── docs/                         # Documentation (you are here)
├── plan/                         # Development plan
├── scripts/                      # Build and dev scripts
└── .github/workflows/            # CI/CD workflows
```
