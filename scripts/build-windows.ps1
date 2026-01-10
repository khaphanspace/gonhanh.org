# Build Gõ Nhanh for Windows (Rust core + .NET app)
# Requires: Rust toolchain, .NET 9 SDK
#
# Run: pwsh scripts/build-windows.ps1
#      pwsh scripts/build-windows.ps1 -Platform x64
#      pwsh scripts/build-windows.ps1 -NativeAOT
#      pwsh scripts/build-windows.ps1 -SkipRust

param(
    [ValidateSet('x64', 'arm64', 'all')]
    [string]$Platform = 'all',

    [ValidateSet('Debug', 'Release')]
    [string]$Configuration = 'Release',

    [switch]$NativeAOT,
    [switch]$SkipRust
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path $PSScriptRoot -Parent
Set-Location $ProjectRoot

$CoreDir = Join-Path $ProjectRoot 'core'
$WindowsDir = Join-Path $ProjectRoot 'platforms/windows'
$AppDir = Join-Path $WindowsDir 'GoNhanh.Windows'

function Build-RustCore {
    param([string]$Target, [string]$Platform)

    Write-Host "Building Rust core for $Target..." -ForegroundColor Cyan

    Push-Location $CoreDir
    try {
        cargo build --release --target $Target
        if ($LASTEXITCODE -ne 0) { throw "Rust build failed" }

        $libDir = Join-Path $WindowsDir "libs/$Platform"
        New-Item -ItemType Directory -Force -Path $libDir | Out-Null
        Copy-Item "target/$Target/release/gonhanh_core.dll" $libDir -Force
        Write-Host "Copied DLL to $libDir" -ForegroundColor Green
    }
    finally {
        Pop-Location
    }
}

function Build-App {
    param([string]$Platform)

    Write-Host "Building .NET app for $Platform..." -ForegroundColor Cyan

    Push-Location $AppDir
    try {
        if ($NativeAOT) {
            Write-Host "Publishing with NativeAOT..." -ForegroundColor Yellow
            dotnet publish -c $Configuration -r "win-$Platform"
        } else {
            dotnet build -c $Configuration -p:Platform=$Platform
        }
        if ($LASTEXITCODE -ne 0) { throw "App build failed" }
        Write-Host "App build complete for $Platform" -ForegroundColor Green
    }
    finally {
        Pop-Location
    }
}

# Main
$platforms = if ($Platform -eq 'all') { @('x64', 'arm64') } else { @($Platform) }

foreach ($p in $platforms) {
    $rustTarget = if ($p -eq 'x64') { 'x86_64-pc-windows-msvc' } else { 'aarch64-pc-windows-msvc' }

    if (-not $SkipRust) {
        Build-RustCore -Target $rustTarget -Platform $p
    }
    Build-App -Platform $p
}

Write-Host "`nBuild complete!" -ForegroundColor Green
Write-Host "Output: $AppDir/bin/$Configuration/" -ForegroundColor Cyan
