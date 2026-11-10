param([string]$Root = (Split-Path -Parent $PSScriptRoot))
# Deterministic raster export of the supplied polygon SVG. No packages, network,
# or generated artwork. Original branding files are never modified.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
[xml]$svg = Get-Content -Raw -Encoding UTF8 (Join-Path $Root 'apps/web/public/branding/favicon.svg')
function Read-PolygonPath([string]$Data) {
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.FillMode = [System.Drawing.Drawing2D.FillMode]::Alternate
    $points = New-Object 'System.Collections.Generic.List[System.Drawing.PointF]'
    foreach ($command in [regex]::Matches($Data, '([ML])(-?\d+(?:\.\d+)?),(-?\d+(?:\.\d+)?)|Z')) {
        if ($command.Value -eq 'Z') {
            if ($points.Count -ge 3) { $path.AddPolygon($points.ToArray()) }
            $points.Clear()
        } else {
            if ($command.Groups[1].Value -eq 'M' -and $points.Count -gt 0) { throw 'Expected a closed polygon' }
            $points.Add([System.Drawing.PointF]::new([single]::Parse($command.Groups[2].Value,[Globalization.CultureInfo]::InvariantCulture),[single]::Parse($command.Groups[3].Value,[Globalization.CultureInfo]::InvariantCulture)))
        }
    }
    return ,$path
}
$cat = Read-PolygonPath $svg.svg.path[1].d
$square = Read-PolygonPath $svg.svg.path[0].d
try {
    foreach ($asset in @(@{Name='icon-192.png';Size=192;Maskable=$false},@{Name='icon-512.png';Size=512;Maskable=$false},@{Name='icon-maskable-512.png';Size=512;Maskable=$true},@{Name='apple-touch-icon.png';Size=180;Maskable=$false})) {
        $bitmap = [System.Drawing.Bitmap]::new($asset.Size,$asset.Size)
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $background = [System.Drawing.Drawing2D.LinearGradientBrush]::new([System.Drawing.PointF]::new(0,0),[System.Drawing.PointF]::new(1254,1254),[System.Drawing.ColorTranslator]::FromHtml('#30013d'),[System.Drawing.ColorTranslator]::FromHtml('#31013f'))
        $foreground = [System.Drawing.Drawing2D.LinearGradientBrush]::new([System.Drawing.PointF]::new(0,0),[System.Drawing.PointF]::new(226,1254),[System.Drawing.ColorTranslator]::FromHtml('#eebcff'),[System.Drawing.ColorTranslator]::FromHtml('#e9affb'))
        try {
            $graphics.Clear([System.Drawing.ColorTranslator]::FromHtml('#30013d'))
            $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
            $graphics.ScaleTransform($asset.Size/1254.0,$asset.Size/1254.0)
            $graphics.FillPath($background,$square)
            if ($asset.Maskable) { $graphics.TranslateTransform(62.7,62.7); $graphics.ScaleTransform(0.9,0.9) }
            $graphics.FillPath($foreground,$cat)
            $bitmap.Save((Join-Path $Root ('apps/web/public/branding/' + $asset.Name)),[System.Drawing.Imaging.ImageFormat]::Png)
        } finally { $foreground.Dispose(); $background.Dispose(); $graphics.Dispose(); $bitmap.Dispose() }
        Write-Output "Exported $($asset.Name) ($($asset.Size)x$($asset.Size))"
    }
} finally { $cat.Dispose(); $square.Dispose() }
