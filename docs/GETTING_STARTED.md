# Getting Started

Updated: May 2026

This guide walks through setting up DazPilot from scratch: installing dependencies, building the bridge plugin, and running the desktop app.

---

## Prerequisites

| Tool | Version | Purpose |
| --- | --- | --- |
| **Node.js** | 20+ | Frontend build and dev server |
| **npm** | 10+ | Package manager (bundled with Node.js) |
| **Rust** | 1.70+ | Tauri backend compilation |
| **CMake** | 3.20+ | Bridge plugin build system |
| **Visual Studio Build Tools** | 2022 | C++ compiler for the bridge plugin (Windows) |
| **Daz Studio** | 4.5+ | Target application for the bridge plugin |
| **Daz Studio SDK** | 4.5+ | Proprietary SDK headers (installed via DIM) |

### Installing Node.js

Download from [nodejs.org](https://nodejs.org/). The LTS version (20+) is recommended.

```powershell
node --version    # Should print v20.x.x or higher
npm --version     # Should print 10.x.x or higher
```

### Installing Rust

Install via [rustup](https://rustup.rs/):

```powershell
winget install Rustlang.Rustup
# or
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify:

```powershell
rustc --version   # Should print 1.70.0 or higher
cargo --version
```

### Installing CMake

Download from [cmake.org](https://cmake.org/download/) or install via a package manager:

```powershell
winget install Kitware.CMake
# or
choco install cmake
```

Verify:

```powershell
cmake --version   # Should print 3.20 or higher
```

### Installing Visual Studio Build Tools

The C++ bridge plugin requires a C++ compiler. On Windows, install the Visual Studio 2022 Build Tools:

1. Download from [visualstudio.microsoft.com](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
2. In the installer, select **Desktop development with C++**
3. Complete the installation

### Installing the Daz Studio SDK

The Daz Studio C++ SDK is proprietary and not hosted on GitHub.

1. Open **Daz Install Manager** (DIM)
2. Search for `Daz Studio SDK` or `DAZStudio4.5+ SDK`
3. Install it to the default location
4. Note the install path — you will need it for the build

---

## Clone The Repository

```powershell
git clone https://github.com/millsydotdev/DazPilot.git
cd DazPilot
```

---

## Configure The SDK Path

DazPilot needs the Daz SDK headers to build the bridge plugin and index SDK classes.

**Option A:** Place the SDK folder in the thirdparty directory:

```text
DazPilot/
├── thirdparty/
│   └── DAZStudio4.5+ SDK/    ← SDK folder here
├── src/
├── src-tauri/
└── ...
```

**Option B:** Set an environment variable:

```powershell
$env:DAZ_SDK_PATH = "C:\Users\You\Documents\DAZ 3D\DAZStudio4.5+ SDK\include"
```

The default include path is `thirdparty/DAZStudio4.5+ SDK\include` relative to the repository root.

---

## Build The Bridge Plugin

The bridge plugin is a C++ Daz Studio plugin that runs a TCP server for communication.

```powershell
npm run plugin:rebuild
```

This runs CMake configure + build. The output DLL lands in:

```text
plugins\daz3d-bridge\dist\Release\DazPilotBridge.dll
```

### Install The Plugin

Copy the DLL into your Daz Studio plugins directory:

```powershell
copy plugins\daz3d-bridge\dist\Release\DazPilotBridge.dll "C:\Program Files\DAZ 3D\DAZStudio4\plugins\"
```

Or use the deploy shortcut (may require elevated shell):

```powershell
npm run plugin:rebuild:deploy
```

Restart Daz Studio after copying the DLL.

---

## Install Frontend Dependencies

```powershell
npm install
```

---

## Run The Dev Server

```powershell
npm run dev
```

This starts the Vite dev server for the React frontend. The Tauri backend is not running in this mode — use `npm run tauri dev` for the full desktop app.

### Development With Mocks

For development without Daz Studio running:

```powershell
$env:DAZPILOT_DEV_MOCK_BRIDGE = "1"
$env:DAZPILOT_DEV_MOCK_AI = "1"
npm run dev
```

---

## Build The Full Desktop App

```powershell
npm run check         # Rust clippy + typecheck + lint + format check + Rust fmt + test
npm run tauri build   # Production build
```

The installer output appears in:

```text
src-tauri/target/release/bundle/
```

---

## Run Tests

```powershell
npm test              # Frontend unit tests (Vitest)
cargo test            # Rust backend tests
npm run check         # Full pipeline: Rust clippy + typecheck + lint + format check + Rust fmt + test
npm run acceptance    # Bridge acceptance tests with mock bridge
```

---

## Environment Variables Reference

| Variable | Default | Effect |
| --- | --- | --- |
| `DAZPILOT_DEV_MOCK_BRIDGE` | unset | Set to `1` to enable the bridge mock |
| `DAZPILOT_DEV_MOCK_AI` | unset | Set to `1` to enable the AI mock |
| `DAZPILOT_AI_BACKEND` | local GGUF | Set to `ollama` to use Ollama instead |
| `DAZ_SDK_PATH` | `DAZStudio4.5+ SDK/include` | Override the SDK include path |

---

## Troubleshooting

### "Daz SDK not found" during plugin build

- Confirm the SDK is installed via DIM
- Check that `DAZStudio4.5+ SDK` exists in the repo root, or `DAZ_SDK_PATH` is set
- Verify the `include` subdirectory contains `.h` files

### CMake errors on Windows

- Ensure Visual Studio Build Tools are installed with the C++ workload
- Try running from a **Developer PowerShell for VS 2022**

### Bridge connection refused

- Confirm Daz Studio is running with the bridge plugin installed
- Check the Daz Studio log for plugin load errors
- Verify port `8765` is not blocked by a firewall
- Use `DAZPILOT_DEV_MOCK_BRIDGE=1` to test without Daz Studio

### `npm run check` fails with lint warnings

- Lint warnings for `no-explicit-any` are expected and do not block the build
- Lint **errors** need to be fixed before proceeding

### Frontend tests fail

- Run `npm install` to ensure all dependencies are current
- Check Node.js version is 20+

---

## Next Steps

- [Architecture](ARCHITECTURE.md) — understand the runtime layers
- [Agents](AGENTS.md) — learn how the AI system works
- [Current State](CURRENT_STATE.md) — see what is verified
