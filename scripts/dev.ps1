# One-click dev & test launcher for ActLog Desktop App
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

Write-Host "==> Launching ActLog Desktop App..." -ForegroundColor Green
$appProc = Start-Process -FilePath $binPath -PassThru

Write-Host "==> ActLog running! Press Ctrl+C or close window to stop." -ForegroundColor Cyan
$appProc.WaitForExit()
