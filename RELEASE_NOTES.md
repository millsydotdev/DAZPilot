# DAZStudio MCP 1.0.0

## Features
- Core MCP integration for DAZ Studio.
- JSON‑based command manifest.
- Export options handling.
- Logging system with configurable verbosity.
- Cross‑platform build (Windows) with CMake.

## Bug Fixes & Improvements
- Fixed memory leaks in `Log` module.
- Updated third‑party dependencies (utfcpp, glm, simdjson).
- Improved error handling for missing SDK during build.

## Known Issues
- Plugin may not load on DAZ Studio 4.20+ when built with VS 2019+ without the full SDK (documented in README).

This is the initial stable release of the DAZStudio MCP plugin.
