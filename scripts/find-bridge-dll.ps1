# Resolves the built DazPilotBridge.dll (MSVC multi-config uses dist/Release/).
param(
    [string]$BridgeRoot = "plugins/daz3d-bridge"
)

$ErrorActionPreference = "Stop"

$candidates = @(
    (Join-Path $BridgeRoot "dist/DazPilotBridge.dll"),
    (Join-Path $BridgeRoot "dist/Release/DazPilotBridge.dll"),
    (Join-Path $BridgeRoot "dist/Debug/DazPilotBridge.dll")
)

foreach ($path in $candidates) {
    if (Test-Path -LiteralPath $path) {
        return (Resolve-Path -LiteralPath $path).Path
    }
}

$found = Get-ChildItem -Path $BridgeRoot -Recurse -Filter "DazPilotBridge.dll" -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -notmatch "DazPilotBridgeTests" } |
    Select-Object -First 1

if ($found) {
    return $found.FullName
}

throw "DazPilotBridge.dll not found under $BridgeRoot"
