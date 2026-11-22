# GoNhanh Windows Development Setup Check
# Run: powershell -ExecutionPolicy Bypass -File scripts/setup/windows.ps1
#
# The Windows app is one native exe: C++ Win32 UI + Rust core (static lib), built with
# CMake + MSVC. This script checks the toolchain and prints how to install what's missing.

Write-Host "GoNhanh Windows Setup" -ForegroundColor Cyan
Write-Host "=====================" -ForegroundColor Cyan
$missing = 0

# Rust (MSVC toolchain)
Write-Host "`nRust..." -ForegroundColor Yellow
if (Get-Command rustc -ErrorAction SilentlyContinue) {
    Write-Host "  $(rustc --version)" -ForegroundColor Green
} else {
    Write-Host "  Not found. Install: winget install Rustlang.Rustup" -ForegroundColor Red
    $missing++
}

# CMake
Write-Host "`nCMake (3.22+)..." -ForegroundColor Yellow
if (Get-Command cmake -ErrorAction SilentlyContinue) {
    Write-Host "  $((cmake --version | Select-Object -First 1))" -ForegroundColor Green
} else {
    Write-Host "  Not found. Install: winget install Kitware.CMake" -ForegroundColor Red
    $missing++
}

# Visual Studio 2026 or 2022 with the C++ workload
Write-Host "`nVisual Studio C++ build tools (2026 or 2022)..." -ForegroundColor Yellow
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$vsVersion = $null
if (Test-Path $vsWhere) {
    $vsVersion = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationVersion
}
if ($vsVersion) {
    Write-Host "  Found version $vsVersion" -ForegroundColor Green
} else {
    Write-Host "  Not found. Install:" -ForegroundColor Red
    Write-Host '  winget install Microsoft.VisualStudio.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"' -ForegroundColor Red
    $missing++
}

if ($missing -gt 0) {
    Write-Host "`n$missing tool(s) missing. Open a new terminal after installing so PATH is updated." -ForegroundColor Red
    exit 1
}

Write-Host "`nAll set. Build with:" -ForegroundColor Green
Write-Host "  powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1 [-Run] [-Debug] [-Package]" -ForegroundColor Cyan
