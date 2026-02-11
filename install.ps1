$REPO = "cashlycash/peapod-v2"
Write-Host "🫛 PeaPod Installer (Alpha)"

Write-Host "Detecting latest release..."
$Latest = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest"
$Tag = $Latest.tag_name

if (!$Tag) {
    Write-Error "Could not find latest release."
    exit 1
}

Write-Host "Found version: $Tag"

# Find .msi or .exe
$Asset = $Latest.assets | Where-Object { $_.name -match ".msi$" -or $_.name -match ".exe$" } | Select-Object -First 1

if (!$Asset) {
    Write-Error "Could not find Windows binary."
    Write-Host "Visit: https://github.com/$REPO/releases/tag/$Tag"
    exit 1
}

$Url = $Asset.browser_download_url
$OutFile = $Asset.name

Write-Host "Downloading $Url..."
Invoke-WebRequest -Uri $Url -OutFile $OutFile

Write-Host "✅ Download complete: $OutFile"
Write-Host "Running installer..."
Start-Process -FilePath $OutFile
