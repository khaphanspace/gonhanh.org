# Types Telex/VNI keystrokes into another app at a fixed speed, to test GoNhanh where a human
# can't type fast enough (terminals, Electron apps...).
#
# Usage:
#   .\platforms\windows\tools\typing-test.ps1 -Target warp
#   .\platforms\windows\tools\typing-test.ps1 -Target warp -DelayMs 5 -Repeat 3
#   .\platforms\windows\tools\typing-test.ps1 -Keys "vieetj nam " -Target notepad
#
# GoNhanh ignores keys injected by other software, so this script restarts it with
# GONHANH_TEST_INPUT=1 (accept injected keys) and restarts it normally when done.

param(
    # Raw keystrokes to send (Telex by default)
    [string]$Keys = "Tieengs Vieetj cos daaus raats ddepj, gox nhanh khoong bij maats chuwx. ",
    # What the app should show afterwards (printed for comparison)
    [string]$Expect = "Tiếng Việt có dấu rất đẹp, gõ nhanh không bị mất chữ. ",
    # Process name of the app to type into (activated automatically); empty = the focused window
    [string]$Target = "",
    # Pause between keystrokes; 10 ms ≈ 1000 chars/min, far above human speed
    [int]$DelayMs = 10,
    # How long a key stays down
    [int]$HoldMs = 2,
    [int]$Repeat = 1,
    [int]$CountdownSec = 3,
    # Leave GoNhanh running in test mode afterwards
    [switch]$KeepTestMode,
    # Use the GoNhanh that is already running (must have been started with GONHANH_TEST_INPUT=1)
    [switch]$NoRestart
)

$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$exe = @("$root\publish\GoNhanh.exe", "$root\build\Release\gonhanh.exe") |
    Where-Object { Test-Path $_ } | Select-Object -First 1

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class TypingTest {
    [StructLayout(LayoutKind.Sequential)] struct KEYBDINPUT { public ushort wVk, wScan; public uint dwFlags, time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Sequential)] struct MOUSEINPUT { public int dx, dy; public uint mouseData, dwFlags, time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Explicit)] struct UNION { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
    [StructLayout(LayoutKind.Sequential)] struct INPUT { public uint type; public UNION u; }
    [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint n, INPUT[] inputs, int size);
    [DllImport("user32.dll")] static extern short VkKeyScanW(char ch);
    [DllImport("user32.dll")] static extern uint MapVirtualKeyW(uint code, uint type);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hwnd, int cmd);

    static void Key(ushort vk, bool up) {
        var i = new INPUT[1];
        i[0].type = 1;
        i[0].u.ki.wVk = vk;
        i[0].u.ki.wScan = (ushort)MapVirtualKeyW(vk, 0);
        i[0].u.ki.dwFlags = up ? 2u : 0u;
        SendInput(1, i, Marshal.SizeOf(typeof(INPUT)));
    }

    // Returns false if a character has no key on the current layout
    public static bool Type(string text, int delayMs, int holdMs) {
        foreach (char c in text) {
            short scan = VkKeyScanW(c);
            if (scan == -1) return false;
            ushort vk = (ushort)(scan & 0xFF);
            bool shift = (scan & 0x100) != 0;
            if (shift) Key(0x10, false);
            Key(vk, false);
            if (holdMs > 0) System.Threading.Thread.Sleep(holdMs);
            Key(vk, true);
            if (shift) Key(0x10, true);
            if (delayMs > 0) System.Threading.Thread.Sleep(delayMs);
        }
        return true;
    }
}
"@

function Start-GoNhanh([bool]$testMode) {
    Get-Process gonhanh -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Milliseconds 300
    if ($testMode) { $env:GONHANH_TEST_INPUT = "1" } else { Remove-Item Env:GONHANH_TEST_INPUT -ErrorAction SilentlyContinue }
    Start-Process -FilePath $exe
    Remove-Item Env:GONHANH_TEST_INPUT -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 800
}

if (-not $NoRestart) {
    if (-not $exe) { throw "GoNhanh.exe not found. Build it first: .\scripts\build-windows.ps1" }
    Write-Host "Restarting GoNhanh in test mode ($exe)"
    Start-GoNhanh $true
}

if ($Target) {
    $proc = Get-Process $Target -ErrorAction SilentlyContinue |
        Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } | Select-Object -First 1
    if (-not $proc) { throw "No window found for process '$Target'" }
    Write-Host "Typing into $($proc.ProcessName) (pid $($proc.Id)) in $CountdownSec s - don't touch the keyboard"
    Start-Sleep -Seconds $CountdownSec
    [TypingTest]::ShowWindow($proc.MainWindowHandle, 9) | Out-Null
    [TypingTest]::SetForegroundWindow($proc.MainWindowHandle) | Out-Null
    Start-Sleep -Milliseconds 400
} else {
    Write-Host "Click into the target app now - typing starts in $CountdownSec s"
    Start-Sleep -Seconds $CountdownSec
}

$sw = [Diagnostics.Stopwatch]::StartNew()
for ($i = 0; $i -lt $Repeat; $i++) {
    if (-not [TypingTest]::Type($Keys, $DelayMs, $HoldMs)) { Write-Warning "Some characters can't be typed on this keyboard layout" }
}
$sw.Stop()

$total = $Keys.Length * $Repeat
$cpm = [int]($total / $sw.Elapsed.TotalMinutes)
Write-Host ""
Write-Host "Sent $total keystrokes in $([int]$sw.Elapsed.TotalMilliseconds) ms (~$cpm keys/min)"
Write-Host "Expected ($Repeat x):"
Write-Host "  $Expect"

if (-not $NoRestart -and -not $KeepTestMode) {
    Start-Sleep -Seconds 1  # let paced output for slow apps finish
    Start-GoNhanh $false
    Write-Host "GoNhanh restarted in normal mode"
}
