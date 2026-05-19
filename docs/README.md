# DazPilot Docs

> **Disclaimer:** DazPilot is an independent, third-party project and is **not affiliated with, authorized, or endorsed by Daz 3D.** All product names, logos, and brands are property of their respective owners.

Use this index as the starting point for project, runtime, release, and publishing documentation.

| Document | Start here when you need to... |
| --- | --- |
| [Current State](CURRENT_STATE.md) | See what is implemented, verified, and still waiting on live Daz acceptance |
| [Architecture](ARCHITECTURE.md) | Understand the runtime layers, bridge ownership, AI flow, and data sources |
| [Implementation](IMPLEMENTATION.md) | Review completed phases and important implementation files |
| [Release Guide](RELEASE_GUIDE.md) | Tag and publish GitHub releases with Tauri installers |
| [Publishing](PUBLISHING.md) | Prepare GitHub and Daz 3D Marketplace publishing materials |
| [Permissions](PERMISSIONS.md) | Understand the permission and audit model |
| [Agents](AGENTS.md) | Understand the AI agent responsibilities and message flow |

## Quick Runtime Facts

- The Daz Studio plugin owns the bridge server on `127.0.0.1:8765`.
- The Tauri app connects as a TCP client and sends newline-delimited JSON.
- The default AI path is local GGUF through `llama-server.exe`.
- Development mocks are opt-in with environment flags.
- Scene export is deliberately reported as unsupported until a real SDK exporter is implemented.
