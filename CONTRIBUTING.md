# Contributing to DazPilot

## Getting Started

1. Fork the repository
2. Ensure you have: Node.js 20+, Rust 1.70+, CMake 3.20+
3. Run `npm install` to install frontend dependencies
4. Run `npm run dev` for development

## Development Workflow

- Frontend: React + TypeScript + Zustand stores in `src/`
- Backend: Rust + Tauri 2 commands in `src-tauri/src/`
- Bridge: Custom C++ Daz Studio plugin — source at `plugins/daz3d-bridge/`

## Git Hooks

This project uses [Husky](https://typicode.github.io/husky/) and [lint-staged](https://github.com/lint-staged/lint-staged) to enforce code quality automatically.

### Pre-commit (runs on `git commit`)

Runs `lint-staged` on staged files only:

- `*.{ts,tsx}` — ESLint fix + Prettier format
- `*.{json,css,md,yml,yaml}` — Prettier format
- `*.md` — markdownlint fix

### Pre-push (runs on `git push`)

Runs the full validation pipeline:

```
npm run check
```

This executes: typecheck → lint → format check → tests. If any step fails, the push is blocked.

### Skipping hooks

In rare cases, you can skip hooks with `--no-verify`:

```bash
git commit --no-verify    # Skip pre-commit
git push --no-verify      # Skip pre-push
```

Use this sparingly — CI will still catch failures.

## Testing

```bash
npm test              # Run frontend tests
cargo test            # Run Rust backend tests
npm run check         # Full pipeline: typecheck + lint + format + test
```

## Code Style

- TypeScript: Strict mode, Prettier formatting (2 spaces, single quotes)
- Rust: rustfmt with standard settings
- C++: Follow existing patterns in the bridge plugin at `plugins/daz3d-bridge/`

## Pull Requests

- Keep changes focused and well-described
- Add tests for new functionality
- Verify `npm run check` passes before submitting
