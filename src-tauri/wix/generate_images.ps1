Add-Type -AssemblyName System.Drawing
$width = 493
$bannerHeight = 58
$dialogHeight = 312
$white = [System.Drawing.Color]::White
$darkColor = [System.Drawing.Color]::FromArgb(17, 24, 39)
$accentColor = [System.Drawing.Color]::FromArgb(0, 209, 255)

$brushWhite = New-Object System.Drawing.SolidBrush($white)
$brushDark = New-Object System.Drawing.SolidBrush($darkColor)
$brushAccent = New-Object System.Drawing.SolidBrush($accentColor)

# Paths
$iconPath = Join-Path $PSScriptRoot "..\icons\icon.png"
if (-not (Test-Path $iconPath)) { 
    Write-Error "Icon not found at $iconPath"
    exit 1
}
$logo = [System.Drawing.Image]::FromFile($iconPath)

# ---------------------------------------------------------
# Banner Generation
# ---------------------------------------------------------
$bmpBanner = New-Object System.Drawing.Bitmap $width, $bannerHeight
$gBanner = [System.Drawing.Graphics]::FromImage($bmpBanner)
$gBanner.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic

# Background: White
$gBanner.FillRectangle($brushWhite, 0, 0, $width, $bannerHeight)

# Styled Strip on Right
$stripWidth = 80
$accentWidth = 4
$stripX = $width - $stripWidth
$gBanner.FillRectangle($brushDark, $stripX, 0, $stripWidth, $bannerHeight)
$gBanner.FillRectangle($brushAccent, $stripX - $accentWidth, 0, $accentWidth, $bannerHeight)

# Draw Logo centered in the dark strip
$logoSize = 48
$logoX = $stripX + ($stripWidth - $logoSize) / 2
$logoY = ($bannerHeight - $logoSize) / 2
$gBanner.DrawImage($logo, $logoX, $logoY, $logoSize, $logoSize)

$bmpBanner.Save("$PSScriptRoot\Banner.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$gBanner.Dispose()
$bmpBanner.Dispose()

# ---------------------------------------------------------
# Dialog Generation
# ---------------------------------------------------------
$bmpDialog = New-Object System.Drawing.Bitmap $width, $dialogHeight
$gDialog = [System.Drawing.Graphics]::FromImage($bmpDialog)
$gDialog.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic

# Background: White
$gDialog.FillRectangle($brushWhite, 0, 0, $width, $dialogHeight)

# Styled Sidebar on Left
$sidebarWidth = 164
$gDialog.FillRectangle($brushDark, 0, 0, $sidebarWidth, $dialogHeight)
$gDialog.FillRectangle($brushAccent, $sidebarWidth, 0, $accentWidth, $dialogHeight)

# Draw Logo centered in top area of sidebar
$logoSize = 96
$logoX = ($sidebarWidth - $logoSize) / 2
$logoY = 30
$gDialog.DrawImage($logo, $logoX, $logoY, $logoSize, $logoSize)

$bmpDialog.Save("$PSScriptRoot\Dialog.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$gDialog.Dispose()
$bmpDialog.Dispose()
$logo.Dispose()

Write-Host "Images generated successfully with Logo in $PSScriptRoot"
