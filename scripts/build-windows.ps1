# GoNhanh Windows Build Script (PowerShell)
# Usage: .\scripts\build-windows.ps1 [-Clean] [-Debug]

param(
    [switch]$Clean,
    [switch]$Debug,
    [switch]$Package,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

if ($Help) {
    Write-Host "Usage: build-windows.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Clean    Remove existing build artifacts before building"
    Write-Host "  -Debug    Build with debug console enabled"
    Write-Host "  -Package  Create ZIP package for distribution"
    Write-Host "  -Help     Show this help message"
    exit 0
}

# Get project root - when run via Make, use current directory
$ProjectRoot = (Get-Location).Path
if ((Split-Path -Leaf $ProjectRoot) -eq "scripts") {
    $ProjectRoot = Split-Path -Parent $ProjectRoot
}
# Validate project root has core/ folder
if (-not (Test-Path "$ProjectRoot\core")) {
    Write-Host "Error: Cannot find core/ folder in $ProjectRoot"
    Write-Host "Please run from project root: .\scripts\build-windows.ps1"
    exit 1
}

$WindowsDir = "$ProjectRoot\platforms\windows"
$BuildDir = "$WindowsDir\build"

# Clean build artifacts
if ($Clean) {
    Write-Host "Cleaning build artifacts..."

    # Kill running GoNhanh processes
    $proc = Get-Process -Name "gonhanh" -ErrorAction SilentlyContinue
    if ($proc) {
        Write-Host "  Stopping gonhanh.exe..."
        Stop-Process -Name "gonhanh" -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 1
    }

    Remove-Item -Path $BuildDir -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -Path "$WindowsDir\publish" -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "  Done"
    Write-Host ""
}

# Get version from git tag
try {
    $GIT_TAG = git describe --tags --abbrev=0 2>$null
    if (-not $GIT_TAG) { $GIT_TAG = "v0.0.0" }
} catch {
    $GIT_TAG = "v0.0.0"
}
$VERSION = $GIT_TAG -replace "^v", ""

Write-Host "Building GoNhanh for Windows"
Write-Host "Version: $VERSION"
Write-Host ""

# Check for CMake
$cmakePath = Get-Command cmake -ErrorAction SilentlyContinue
if (-not $cmakePath) {
    Write-Host "Error: CMake not found"
    Write-Host "Install from: https://cmake.org/download/"
    exit 1
}

# Use local drive for build to avoid network drive issues
$IsNetworkDrive = $ProjectRoot -match "^\\\\|^C:\\Mac\\Home"
if ($IsNetworkDrive) {
    $LocalBuildDir = "$env:LOCALAPPDATA\gonhanh-build\windows"
    Write-Host "Using local build dir: $LocalBuildDir"
    Write-Host ""
} else {
    $LocalBuildDir = $BuildDir
}

# Configure CMake
Write-Host "[1/2] Configuring CMake..."
if (-not (Test-Path $LocalBuildDir)) {
    New-Item -ItemType Directory -Path $LocalBuildDir -Force | Out-Null
}

$cmakeArgs = @(
    "-S", $WindowsDir,
    "-B", $LocalBuildDir,
    "-G", "Visual Studio 17 2022",
    "-A", "x64"
)

if ($Debug) {
    $cmakeArgs += "-DENABLE_DEBUG_CONSOLE=ON"
}

cmake @cmakeArgs
if ($LASTEXITCODE -ne 0) { throw "CMake configure failed" }

# Build
Write-Host "[2/2] Building..."
$buildType = if ($Debug) { "Debug" } else { "Release" }
cmake --build $LocalBuildDir --config $buildType
if ($LASTEXITCODE -ne 0) { throw "CMake build failed" }

# Copy to publish folder
$PublishDir = "$WindowsDir\publish"
if (-not (Test-Path $PublishDir)) {
    New-Item -ItemType Directory -Path $PublishDir -Force | Out-Null
}

$ExeSource = "$LocalBuildDir\$buildType\gonhanh.exe"
if (Test-Path $ExeSource) {
    Copy-Item $ExeSource "$PublishDir\GoNhanh.exe" -Force
}

Write-Host ""
Write-Host "Build complete!"
Write-Host "Output: platforms/windows/publish/GoNhanh.exe"

# Create ZIP package only if -Package specified
if ($Package) {
    Write-Host ""
    Write-Host "Creating package..."
    $ZipName = "GoNhanh-$VERSION-win-x64.zip"
    $ZipPath = "$WindowsDir\$ZipName"
    Remove-Item $ZipPath -Force -ErrorAction SilentlyContinue

    if (Test-Path "$PublishDir\GoNhanh.exe") {
        Compress-Archive -Path "$PublishDir\GoNhanh.exe" -DestinationPath $ZipPath -Force
        Write-Host "Package: platforms/windows/$ZipName"
    }
}
