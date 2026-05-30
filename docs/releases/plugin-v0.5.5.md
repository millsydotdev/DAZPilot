Pre-built **Windows x64** Daz Studio bridge plugin for DazPilot **v0.5.5**.

## Files

| File | Use |
| --- | --- |
| `DazPilotBridge-x64.dll` | Recommended for Windows x64 / Daz Studio |
| `DazPilotBridge.dll` | Same build, generic filename |
| `bridge_commands.manifest` | Command list (reference / schema parity with the desktop app) |

## Install

1. Download **`DazPilotBridge-x64.dll`** (or `DazPilotBridge.dll`).
2. Copy into your Daz Studio **plugins** folder (same location as other `.dll` plugins).
3. Restart Daz Studio.
4. Open **DazPilot Bridge** from the DazPilot menu.

The bridge listens on **`127.0.0.1:8765`**. Start the DazPilot desktop app to connect.

## Pane features

- Bridge listen status (host/port)
- **Send viewport to DazPilot** — pushes scene/viewport context to the app inbox on `127.0.0.1:8766`
- **Copy diagnostics** for support troubleshooting

## Note on ARM64 Windows

The DAZ SDK only provides **x64** Windows libraries. Daz Studio on ARM64 Windows uses this same x64 plugin.

## Desktop app

Installers and updates: **[DazPilot v0.5.5](https://github.com/millsydotdev/DAZPilot/releases/tag/v0.5.5)**
