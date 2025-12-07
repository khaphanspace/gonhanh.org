# GoNhanh Windows Development Setup Script
# Run: powershell -ExecutionPolicy Bypass -File scripts/setup-windows.ps1

Write-Host "GoNhanh Windows Setup" -ForegroundColor Cyan
Write-Host "=====================" -ForegroundColor Cyan

# Check Rust installation
Write-Host "`nChecking Rust installation..." -ForegroundColor Yellow
if (Get-Command rustc -ErrorAction SilentlyContinue) {
    $rustVersion = rustc --version
    Write-Host "Rust installed: $rustVersion" -ForegroundColor Green
} else {
    Write-Host "Rust not found. Please install from https://rustup.rs" -ForegroundColor Red
    exit 1
}

# Add Windows targets
Write-Host "`nAdding Windows targets..." -ForegroundColor Yellow

$targets = @(
    "x86_64-pc-windows-msvc",    # 64-bit Windows (primary)
    "i686-pc-windows-msvc",       # 32-bit Windows (optional)
    "aarch64-pc-windows-msvc"     # ARM64 Windows (optional)
)

foreach ($target in $targets) {
    Write-Host "Adding target: $target"
    rustup target add $target
}

# Check .NET SDK
Write-Host "`nChecking .NET SDK..." -ForegroundColor Yellow
if (Get-Command dotnet -ErrorAction SilentlyContinue) {
    $dotnetVersion = dotnet --version
    Write-Host ".NET SDK installed: $dotnetVersion" -ForegroundColor Green
} else {
    Write-Host ".NET SDK not found. Please install .NET 8.0 SDK from https://dotnet.microsoft.com" -ForegroundColor Red
    exit 1
}

# Check Visual Studio Build Tools
Write-Host "`nChecking Visual Studio Build Tools..." -ForegroundColor Yellow
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsPath = & $vsWhere -latest -property installationPath
    Write-Host "Visual Studio found: $vsPath" -ForegroundColor Green
} else {
    Write-Host "Visual Studio not found. Rust MSVC target requires Visual Studio Build Tools." -ForegroundColor Yellow
    Write-Host "Download from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022" -ForegroundColor Yellow
}

Write-Host "`nSetup complete!" -ForegroundColor Green
Write-Host "Run 'scripts/build-core-windows.ps1' to build the Rust core library." -ForegroundColor Cyan
