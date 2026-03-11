# Meridian Icon Creator (Windows PowerShell)
# Generates all required icon files using System.Drawing (no external tools needed)
# Usage: powershell -ExecutionPolicy Bypass -File scripts\create-icons.ps1

Add-Type -AssemblyName System.Drawing

$IconsDir = Join-Path $PSScriptRoot "..\src-tauri\icons"
New-Item -ItemType Directory -Force -Path $IconsDir | Out-Null

function New-IconPng {
    param (
        [int]    $Size,
        [string] $OutputPath
    )

    $bmp = New-Object System.Drawing.Bitmap($Size, $Size)
    $g   = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode    = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAlias

    # Gradient background  indigo-600 -> violet-600
    $rect  = New-Object System.Drawing.RectangleF(0, 0, $Size, $Size)
    $brush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
        $rect,
        [System.Drawing.Color]::FromArgb(255, 79,  70,  229),
        [System.Drawing.Color]::FromArgb(255, 124, 58,  237),
        [System.Drawing.Drawing2D.LinearGradientMode]::ForwardDiagonal
    )

    # Rounded rectangle background
    $r    = [int]($Size * 0.22)
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.AddArc(0,          0,          $r*2, $r*2, 180, 90)
    $path.AddArc($Size-$r*2, 0,          $r*2, $r*2, 270, 90)
    $path.AddArc($Size-$r*2, $Size-$r*2, $r*2, $r*2, 0,   90)
    $path.AddArc(0,          $Size-$r*2, $r*2, $r*2, 90,  90)
    $path.CloseFigure()
    $g.FillPath($brush, $path)

    # White letter M centred
    $fs   = [float]($Size * 0.55)
    $font = New-Object System.Drawing.Font("Arial", $fs, [System.Drawing.FontStyle]::Bold)
    $tb   = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
    $sf   = New-Object System.Drawing.StringFormat
    $sf.Alignment     = [System.Drawing.StringAlignment]::Center
    $sf.LineAlignment = [System.Drawing.StringAlignment]::Center
    $g.DrawString("M", $font, $tb, $rect, $sf)

    $bmp.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose()
    $bmp.Dispose()

    $kb = [int]([System.IO.FileInfo]$OutputPath).Length / 1KB
    Write-Host "[OK] $OutputPath  ($Size x $Size px, ${kb} KB)"
}

# Standard sizes
foreach ($sz in @(16, 32, 64, 128, 256)) {
    $outFile = Join-Path $IconsDir ("$sz" + "x" + "$sz" + ".png")
    New-IconPng -Size $sz -OutputPath $outFile
}

# Retina / HiDPI
New-IconPng -Size 256 -OutputPath (Join-Path $IconsDir "128x128@2x.png")

# System-tray icon
New-IconPng -Size 32  -OutputPath (Join-Path $IconsDir "tray-icon.png")

# ---- Build a minimal .ico from the 256-px PNG --------------------------------
$pngPath   = Join-Path $IconsDir "256x256.png"
$icoPath   = Join-Path $IconsDir "icon.ico"
$pngBitmap = New-Object System.Drawing.Bitmap($pngPath)

$pngStream = New-Object System.IO.MemoryStream
$pngBitmap.Save($pngStream, [System.Drawing.Imaging.ImageFormat]::Png)
$pngBytes  = $pngStream.ToArray()
$pngBitmap.Dispose()

# ICO file layout:
#   6-byte ICONDIR  +  16-byte ICONDIRENTRY  +  PNG data
$icoStream = [System.IO.File]::Open($icoPath,
    [System.IO.FileMode]::Create,
    [System.IO.FileAccess]::Write)
$w = New-Object System.IO.BinaryWriter($icoStream)

# ICONDIR
$w.Write([uint16]0)    # reserved = 0
$w.Write([uint16]1)    # type     = 1 (icon)
$w.Write([uint16]1)    # count    = 1 image

# ICONDIRENTRY (16 bytes)
$w.Write([byte]0)      # width  (0 means 256)
$w.Write([byte]0)      # height (0 means 256)
$w.Write([byte]0)      # color count
$w.Write([byte]0)      # reserved
$w.Write([uint16]1)    # planes
$w.Write([uint16]32)   # bit-count
$w.Write([uint32]$pngBytes.Length)   # image data size
$w.Write([uint32]22)                  # image data offset (6 + 16)

# PNG data
$w.Write($pngBytes)
$w.Close()
$icoStream.Close()

$icoKb = [int]([System.IO.FileInfo]$icoPath).Length / 1KB
Write-Host "[OK] $icoPath  (${icoKb} KB)"

# ---- macOS .icns (placeholder -- run on macOS) -------------------------------
$icnsPath = Join-Path $IconsDir "icon.icns"
# Copy 128x128 PNG as a placeholder so Tauri can build on macOS later
Copy-Item (Join-Path $IconsDir "128x128.png") $icnsPath -Force
Write-Host "[PLACEHOLDER] $icnsPath  (replace with real .icns on macOS)"

Write-Host ""
Write-Host "All icons generated in: $IconsDir"
Write-Host ""
Write-Host "Next steps:"
Write-Host "  npm install"
Write-Host "  npm run tauri:build"
