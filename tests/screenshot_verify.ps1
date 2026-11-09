Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$code = @"
using System;
using System.Drawing;
using System.Runtime.InteropServices;

public class Win32 {
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdcBlt, uint nFlags);

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }
}
"@
Add-Type -TypeDefinition $code

Write-Host "Launching actlog.exe..."
$proc = Start-Process -FilePath "target\debug\actlog.exe" -PassThru
Start-Sleep -Seconds 3

try {
    $hWnd = $proc.MainWindowHandle
    if ($hWnd -eq [IntPtr]::Zero) {
        $found = Get-Process -Name "actlog" | Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero }
        if ($found) {
            $hWnd = $found.MainWindowHandle
        }
    }

    if ($hWnd -ne [IntPtr]::Zero) {
        $rect = New-Object Win32+RECT
        [Win32]::GetWindowRect($hWnd, [ref]$rect) | Out-Null

        $width = $rect.Right - $rect.Left
        $height = $rect.Bottom - $rect.Top

        Write-Host "Window size: ${width}x${height} at ($($rect.Left), $($rect.Top))"

        if ($width -gt 0 -and $height -gt 0) {
            $bitmap = New-Object System.Drawing.Bitmap($width, $height)
            $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
            $hdc = $graphics.GetHdc()
            
            # PW_RENDERFULLCONTENT = 2
            $success = [Win32]::PrintWindow($hWnd, $hdc, 2)
            $graphics.ReleaseHdc($hdc)

            $outPath = Join-Path (Get-Location) "target\screenshot_current.png"
            $bitmap.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Png)
            $graphics.Dispose()
            $bitmap.Dispose()

            Write-Host "Screenshot successfully saved (success=$success) to: $outPath"
        } else {
            Write-Warning "Invalid window dimensions."
        }
    } else {
        Write-Warning "Could not find MainWindowHandle for actlog."
    }
} finally {
    Write-Host "Stopping actlog..."
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
}
