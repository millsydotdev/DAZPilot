# Platform & Architecture Matrix

Updated: May 2026

DazPilot ships as a **Tauri desktop app** plus a **Daz Studio bridge plugin**. The app runs on Windows, macOS, and Linux (x64 and ARM64 where noted). The bridge plugin follows Daz Studio's platform support.

## App (Tauri) — CI build matrix

| OS | Architecture | Rust target | Bundles | CI runner |
| --- | --- | --- | --- | --- |
| Windows | x64 | `x86_64-pc-windows-msvc` | NSIS, MSI | `windows-latest` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` | NSIS, MSI | `windows-latest` |
| macOS | ARM64 (Apple Silicon) | `aarch64-apple-darwin` | `.app`, DMG | `macos-latest` |
| macOS | x64 (Intel) | `x86_64-apple-darwin` | `.app`, DMG | `macos-latest` |
| Linux | x64 | `x86_64-unknown-linux-gnu` | AppImage, deb, rpm | `ubuntu-22.04` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | AppImage, deb, rpm | `ubuntu-24.04-arm` |

Updater keys in `latest.json` (Tauri v2):

| Platform key | Target |
| --- | --- |
| `windows-x86_64` | Windows x64 |
| `windows-aarch64` | Windows ARM64 |
| `darwin-aarch64` | macOS Apple Silicon |
| `darwin-x86_64` | macOS Intel |
| `linux-x86_64` | Linux x64 |
| `linux-aarch64` | Linux ARM64 |

## Bridge plugin — install names vs release assets

| OS | Installed in Daz plugins folder | GitHub release asset (arch-specific) |
| --- | --- | --- |
| Windows x64 | `DazPilotBridge.dll` | `DazPilotBridge-x64.dll` |
| Windows ARM64 | `DazPilotBridge.dll` | `DazPilotBridge-arm64.dll` |
| macOS x64 | `libDazPilotBridge.dylib` | `libDazPilotBridge-x64.dylib` |
| macOS ARM64 | `libDazPilotBridge.dylib` | `libDazPilotBridge-arm64.dylib` |
| Linux x64 | `libDazPilotBridge.so` | `libDazPilotBridge-x64.so` |
| Linux ARM64 | `libDazPilotBridge.so` | `libDazPilotBridge-arm64.so` |

The app tries the arch-specific asset first, then falls back to the generic installed filename.

### Daz Studio plugin paths (auto-detected)

| OS | Default plugins directory |
| --- | --- |
| Windows | `C:\Program Files\DAZ 3D\DAZStudio4\plugins` (also `(x86)` path) |
| macOS | `~/Library/Application Support/DAZ 3D/Studio4/plugins` |
| Linux | `~/.local/share/DAZ 3D/Studio4/plugins` |

## Bridge plugin — build status

| Platform | Source | CI build | Notes |
| --- | --- | --- | --- |
| Windows x64 | `plugins/daz3d-bridge/` | Yes | Primary; 191 commands |
| Windows ARM64 | Same | Yes (plugin matrix) | Requires ARM64 Daz SDK libs when available |
| macOS | Same | Manual / future CI | Needs macOS Daz SDK (`mac64` libs) |
| Linux | Same | Manual | Wine strategy in ROADMAP |

## Version alignment

| File | Field |
| --- | --- |
| `package.json` | `version` |
| `src-tauri/tauri.conf.json` | `version` |
| `src-tauri/Cargo.toml` | `version` |
| Git tag | `v*` (app), `plugin-v*` (bridge-only) |

## Release tags

| Tag pattern | Workflow | Output |
| --- | --- | --- |
| `v0.x.y` | `app-release.yml` | Full app for all 6 targets + `latest.json` |
| `plugin-v0.x.y` | `plugin-release.yml` | Bridge binaries per Windows arch |

## Schema parity

- Manifest: `plugins/daz3d-bridge/bridge_commands.manifest`
- Regenerate: `scripts/sync-bridge-manifest.ps1`
- Test: `cargo test bridge_schema_parity_with_cpp_manifest`
