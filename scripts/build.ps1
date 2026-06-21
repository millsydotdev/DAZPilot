param(
    [string]$Config = "Release",
    [switch]$Clean
)

if ($Clean -and (Test-Path "build")) {
    Remove-Item -Recurse -Force "build"
}

$SDKRoot = $env:DAZ_SDK_ROOT
if (-not $SDKRoot) {
    $SDKRoot = "E:\DAZ_Studio_SDK"  # fallback default
}

cmake -B build -DCMAKE_BUILD_TYPE=$Config -DDAZ_SDK_ROOT="$SDKRoot"
if ($?) {
    cmake --build build --config $Config
}
