# Build Gõ Nhanh Rust core for Windows
# Requires: Rust with x86_64-pc-windows-msvc and aarch64-pc-windows-msvc targets
#
# Run: pwsh scripts/build-windows.ps1

$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

Write-Host "Building gonhanh_core.dll for Windows..." -ForegroundColor Cyan

# Build for x64
Write-Host "Building x64..." -ForegroundColor Yellow
cargo build --release --target x86_64-pc-windows-msvc
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$x64Src = "target/x86_64-pc-windows-msvc/release/gonhanh_core.dll"
$x64Dst = "platforms/windows/libs/x64/gonhanh_core.dll"
if (Test-Path $x64Src) {
    Copy-Item $x64Src $x64Dst -Force
    Write-Host "Copied to $x64Dst" -ForegroundColor Green
} else {
    Write-Host "ERROR: $x64Src not found" -ForegroundColor Red
    exit 1
}

# Build for ARM64
Write-Host "Building arm64..." -ForegroundColor Yellow
cargo build --release --target aarch64-pc-windows-msvc
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$arm64Src = "target/aarch64-pc-windows-msvc/release/gonhanh_core.dll"
$arm64Dst = "platforms/windows/libs/arm64/gonhanh_core.dll"
if (Test-Path $arm64Src) {
    Copy-Item $arm64Src $arm64Dst -Force
    Write-Host "Copied to $arm64Dst" -ForegroundColor Green
} else {
    Write-Host "ERROR: $arm64Src not found" -ForegroundColor Red
    exit 1
}

Write-Host "Build complete!" -ForegroundColor Green
