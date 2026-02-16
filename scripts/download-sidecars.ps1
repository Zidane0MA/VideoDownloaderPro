#!/usr/bin/env pwsh
# download-sidecars.ps1
# Downloads yt-dlp.exe and ffmpeg.exe to src-tauri/binaries/ with target-triple naming.
# Run this script before `npm run tauri dev` or `npm run tauri build`.

$ErrorActionPreference = "Stop"

$BINARIES_DIR = Join-Path $PSScriptRoot "src-tauri" "binaries"
$TARGET_TRIPLE = "x86_64-pc-windows-msvc"

# Create binaries directory
if (-not (Test-Path $BINARIES_DIR)) {
    New-Item -ItemType Directory -Path $BINARIES_DIR -Force | Out-Null
}

# --- yt-dlp ---
$YTDLP_URL = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
$YTDLP_PATH = Join-Path $BINARIES_DIR "yt-dlp-$TARGET_TRIPLE.exe"

if (-not (Test-Path $YTDLP_PATH)) {
    Write-Host "Downloading yt-dlp.exe..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri $YTDLP_URL -OutFile $YTDLP_PATH -UseBasicParsing
    Write-Host "  -> Saved to $YTDLP_PATH" -ForegroundColor Green
} else {
    Write-Host "yt-dlp.exe already exists, skipping." -ForegroundColor Yellow
}

# --- ffmpeg ---
# Using BtbN's static builds (licensed under GPL)
$FFMPEG_ZIP_URL = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
$FFMPEG_ZIP = Join-Path $env:TEMP "ffmpeg-latest.zip"
$FFMPEG_EXTRACT = Join-Path $env:TEMP "ffmpeg-extract"
$FFMPEG_PATH = Join-Path $BINARIES_DIR "ffmpeg-$TARGET_TRIPLE.exe"

if (-not (Test-Path $FFMPEG_PATH)) {
    Write-Host "Downloading ffmpeg..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri $FFMPEG_ZIP_URL -OutFile $FFMPEG_ZIP -UseBasicParsing
    Write-Host "  -> Extracting..." -ForegroundColor Cyan
    
    if (Test-Path $FFMPEG_EXTRACT) {
        Remove-Item -Path $FFMPEG_EXTRACT -Recurse -Force
    }
    Expand-Archive -Path $FFMPEG_ZIP -DestinationPath $FFMPEG_EXTRACT -Force
    
    # Find ffmpeg.exe in the extracted folder
    $ffmpegExe = Get-ChildItem -Path $FFMPEG_EXTRACT -Recurse -Filter "ffmpeg.exe" | Select-Object -First 1
    if ($ffmpegExe) {
        Copy-Item -Path $ffmpegExe.FullName -Destination $FFMPEG_PATH
        Write-Host "  -> Saved to $FFMPEG_PATH" -ForegroundColor Green
    } else {
        Write-Host "  ERROR: ffmpeg.exe not found in archive!" -ForegroundColor Red
        exit 1
    }
    
    # Cleanup
    Remove-Item -Path $FFMPEG_ZIP -Force -ErrorAction SilentlyContinue
    Remove-Item -Path $FFMPEG_EXTRACT -Recurse -Force -ErrorAction SilentlyContinue
} else {
    Write-Host "ffmpeg.exe already exists, skipping." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Sidecar binaries ready!" -ForegroundColor Green
Write-Host "  yt-dlp:  $YTDLP_PATH"
Write-Host "  ffmpeg:  $FFMPEG_PATH"
