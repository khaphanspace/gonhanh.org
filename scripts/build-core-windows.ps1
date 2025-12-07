# GoNhanh Windows Core Build Script
# Run: powershell -ExecutionPolicy Bypass -File scripts/build-core-windows.ps1

param(
    [switch]$Release = $true,
    [switch]$x86 = $false,
    [switch]$ARM64 = $false
)

$ErrorActionPreference = "Stop"

Write-Host "Building GoNhanh Core for Windows" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan

$projectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$coreDir = Join-Path $projectRoot "core"
$outputDir = Join-Path $projectRoot "platforms/windows/GoNhanh/Native"

# Ensure output directory exists
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
    Write-Host "Created output directory: $outputDir" -ForegroundColor Yellow
}

# Build configuration
$buildType = if ($Release) { "release" } else { "debug" }
$buildFlag = if ($Release) { "--release" } else { "" }

# Primary target: x64
Write-Host "`nBuilding for x86_64-pc-windows-msvc..." -ForegroundColor Yellow
Push-Location $coreDir
try {
    cargo build $buildFlag --target x86_64-pc-windows-msvc

    $srcDll = Join-Path $coreDir "target/x86_64-pc-windows-msvc/$buildType/gonhanh_core.dll"
    $dstDll = Join-Path $outputDir "gonhanh_core.dll"

    if (Test-Path $srcDll) {
        Copy-Item $srcDll $dstDll -Force
        Write-Host "Copied: $dstDll" -ForegroundColor Green

        $fileInfo = Get-Item $dstDll
        Write-Host "Size: $([math]::Round($fileInfo.Length / 1KB, 2)) KB" -ForegroundColor Gray
    } else {
        Write-Host "Build failed: DLL not found at $srcDll" -ForegroundColor Red
        exit 1
    }
} finally {
    Pop-Location
}

# Optional: 32-bit build
if ($x86) {
    Write-Host "`nBuilding for i686-pc-windows-msvc..." -ForegroundColor Yellow
    Push-Location $coreDir
    try {
        cargo build $buildFlag --target i686-pc-windows-msvc

        $srcDll = Join-Path $coreDir "target/i686-pc-windows-msvc/$buildType/gonhanh_core.dll"
        $dstDll = Join-Path $outputDir "gonhanh_core_x86.dll"

        if (Test-Path $srcDll) {
            Copy-Item $srcDll $dstDll -Force
            Write-Host "Copied: $dstDll" -ForegroundColor Green
        }
    } finally {
        Pop-Location
    }
}

# Optional: ARM64 build
if ($ARM64) {
    Write-Host "`nBuilding for aarch64-pc-windows-msvc..." -ForegroundColor Yellow
    Push-Location $coreDir
    try {
        cargo build $buildFlag --target aarch64-pc-windows-msvc

        $srcDll = Join-Path $coreDir "target/aarch64-pc-windows-msvc/$buildType/gonhanh_core.dll"
        $dstDll = Join-Path $outputDir "gonhanh_core_arm64.dll"

        if (Test-Path $srcDll) {
            Copy-Item $srcDll $dstDll -Force
            Write-Host "Copied: $dstDll" -ForegroundColor Green
        }
    } finally {
        Pop-Location
    }
}

Write-Host "`nBuild complete!" -ForegroundColor Green
