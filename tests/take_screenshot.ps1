$bmp = "target\screenshot_current.bmp"
$png = "target\screenshot_current.png"

if (Test-Path $bmp) { Remove-Item $bmp -Force }
if (Test-Path $png) { Remove-Item $png -Force }

Write-Host "Building actlog..."
cargo build
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed!"
    exit 1
}

Write-Host "Running actlog with screenshot..."
$proc = Start-Process -FilePath "target\debug\actlog.exe" -ArgumentList "--screenshot", $bmp -PassThru

$timeout = 15
$timer = 0
while ($timer -lt $timeout) {
    if (Test-Path $bmp) {
        Write-Host "Found BMP output!"
        break
    }
    Start-Sleep -Milliseconds 500
    $timer += 0.5
}

if (Test-Path $bmp) {
    Add-Type -AssemblyName System.Drawing
    $img = [System.Drawing.Bitmap]::FromFile((Resolve-Path $bmp).Path)
    $img.Save((Join-Path (Get-Location) $png), [System.Drawing.Imaging.ImageFormat]::Png)
    $img.Dispose()
    Write-Host "PNG saved successfully to $png"
} else {
    Write-Host "Screenshot failed to generate within timeout."
}

Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
