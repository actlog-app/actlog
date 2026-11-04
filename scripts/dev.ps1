# One-click dev & test launcher for ActLog + Chrome Extension
param(
    [switch]$Release
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

Write-Host "==> Compiling ActLog..." -ForegroundColor Cyan
if ($Release) {
    cargo build --release
    $binPath = "$root\target\release\actlog.exe"
} else {
    cargo build
    $binPath = "$root\target\debug\actlog.exe"
}

# Find Chrome / Edge binary
$chromePaths = @(
    "C:\Program Files\Google\Chrome\Application\chrome.exe",
    "C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    "$env:LOCALAPPDATA\Google\Chrome\Application\chrome.exe",
    "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
    "C:\Program Files\Microsoft\Edge\Application\msedge.exe"
)

$browserExe = $null
foreach ($path in $chromePaths) {
    if (Test-Path $path) {
        $browserExe = $path
        break
    }
}

$extPath = "$root\extension"
$tempProfile = "$root\.temp\dev_browser_profile"

Write-Host "==> Launching ActLog Desktop App..." -ForegroundColor Green
$appProc = Start-Process -FilePath $binPath -PassThru

if ($browserExe) {
    Write-Host "==> Launching Browser with Auto-Loaded Extension ($browserExe)..." -ForegroundColor Green
    Start-Process -FilePath $browserExe -ArgumentList `
        "--load-extension=`"$extPath`"", `
        "--user-data-dir=`"$tempProfile`"", `
        "--no-first-run", `
        "--no-default-browser-check", `
        "http://127.0.0.1:5577/dashboard"
} else {
    Write-Host "==> Browser executable not found. Open http://127.0.0.1:5577/dashboard manually." -ForegroundColor Yellow
}

Write-Host "==> Dev environment running! Press Ctrl+C to stop app." -ForegroundColor Cyan
$appProc.WaitForExit()

