# download_sidecars.ps1
# Downloads yt-dlp, ffmpeg, and deno to src-tauri/binaries/ with target-triple naming.
# Run this before `npm run tauri dev` or `npm run tauri build`.

$ErrorActionPreference = "Stop"

# Force TLS 1.2 to avoid connection errors
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$BINARIES_DIR = Join-Path $PSScriptRoot "..\src-tauri\binaries"
# Normalize path
$BINARIES_DIR = [System.IO.Path]::GetFullPath($BINARIES_DIR)

$TARGET_TRIPLE = "x86_64-pc-windows-msvc"

Write-Host "Target Binaries Directory: $BINARIES_DIR" -ForegroundColor Gray

# Create binaries directory
if (-not (Test-Path $BINARIES_DIR)) {
  New-Item -ItemType Directory -Path $BINARIES_DIR -Force | Out-Null
}

# --- yt-dlp ---
$YTDLP_URL = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
$YTDLP_PATH = Join-Path $BINARIES_DIR "yt-dlp-$TARGET_TRIPLE.exe"

if (-not (Test-Path $YTDLP_PATH)) {
  Write-Host "Downloading yt-dlp..." -ForegroundColor Cyan
  Invoke-WebRequest -Uri $YTDLP_URL -OutFile $YTDLP_PATH -UseBasicParsing
  Write-Host "  -> Saved to $YTDLP_PATH" -ForegroundColor Green
}
else {
  Write-Host "yt-dlp already exists." -ForegroundColor Gray
}

# --- ffmpeg ---
# Using BtbN's static builds (GPL)
$FFMPEG_ZIP_URL = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
$FFMPEG_ZIP = Join-Path $env:TEMP "ffmpeg-latest.zip"
$FFMPEG_EXTRACT = Join-Path $env:TEMP "ffmpeg-extract"
$FFMPEG_PATH = Join-Path $BINARIES_DIR "ffmpeg-$TARGET_TRIPLE.exe"

if (-not (Test-Path $FFMPEG_PATH)) {
  Write-Host "Downloading ffmpeg..." -ForegroundColor Cyan
  Invoke-WebRequest -Uri $FFMPEG_ZIP_URL -OutFile $FFMPEG_ZIP -UseBasicParsing
    
  Write-Host "  -> Extracting..." -ForegroundColor Cyan
  if (Test-Path $FFMPEG_EXTRACT) { Remove-Item -Path $FFMPEG_EXTRACT -Recurse -Force }
  Expand-Archive -Path $FFMPEG_ZIP -DestinationPath $FFMPEG_EXTRACT -Force
    
  $ffmpegExe = Get-ChildItem -Path $FFMPEG_EXTRACT -Recurse -Filter "ffmpeg.exe" | Select-Object -First 1
  if ($ffmpegExe) {
    Copy-Item -Path $ffmpegExe.FullName -Destination $FFMPEG_PATH
    Write-Host "  -> Saved to $FFMPEG_PATH" -ForegroundColor Green
  }
  else {
    Write-Error "ffmpeg.exe not found in archive!"
  }
    
  # Cleanup
  Remove-Item -Path $FFMPEG_ZIP -Force -ErrorAction SilentlyContinue
  Remove-Item -Path $FFMPEG_EXTRACT -Recurse -Force -ErrorAction SilentlyContinue
}
else {
  Write-Host "ffmpeg already exists." -ForegroundColor Gray
}

# --- Deno ---
$DENO_URL = "https://github.com/denoland/deno/releases/download/v2.6.10/deno-x86_64-pc-windows-msvc.zip"
$DENO_ZIP = Join-Path $env:TEMP "deno.zip"
$DENO_PATH = Join-Path $BINARIES_DIR "deno-$TARGET_TRIPLE.exe"

if (-not (Test-Path $DENO_PATH)) {
  Write-Host "Downloading Deno..." -ForegroundColor Cyan
  Invoke-WebRequest -Uri $DENO_URL -OutFile $DENO_ZIP -UseBasicParsing
    
  Write-Host "  -> Extracting..." -ForegroundColor Cyan
  Expand-Archive -Path $DENO_ZIP -DestinationPath $env:TEMP -Force
    
  $extractedDeno = Join-Path $env:TEMP "deno.exe"
  if (Test-Path $extractedDeno) {
    Move-Item -Path $extractedDeno -Destination $DENO_PATH -Force
    Write-Host "  -> Saved to $DENO_PATH" -ForegroundColor Green
  }
  else {
    Write-Error "deno.exe not found in archive!"
  }
    
  # Cleanup
  Remove-Item -Path $DENO_ZIP -Force -ErrorAction SilentlyContinue
}
else {
  Write-Host "Deno already exists." -ForegroundColor Gray
}

Write-Host ""
Write-Host "All sidecar binaries are ready!" -ForegroundColor Green
