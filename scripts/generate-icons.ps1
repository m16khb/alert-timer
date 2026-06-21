Add-Type -AssemblyName System.Drawing
$ErrorActionPreference = "Stop"

$iconDir = Join-Path $PSScriptRoot "..\src-tauri\icons"
New-Item -ItemType Directory -Force -Path $iconDir | Out-Null

function New-RoundedRectanglePath {
    param(
        [single]$X,
        [single]$Y,
        [single]$Width,
        [single]$Height,
        [single]$Radius
    )

    $path = [System.Drawing.Drawing2D.GraphicsPath]::new()
    $diameter = $Radius * 2
    $path.AddArc($X, $Y, $diameter, $diameter, 180, 90)
    $path.AddArc(($X + $Width - $diameter), $Y, $diameter, $diameter, 270, 90)
    $path.AddArc(($X + $Width - $diameter), ($Y + $Height - $diameter), $diameter, $diameter, 0, 90)
    $path.AddArc($X, ($Y + $Height - $diameter), $diameter, $diameter, 90, 90)
    $path.CloseFigure()
    return $path
}

function New-IconPen {
    param(
        [System.Drawing.Color]$Color,
        [single]$Width
    )

    $pen = [System.Drawing.Pen]::new($Color, $Width)
    $pen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $pen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    return $pen
}

function New-AlertTimerBitmap {
    param([int]$Size)

    $canvasSize = $Size * 4
    $canvas = [System.Drawing.Bitmap]::new($canvasSize, $canvasSize, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [System.Drawing.Graphics]::FromImage($canvas)
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $graphics.Clear([System.Drawing.Color]::Transparent)

    $s = [single]$canvasSize
    $outerPad = $s * 0.055
    $outerSize = $s - ($outerPad * 2)
    $outerRadius = $s * 0.215
    $outerRect = [System.Drawing.RectangleF]::new($outerPad, $outerPad, $outerSize, $outerSize)
    $shadowPath = New-RoundedRectanglePath ($outerPad + ($s * 0.022)) ($outerPad + ($s * 0.03)) $outerSize $outerSize $outerRadius
    $outerPath = New-RoundedRectanglePath $outerPad $outerPad $outerSize $outerSize $outerRadius

    $shadowBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(105, 0, 0, 0))
    $graphics.FillPath($shadowBrush, $shadowPath)

    $outerBrush = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
        $outerRect,
        [System.Drawing.Color]::FromArgb(255, 255, 62, 94),
        [System.Drawing.Color]::FromArgb(255, 255, 187, 72),
        38.0
    )
    $graphics.FillPath($outerBrush, $outerPath)

    $innerPad = $s * 0.135
    $innerSize = $s - ($innerPad * 2)
    $innerRadius = $s * 0.155
    $innerRect = [System.Drawing.RectangleF]::new($innerPad, $innerPad, $innerSize, $innerSize)
    $innerPath = New-RoundedRectanglePath $innerPad $innerPad $innerSize $innerSize $innerRadius
    $innerBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(255, 12, 18, 23))
    $graphics.FillPath($innerBrush, $innerPath)

    $innerOutline = New-IconPen ([System.Drawing.Color]::FromArgb(80, 255, 255, 255)) ($s * 0.013)
    $graphics.DrawPath($innerOutline, $innerPath)

    $glowRect = [System.Drawing.RectangleF]::new($innerPad, $innerPad, $innerSize, $innerSize * 0.52)
    $glowBrush = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
        $glowRect,
        [System.Drawing.Color]::FromArgb(56, 48, 222, 206),
        [System.Drawing.Color]::FromArgb(0, 48, 222, 206),
        90.0
    )
    $graphics.FillPath($glowBrush, $innerPath)

    $centerX = $s * 0.5
    $centerY = $s * 0.505
    $ringRadius = $s * 0.235
    $ringRect = [System.Drawing.RectangleF]::new(
        ($centerX - $ringRadius),
        ($centerY - $ringRadius),
        ($ringRadius * 2),
        ($ringRadius * 2)
    )

    $trackPen = New-IconPen ([System.Drawing.Color]::FromArgb(255, 34, 49, 58)) ($s * 0.058)
    $graphics.DrawArc($trackPen, $ringRect, -90, 360)

    $alertPen = New-IconPen ([System.Drawing.Color]::FromArgb(255, 255, 76, 107)) ($s * 0.073)
    $graphics.DrawArc($alertPen, $ringRect, -92, 285)

    $warmPen = New-IconPen ([System.Drawing.Color]::FromArgb(255, 255, 200, 86)) ($s * 0.046)
    $graphics.DrawArc($warmPen, $ringRect, 194, 72)

    $handPen = New-IconPen ([System.Drawing.Color]::FromArgb(255, 234, 248, 246)) ($s * 0.038)
    $graphics.DrawLine($handPen, $centerX, $centerY, ($centerX + ($ringRadius * 0.42)), ($centerY - ($ringRadius * 0.52)))
    $graphics.DrawLine($handPen, $centerX, $centerY, ($centerX - ($ringRadius * 0.28)), ($centerY - ($ringRadius * 0.18)))

    $dotSize = $s * 0.092
    $dotRect = [System.Drawing.RectangleF]::new(($centerX - ($dotSize / 2)), ($centerY - ($dotSize / 2)), $dotSize, $dotSize)
    $dotBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(255, 240, 252, 250))
    $graphics.FillEllipse($dotBrush, $dotRect)

    $sparkSize = $s * 0.105
    $sparkRect = [System.Drawing.RectangleF]::new(($centerX - ($sparkSize / 2)), ($innerPad - ($sparkSize * 0.18)), $sparkSize, $sparkSize)
    $sparkBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(255, 255, 224, 128))
    $graphics.FillEllipse($sparkBrush, $sparkRect)

    $sparkBrush.Dispose()
    $dotBrush.Dispose()
    $handPen.Dispose()
    $warmPen.Dispose()
    $alertPen.Dispose()
    $trackPen.Dispose()
    $glowBrush.Dispose()
    $innerOutline.Dispose()
    $innerBrush.Dispose()
    $innerPath.Dispose()
    $outerBrush.Dispose()
    $shadowBrush.Dispose()
    $outerPath.Dispose()
    $shadowPath.Dispose()
    $graphics.Dispose()

    if ($canvasSize -eq $Size) {
        return $canvas
    }

    $bitmap = [System.Drawing.Bitmap]::new($Size, $Size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $resizeGraphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $resizeGraphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $resizeGraphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    $resizeGraphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $resizeGraphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $resizeGraphics.Clear([System.Drawing.Color]::Transparent)
    $resizeGraphics.DrawImage($canvas, 0, 0, $Size, $Size)
    $resizeGraphics.Dispose()
    $canvas.Dispose()

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
