# GoNhanh Windows Development Setup Script
# Run: powershell -ExecutionPolicy Bypass -File scripts/setup/windows.ps1

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

# Check Visual Studio C++ tools (the Rust MSVC target needs the MSVC linker and rc.exe)
Write-Host "`nChecking Visual Studio C++ tools..." -ForegroundColor Yellow
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$vsPath = if (Test-Path $vsWhere) { & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath }
if ($vsPath) {
    Write-Host "Visual Studio C++ tools found: $vsPath" -ForegroundColor Green
} else {
    Write-Host "Visual Studio C++ tools not found. Install Build Tools with 'Desktop development with C++':" -ForegroundColor Red
    Write-Host "https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022" -ForegroundColor Red
    exit 1
}

Write-Host "`nSetup complete!" -ForegroundColor Green
Write-Host "Build with (Git Bash): ./scripts/build/windows.sh" -ForegroundColor Cyan
