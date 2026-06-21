param(
    [string]$Config = "Release"
)

$paths = @(
    "dist\DAZStudioMCP.dll",
    "build\dist\DAZStudioMCP.dll",
    "build\$Config\DAZStudioMCP.dll",
    "build\bin\$Config\DAZStudioMCP.dll"
)

foreach ($p in $paths) {
    if (Test-Path $p) {
        Write-Output (Resolve-Path $p).Path
        exit 0
    }
}

Write-Error "DAZStudioMCP.dll not found. Build first with: scripts\build.ps1"
exit 1
