# Install Gõ Nhanh locally from build output
# Requires: Run build-windows.ps1 -NativeAOT first
#
# Run: pwsh scripts/install-windows.ps1
#      pwsh scripts/install-windows.ps1 -Platform arm64

param(
    [ValidateSet('x64', 'arm64')]
    [string]$Platform = 'x64'
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path $PSScriptRoot -Parent
$AppDir = Join-Path $ProjectRoot 'platforms/windows/GoNhanh.Windows'

# Find MSIX package
$publishPath = Join-Path $AppDir "bin/Release/net9.0-windows10.0.26100.0/win-$Platform/publish"
$msixPath = Get-ChildItem "$publishPath/*.msix" -ErrorAction SilentlyContinue | Select-Object -First 1

if (-not $msixPath) {
    # Try to find AppX package
    $msixPath = Get-ChildItem "$publishPath/*.appx" -ErrorAction SilentlyContinue | Select-Object -First 1
}

if (-not $msixPath) {
    Write-Host "MSIX/APPX package not found at: $publishPath" -ForegroundColor Red
    Write-Host "Please run: .\scripts\build-windows.ps1 -NativeAOT -Platform $Platform" -ForegroundColor Yellow
    exit 1
}

Write-Host "Installing $($msixPath.Name)..." -ForegroundColor Cyan

try {
    # Remove existing installation if present
    $existingPackage = Get-AppxPackage -Name "GoNhanh.Windows" -ErrorAction SilentlyContinue
    if ($existingPackage) {
        Write-Host "Removing existing installation..." -ForegroundColor Yellow
        Remove-AppxPackage -Package $existingPackage.PackageFullName
    }

    # Install new package
    Add-AppxPackage -Path $msixPath.FullName

    Write-Host "`nInstallation complete!" -ForegroundColor Green
    Write-Host "You can find Go Nhanh in the Start Menu." -ForegroundColor Cyan
}
catch {
    Write-Host "Installation failed: $_" -ForegroundColor Red
    Write-Host "`nTip: For unsigned packages, enable Developer Mode in Windows Settings:" -ForegroundColor Yellow
    Write-Host "Settings > Update & Security > For developers > Developer Mode" -ForegroundColor Yellow
    exit 1
}
