# End-to-end test of the `gonhanh` / `gn` CLI against a real GoNhanh.exe.
#
# Usage (after .\scripts\build-windows.ps1):
#   .\platforms\windows\tools\cli-test.ps1
#
# Stops a running GoNhanh, tests the commands that work without the app, starts the built
# GoNhanh.exe, tests every command, then puts things back: the Vietnamese/English state and
# input method are restored and a previously running GoNhanh is started again.
# Exit code 1 if any check fails (used by CI).

param(
    [string]$PublishDir = (Join-Path $PSScriptRoot "..\publish")
)

$ErrorActionPreference = "Stop"
$PublishDir = (Resolve-Path $PublishDir).Path
$App = Join-Path $PublishDir "GoNhanh.exe"
$Cli = Join-Path $PublishDir "bin\gonhanh.exe"
$Gn = Join-Path $PublishDir "bin\gn.exe"
foreach ($f in $App, $Cli, $Gn) { if (-not (Test-Path $f)) { throw "Not found: $f (build first)" } }

$script:failures = 0
$script:checks = 0

# Run the CLI, capturing stdout/stderr as UTF-8 regardless of the console code page
function Invoke-Cli([string]$exe, [string[]]$cliArgs) {
    $psi = New-Object Diagnostics.ProcessStartInfo
    $psi.FileName = $exe
    $psi.Arguments = ($cliArgs | ForEach-Object { if ($_ -match '\s') { "`"$_`"" } else { $_ } }) -join ' '
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.StandardOutputEncoding = [Text.Encoding]::UTF8
    $psi.StandardErrorEncoding = [Text.Encoding]::UTF8
    $p = [Diagnostics.Process]::Start($psi)
    $out = $p.StandardOutput.ReadToEnd()
    $err = $p.StandardError.ReadToEnd()
    $p.WaitForExit()
    [pscustomobject]@{ Code = $p.ExitCode; Out = $out.TrimEnd("`r", "`n"); Err = $err }
}

# Check exit code, exact stdout (or $null to skip) and optionally a stderr substring
function Check([string]$name, [string[]]$cliArgs, [int]$code, $out, [string]$errLike = $null, [string]$exe = $Cli) {
    $script:checks++
    $r = Invoke-Cli $exe $cliArgs
    $ok = $r.Code -eq $code -and ($null -eq $out -or $r.Out -ceq $out) -and
          (-not $errLike -or $r.Err -like "*$errLike*")
    if ($ok) {
        Write-Host "  ok    $name"
    } else {
        $script:failures++
        Write-Host "  FAIL  $name" -ForegroundColor Red
        Write-Host "        expected exit=$code out='$out' err~'$errLike'"
        Write-Host "        got      exit=$($r.Code) out='$($r.Out)' err='$($r.Err.Trim())'"
    }
}

function Stop-GoNhanh {
    Get-Process GoNhanh -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -and (Split-Path $_.Path -Leaf) -ieq "GoNhanh.exe" -and $_.Path -notlike "*\bin\*" } |
        Stop-Process -Force
    for ($i = 0; $i -lt 50 -and (Invoke-Cli $Cli @()).Err -notlike "*chưa chạy*"; $i++) { Start-Sleep -Milliseconds 100 }
}

# --- Remember what to put back -------------------------------------------------------------
$original = Get-Process GoNhanh -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -and $_.Path -notlike "*\bin\*" } | Select-Object -First 1
$originalPath = if ($original) { $original.Path } else { $null }
$before = Invoke-Cli $Cli @("status")
$originalEnabled = $null; $originalMethod = $null
if ($before.Code -eq 0) {
    $originalEnabled = $before.Out -like "*BẬT*"
    $originalMethod = if ($before.Out -like "*vni*") { "vni" } else { "telex" }
}

$testApp = $null
try {
    Stop-GoNhanh

    Write-Host "Without GoNhanh running:"
    Check "version"            @("version") 0 $null
    Check "--version"          @("--version") 0 $null
    Check "help"               @("help") 0 $null
    Check "get -> error"       @() 1 "" "chưa chạy"
    Check "vi -> error"        @("vi") 1 "" "chưa chạy"
    Check "en -> ok (already English)" @("en") 0 ""
    Check "off -> error"       @("off") 1 "" "chưa chạy"
    Check "status -> error"    @("status") 1 "" "chưa chạy"
    Check "quit -> ok (nothing to quit)" @("quit") 0 "Gõ Nhanh chưa chạy."
    Check "unknown command"    @("bogus") 1 "" "Lệnh không hợp lệ: bogus"
    Check "too many arguments" @("vi", "en") 1 "" "Lệnh không hợp lệ: vi en"
    $v = (Invoke-Cli $Cli @("version")).Out
    $script:checks++
    if ($v -match '^Gõ Nhanh v\d+\.\d+\.\d+$') { Write-Host "  ok    version format ($v)" }
    else { $script:failures++; Write-Host "  FAIL  version format: '$v'" -ForegroundColor Red }

    # Tab completion: load the script into a clean PowerShell and ask it to complete
    $script:checks++
    $completion = Invoke-Cli $Cli @("completion")
    $probe = Join-Path ([IO.Path]::GetTempPath()) "gonhanh-completion-test.ps1"
    @"
`$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.Encoding]::UTF8
& '$Cli' completion | Out-String | Invoke-Expression
`$r = TabExpansion2 -inputScript 'gn t' -cursorColumn 4
(`$r.CompletionMatches | ForEach-Object CompletionText) -join ','
`$r = TabExpansion2 -inputScript 'gonhanh.exe ' -cursorColumn 12
`$r.CompletionMatches.Count
(`$r.CompletionMatches | Where-Object CompletionText -eq 'vi').ToolTip
"@ | Out-File $probe -Encoding utf8
    $tab = Invoke-Cli "powershell.exe" @("-NoProfile", "-ExecutionPolicy", "Bypass", "-File", $probe)
    Remove-Item $probe -ErrorAction SilentlyContinue
    $tabLines = $tab.Out -split "`r?`n"
    $ascii = $completion.Out -notmatch '[^\x00-\x7F]'
    if ($completion.Code -eq 0 -and $ascii -and $tabLines[0] -eq "toggle,telex" -and [int]$tabLines[1] -ge 12 -and
        $tabLines[2] -eq "Bật gõ tiếng Việt (im lặng, dùng cho Vim/script)") {
        Write-Host "  ok    completion (gn t<Tab> -> $($tabLines[0]); $($tabLines[1]) commands; Vietnamese tooltips)"
    } else {
        $script:failures++
        Write-Host "  FAIL  completion: ascii=$ascii exit=$($completion.Code) out='$($tab.Out)' err='$($tab.Err.Trim())'" -ForegroundColor Red
    }

    Write-Host "Starting with gn start:"
    Check "start"           @("start") 0 "Đã mở Gõ Nhanh."
    $testApp = $true
    Check "start again"     @("start") 0 "Gõ Nhanh đang chạy."

    Write-Host "With GoNhanh running:"
    if ($null -eq $originalEnabled) {
        # GoNhanh wasn't running: remember the saved state it started with
        $s = (Invoke-Cli $Cli @("status")).Out
        $originalEnabled = $s -like "*BẬT*"
        $originalMethod = if ($s -like "*vni*") { "vni" } else { "telex" }
    }
    Check "en"              @("en") 0 ""
    Check "get -> en"       @() 0 "en"
    Check "en again (no-op)" @("en") 0 ""
    Check "vi"              @("vi") 0 ""
    Check "get -> vi"       @() 0 "vi"
    Check "toggle -> off"   @("toggle") 0 "○ TẮT │ telex"
    Check "toggle -> on"    @("toggle") 0 "● BẬT │ telex │ gonhanh"
    Check "vni"             @("vni") 0 "● BẬT │ vni │ gonhanh"
    Check "off keeps vni"   @("off") 0 "○ TẮT │ vni"
    Check "telex turns on"  @("telex") 0 "● BẬT │ telex │ gonhanh"
    Check "off"             @("off") 0 "○ TẮT │ telex"
    Check "on"              @("on") 0 "● BẬT │ telex │ gonhanh"
    Check "status"          @("status") 0 "● BẬT │ telex │ gonhanh"
    Check "gn alias"        @("status") 0 "● BẬT │ telex │ gonhanh" -exe $Gn
    Check "gn en"           @("en") 0 "" -exe $Gn
    Check "gn get"          @() 0 "en" -exe $Gn

    $sw = [Diagnostics.Stopwatch]::StartNew()
    for ($i = 0; $i -lt 20; $i++) { $null = Invoke-Cli $Gn @("en") }
    $sw.Stop()
    Write-Host ("  info  {0:N1} ms per call (process start included)" -f ($sw.Elapsed.TotalMilliseconds / 20))

    # Put the starting state back while the app still runs, then close it with gn quit
    if ($null -ne $originalEnabled) {
        $null = Invoke-Cli $Cli @($originalMethod)
        if (-not $originalEnabled) { $null = Invoke-Cli $Cli @("en") }
        $originalEnabled = $null
    }
    Check "quit"            @("quit") 0 "Đã thoát Gõ Nhanh."
    Check "get after quit"  @() 1 "" "chưa chạy"
    $testApp = $null
}
finally {
    # Put back the user's mode and method, then the GoNhanh they had running
    if ($null -ne $originalEnabled -and (Invoke-Cli $Cli @()).Code -eq 0) {
        $null = Invoke-Cli $Cli @($originalMethod)
        if (-not $originalEnabled) { $null = Invoke-Cli $Cli @("en") }
    }
    if ($testApp) { Stop-GoNhanh }
    if ($originalPath) { Start-Process -FilePath $originalPath | Out-Null }
}

Write-Host ""
if ($script:failures -gt 0) {
    Write-Host "$($script:failures) of $($script:checks) checks failed" -ForegroundColor Red
    exit 1
}
Write-Host "All $($script:checks) checks passed" -ForegroundColor Green
exit 0
