# DazPilot bridge acceptance — automated (mock) + manual checklist
# Run from repo root: .\scripts\acceptance-bridge.ps1

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

Write-Host "=== DazPilot Bridge Acceptance ===" -ForegroundColor Cyan

Write-Host "`n[1/3] Workspace checks (npm + rust)..." -ForegroundColor Yellow
npm run check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n[2/3] Automated bridge tests (DAZPILOT_DEV_MOCK_BRIDGE=1)..." -ForegroundColor Yellow
$env:DAZPILOT_DEV_MOCK_BRIDGE = "1"
Push-Location src-tauri
cargo test acceptance_ -- --nocapture
$testExit = $LASTEXITCODE
Pop-Location
$env:DAZPILOT_DEV_MOCK_BRIDGE = $null
if ($testExit -ne 0) { exit $testExit }

Write-Host "`n[3/3] Manual live Daz Studio checklist (requires local install):" -ForegroundColor Yellow
@(
  "  - Build plugin: npm run plugin:rebuild",
  "  - Install DLL via Settings or: npm run tauri dev -> install plugin",
  "  - Start Daz Studio; confirm bridge listens on 127.0.0.1:8765",
  "  - In DazPilot: connect bridge; run chat: 'list nodes' / 'get scene info'",
  "  - Load a .duf asset from indexed library via chat",
  "  - Apply a pose preset; capture viewport; verify Scene panel sync",
  "  - Import a model; export a scene and verify plugin export plus fallback output"
) | ForEach-Object { Write-Host $_ }

Write-Host "`n=== Automated acceptance passed ===" -ForegroundColor Green
