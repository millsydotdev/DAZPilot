# Contributing to DazPilot

## Getting Started

1. Fork the repository
2. Ensure you have: Node.js 20+, Rust 1.70+, CMake 3.20+
3. Run `npm install` to install frontend dependencies
4. Run `npm run dev` for development

## Development Workflow

- Frontend: React + TypeScript + Zustand stores in `src/`
- Backend: Rust + Tauri 2 commands in `src-tauri/src/`
- Bridge: C++ Daz Studio plugin in `plugins/daz3d-bridge/`

## Testing

```bash
npm test          # Run frontend tests
cargo test        # Run Rust backend tests
npm run check     # Full pipeline: typecheck + lint + format + test
```

## Code Style

- TypeScript: Strict mode, Prettier formatting (2 spaces, single quotes)
- Rust: rustfmt with standard settings
- C++: Follow existing patterns in the bridge plugin

## Pull Requests

- Keep changes focused and well-described
- Add tests for new functionality
- Verify the app builds and runs before submitting
