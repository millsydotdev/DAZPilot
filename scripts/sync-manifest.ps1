$manifestFile = "mcp_commands.manifest"
$outputFile = "manifest\commands.json"
$sourceFile = "DAZStudioMCPPlugin.cpp"

if (-not (Test-Path $manifestFile)) {
    Write-Error "mcp_commands.manifest not found"
    exit 1
}

$commands = Get-Content $manifestFile | Where-Object { $_ -and $_[0] -ne '#' } | ForEach-Object { $_.Trim() }

$json = @{
    plugin = "DAZStudioMCP"
    version = "1.0.0"
    protocol = 1
    commands = $commands | ForEach-Object {
        @{
            name = $_
            description = ""
            category = "General"
            parameters = @()
        }
    }
}

$json | ConvertTo-Json -Depth 3 | Set-Content $outputFile
Write-Output "Generated $outputFile with $($commands.Count) commands"
