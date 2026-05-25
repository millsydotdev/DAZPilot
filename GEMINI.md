# DazPilot Project Instructions

DazPilot is an AI-assisted Daz Studio scene control system consisting of a Tauri desktop app (React/TS/Rust) and a C++ bridge plugin for Daz Studio.

## Workspace Overview

- **Frontend:** React, TypeScript, Vite, Tailwind CSS (located in `src/`).
- **Backend (Tauri):** Rust (located in `src-tauri/`).
- **Daz Bridge:** C++ plugin (located in `plugins/daz3d-bridge/`).
- **Documentation:** Architectural and implementation details are in `docs/`.
- **SDK:** Daz Studio SDK is expected at `thirdparty/DAZStudio4.5+ SDK\` relative to the workspace root (ignored by
git).

## Foundational Mandates

- **Security:** Never log or commit secrets. Protect `.env` files.
- **Context Efficiency:** Use sub-agents for batch tasks or high-volume output. Use `grep_search` and `glob` to minimize file reads.
- **Engineering Standards:** Adhere to existing patterns. Ensure type safety in TypeScript (no `any`) and Rust. Follow C++ conventions for the bridge plugin.

## Architectural Principles

- **Bridge Ownership:** The Daz Studio plugin owns the TCP server (`127.0.0.1:8765`). The Tauri app is a client.
- **Protocol:** Newline-delimited JSON.
- **Main-Thread Execution:** Daz Studio mutations MUST occur on the main GUI thread. The bridge uses a Qt event-based proxy for this.
- **AI Integration:** Action-aware chat. Validate structured actions against schemas before execution.

## Development Workflow

### Useful Commands

- `npm run check`: Run Rust clippy, typecheck, lint, format check, Rust fmt, and tests.
- `npm run dev`: Start the Vite development server.
- `npm run tauri dev`: Start the Tauri application in development mode.
- `npm run plugin:rebuild`: Rebuild the C++ bridge plugin.
- `npm run test`: Run frontend unit tests (Vitest).

### Environment Variables

- `DAZPILOT_DEV_MOCK_BRIDGE=1`: Use mock bridge.
- `DAZPILOT_DEV_MOCK_AI=1`: Use mock AI.
- `DAZPILOT_AI_BACKEND=ollama`: Use Ollama instead of bundled GGUF.

## Coding Style & Conventions

- **TypeScript:** Use functional components and hooks. Prefer `interface` over `type` for object definitions. Use `clsx` and `tailwind-merge` (via `cn` utility) for CSS classes.
- **Rust:** Follow idiomatic Rust patterns. Use `thiserror` for error handling. Use `serde` for JSON serialization.
- **C++:** Adhere to Daz Studio SDK and Qt conventions.

## Testing Standards

- **Frontend:** Use Vitest and React Testing Library. Tests are in `src/` alongside components or in `tests/`.
- **Backend:** Use Rust's built-in testing framework. Tests are in `src-tauri/src/` or `src-tauri/tests/`.
- **Verification:** Always verify changes with `npm run check` before concluding a task.
