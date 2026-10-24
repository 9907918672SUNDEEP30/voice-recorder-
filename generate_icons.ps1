Add-Type -AssemblyName System.Drawing

function Create-AppIcon {
    param(
        [string]$outputPath,
        [int]$r1, [int]$g1, [int]$b1,
        [int]$r2, [int]$g2, [int]$b2,
        [string]$title
    )
    $bmp = New-Object System.Drawing.Bitmap 1024, 1024
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality

    $c1 = [System.Drawing.Color]::FromArgb(255, $r1, $g1, $b1)
    $c2 = [System.Drawing.Color]::FromArgb(255, $r2, $g2, $b2)

    # Background gradient
    $rect = New-Object System.Drawing.Rectangle 0, 0, 1024, 1024
    $brush = New-Object System.Drawing.Drawing2D.LinearGradientBrush $rect, $c1, $c2, 45.0
    $g.FillRectangle($brush, $rect)

    # Outer decorative glow ring
    $pen = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(120, 255, 255, 255)), 12
    $g.DrawEllipse($pen, 60, 60, 904, 904)

    # Inner circular badge
    $badgeBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(50, 0, 0, 0))
    $g.FillEllipse($badgeBrush, 120, 120, 784, 784)

    $innerPen = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(200, 255, 255, 255)), 4
    $g.DrawEllipse($innerPen, 130, 130, 764, 764)

    # Title text
    $font = New-Object System.Drawing.Font("Arial", [float]64, [System.Drawing.FontStyle]::Bold)
    $format = New-Object System.Drawing.StringFormat
    $format.Alignment = [System.Drawing.StringAlignment]::Center
    $format.LineAlignment = [System.Drawing.StringAlignment]::Center
    
    $textBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
    $textRect = New-Object System.Drawing.RectangleF 0, 460, 1024, 180
    $g.DrawString($title, $font, $textBrush, $textRect, $format)

    # Subtext "SOVEREIGN"
    $fontSmall = New-Object System.Drawing.Font("Arial", [float]36, [System.Drawing.FontStyle]::Bold)
    $goldBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 245, 197, 24))
    $subRect = New-Object System.Drawing.RectangleF 0, 360, 1024, 90
    $g.DrawString("SOVEREIGN", $fontSmall, $goldBrush, $subRect, $format)

    $bmp.Save($outputPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose()
    $bmp.Dispose()
    Write-Host "Created: $outputPath"
}

# 1. File Finder (Cyan / Blue)
Create-AppIcon "$PSScriptRoot\apps\sovereign_file_finder\app-icon.png" 14 116 144 3 105 161 "FILE FINDER"

# 2. RAG Vault (Teal / Emerald)
Create-AppIcon "$PSScriptRoot\apps\sovereign_rag_vault\app-icon.png" 15 118 110 16 185 129 "RAG VAULT"

# 3. CDM Vault (Indigo / Purple)
Create-AppIcon "$PSScriptRoot\apps\sovereign_cdm_vault\app-icon.png" 67 24 255 124 58 237 "CDM VAULT"
