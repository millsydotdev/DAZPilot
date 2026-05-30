# Copies the built bridge DLL into a destination (plugin release dir or Tauri resources).
param(
    [Parameter(Mandatory = $true)]
    [string]$Destination,
    [string]$BridgeRoot = "plugins/daz3d-bridge"
)

$ErrorActionPreference = "Stop"

$dll = & "$PSScriptRoot/find-bridge-dll.ps1" -BridgeRoot $BridgeRoot
$destParent = Split-Path -Parent $Destination

if ($destParent -and -not (Test-Path -LiteralPath $destParent)) {
    New-Item -ItemType Directory -Force -Path $destParent | Out-Null
}

Copy-Item -LiteralPath $dll -Destination $Destination -Force
Write-Host "Staged bridge DLL: $dll -> $Destination"
