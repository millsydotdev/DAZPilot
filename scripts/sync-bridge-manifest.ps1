# Regenerate bridge_commands.manifest from C++ dispatch table.
# Run from repo root: powershell -ExecutionPolicy Bypass -File scripts/sync-bridge-manifest.ps1

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$cpp = Join-Path $root "plugins\daz3d-bridge\DazPilotBridgePlugin.cpp"
$out = Join-Path $root "plugins\daz3d-bridge\bridge_commands.manifest"

if (-not (Test-Path $cpp)) {
    throw "Bridge source not found: $cpp"
}

$commands = Select-String -Path $cpp -Pattern 'if \(command == "([^"]+)"\)' |
    ForEach-Object { $_.Matches.Groups[1].Value } |
    Sort-Object -Unique

$header = @(
    "# Auto-generated from DazPilotBridgePlugin.cpp",
    "# Run: scripts/sync-bridge-manifest.ps1",
    ""
)

($header + $commands) | Out-File -FilePath $out -Encoding ascii
Write-Host "Wrote $($commands.Count) commands to $out"
