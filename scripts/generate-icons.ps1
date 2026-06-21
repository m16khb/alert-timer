Add-Type -AssemblyName System.Drawing
$ErrorActionPreference = "Stop"

$iconDir = Join-Path $PSScriptRoot "..\src-tauri\icons"
New-Item -ItemType Directory -Force -Path $iconDir | Out-Null

function New-AlertTimerBitmap {
    param([int]$Size)

    $bitmap = New-Object System.Drawing.Bitmap $Size, $Size
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $graphics.Clear([System.Drawing.Color]::FromArgb(16, 17, 19))

    $rect = [System.Drawing.Rectangle]::new(0, 0, $Size, $Size)
    $textRect = [System.Drawing.RectangleF]::new(0, 0, [float]$Size, [float]$Size)
    $brush = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
        $rect,
        [System.Drawing.Color]::FromArgb(239, 68, 68),
        [System.Drawing.Color]::FromArgb(244, 183, 64),
        45.0
    )
    $graphics.FillRectangle($brush, $rect)

    $inner = [Math]::Max(3, [int]($Size * 0.12))
    $innerRect = [System.Drawing.Rectangle]::new($inner, $inner, ($Size - ($inner * 2)), ($Size - ($inner * 2)))
    $innerBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(20, 21, 24))
    $graphics.FillRectangle($innerBrush, $innerRect)

    $fontSize = [Math]::Max(8, [int]($Size * 0.32))
    $font = New-Object System.Drawing.Font "Segoe UI", $fontSize, ([System.Drawing.FontStyle]::Bold), ([System.Drawing.GraphicsUnit]::Pixel)
    $textBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
    $format = New-Object System.Drawing.StringFormat
    $format.Alignment = [System.Drawing.StringAlignment]::Center
    $format.LineAlignment = [System.Drawing.StringAlignment]::Center
    $graphics.DrawString("AT", $font, $textBrush, $textRect, $format)

    $format.Dispose()
    $textBrush.Dispose()
    $font.Dispose()
    $innerBrush.Dispose()
    $brush.Dispose()
    $graphics.Dispose()

    return $bitmap
}

foreach ($size in @(32, 128)) {
    $bitmap = New-AlertTimerBitmap -Size $size
    $bitmap.Save((Join-Path $iconDir "$($size)x$($size).png"), [System.Drawing.Imaging.ImageFormat]::Png)
    $bitmap.Dispose()
}

$icoBitmap = New-AlertTimerBitmap -Size 256
$icon = [System.Drawing.Icon]::FromHandle($icoBitmap.GetHicon())
$stream = [System.IO.File]::Create((Join-Path $iconDir "icon.ico"))
$icon.Save($stream)
$stream.Dispose()
$icon.Dispose()
$icoBitmap.Dispose()
