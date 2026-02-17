$ErrorActionPreference = "Stop"

# Force TLS 1.2 to avoid connection errors
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$url = "https://github.com/quickjs-ng/quickjs/releases/download/v0.12.1/qjs-windows-x86_64.exe"
# Note: Script is likely running from root, but let's be robust
$root = Get-Location
$destDir = Join-Path $root "src-tauri/binaries"
$targetName = "qjs-x86_64-pc-windows-msvc.exe"
$destFile = Join-Path $destDir $targetName

# Create binaries dir if not exists
if (-not (Test-Path $destDir)) {
    New-Item -ItemType Directory -Path $destDir | Out-Null
}

Write-Host "Downloading QuickJS from $url..."
Invoke-WebRequest -Uri $url -OutFile $destFile

if (Test-Path $destFile) {
    Write-Host "QuickJS installed successfully at $destFile"
    $fileItem = Get-Item $destFile
    Write-Host "Size: $($fileItem.Length) bytes"
} else {
    Write-Error "Failed to download file."
}
