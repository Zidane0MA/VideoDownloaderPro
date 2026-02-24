---
name: ffmpeg-media-processing
description: "FFmpeg integration patterns for Tauri Sidecar: thumbnail generation, video/audio merging, format conversion, and media probing with ffprobe. Use when implementing post-download processing, thumbnail resizing, format conversion, or extracting media metadata in VideoDownloaderPro."
---

# FFmpeg Media Processing Patterns for Tauri

Production patterns for using `ffmpeg` and `ffprobe` as Sidecar binaries from a Tauri v2 Rust backend. Covers thumbnail generation, video/audio merging, format conversion, and metadata extraction.

## When to Use This Skill

- Generating reduced thumbnails for the Wall gallery view
- Merging separate video + audio streams (common in 4K YouTube downloads)
- Converting between container formats (WebM → MP4)
- Extracting audio from video files
- Probing media metadata (dimensions, duration, codec info)
- Generating video previews or GIF thumbnails

## Core Architecture

```
Download Complete → Rust post-processor → spawn ffmpeg/ffprobe → update DB → emit event
```

The `ffmpeg.exe` and `ffprobe.exe` binaries live at `app_data/binaries/`. They are used exclusively for post-download processing, never during the download itself (that's yt-dlp's job).

---

## Pattern 1: Spawning FFmpeg / FFprobe

```rust
use std::path::PathBuf;
use tokio::process::Command;

fn ffmpeg_path(app_handle: &tauri::AppHandle) -> PathBuf {
    app_handle.path().app_data_dir().unwrap()
        .join("binaries").join("ffmpeg.exe")
}

fn ffprobe_path(app_handle: &tauri::AppHandle) -> PathBuf {
    app_handle.path().app_data_dir().unwrap()
        .join("binaries").join("ffprobe.exe")
}

/// Run ffmpeg with given args. Returns Ok(()) on success.
async fn run_ffmpeg(
    binary: &PathBuf,
    args: &[&str],
) -> Result<String, MediaError> {
    let output = Command::new(binary)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .await
        .map_err(|e| MediaError::ProcessSpawn(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MediaError::FfmpegError(stderr.to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

---

## Pattern 2: Thumbnail Generation (Wall View)

The Wall gallery needs small thumbnails (300px wide) for fast rendering. Original thumbnails from yt-dlp are full-size.

```rust
/// Generate a reduced thumbnail for the Wall view.
/// Input:  full-size thumbnail from yt-dlp (e.g., Video.mp4.thumbnail.jpg)
/// Output: 300px wide thumbnail (e.g., Video.mp4.thumb_sm.jpg)
async fn generate_wall_thumbnail(
    ffmpeg: &PathBuf,
    input_path: &PathBuf,
    output_path: &PathBuf,
) -> Result<(), MediaError> {
    let input = input_path.to_string_lossy();
    let output = output_path.to_string_lossy();

    run_ffmpeg(ffmpeg, &[
        "-i", &input,
        "-vf", "scale=300:-1",       // 300px wide, maintain aspect ratio
        "-q:v", "3",                 // JPEG quality (2-5 is good, lower = better)
        "-y",                        // Overwrite if exists
        &output,
    ]).await?;

    Ok(())
}

/// Generate a thumbnail from a video frame (when no thumbnail was downloaded).
async fn generate_thumbnail_from_video(
    ffmpeg: &PathBuf,
    video_path: &PathBuf,
    output_path: &PathBuf,
    timestamp: &str,  // e.g., "00:00:05" — 5 seconds in
) -> Result<(), MediaError> {
    let input = video_path.to_string_lossy();
    let output = output_path.to_string_lossy();

    run_ffmpeg(ffmpeg, &[
        "-ss", timestamp,            // Seek to timestamp
        "-i", &input,
        "-vframes", "1",             // Extract single frame
        "-vf", "scale=300:-1",
        "-q:v", "3",
        "-y",
        &output,
    ]).await?;

    Ok(())
}
```

### Key flags
- `-vf scale=300:-1` — Scale to 300px width, auto-calculate height to maintain aspect ratio.
- `-q:v 3` — JPEG quality level (2 = highest quality, 31 = lowest). 3 is a good balance.
- `-ss` before `-i` — Fast seek (input seeking), much faster than output seeking.
- `-y` — Always overwrite. Safe because we control the output path.

---

## Pattern 3: Media Probing with FFprobe

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MediaProbe {
    pub streams: Vec<StreamInfo>,
    pub format: FormatInfo,
}

#[derive(Debug, Deserialize)]
pub struct StreamInfo {
    pub index: u32,
    pub codec_type: String,       // "video", "audio", "subtitle"
    pub codec_name: String,       // "h264", "aac", "vp9"
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration: Option<String>, // seconds as string
    pub bit_rate: Option<String>,
    pub r_frame_rate: Option<String>,  // "30/1", "24000/1001"
}

#[derive(Debug, Deserialize)]
pub struct FormatInfo {
    pub filename: String,
    pub format_name: String,      // "mov,mp4,m4a,3gp,3g2,mj2"
    pub duration: Option<String>,
    pub size: Option<String>,     // bytes as string
    pub bit_rate: Option<String>,
}

/// Probe a media file for metadata.
async fn probe_media(
    ffprobe: &PathBuf,
    file_path: &PathBuf,
) -> Result<MediaProbe, MediaError> {
    let path = file_path.to_string_lossy();

    let output = Command::new(ffprobe)
        .args(&[
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            &path,
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .creation_flags(0x08000000)
        .output()
        .await
        .map_err(|e| MediaError::ProcessSpawn(e.to_string()))?;

    let probe: MediaProbe = serde_json::from_slice(&output.stdout)
        .map_err(|e| MediaError::ParseError(format!("ffprobe JSON parse failed: {}", e)))?;

    Ok(probe)
}

/// Extract dimensions, duration, and file size from a probed media file.
pub fn extract_media_info(probe: &MediaProbe) -> MediaInfo {
    let video_stream = probe.streams.iter().find(|s| s.codec_type == "video");

    MediaInfo {
        width: video_stream.and_then(|s| s.width),
        height: video_stream.and_then(|s| s.height),
        duration: probe.format.duration
            .as_ref()
            .and_then(|d| d.parse::<f64>().ok()),
        file_size: probe.format.size
            .as_ref()
            .and_then(|s| s.parse::<u64>().ok()),
        codec: video_stream.map(|s| s.codec_name.clone()),
    }
}

#[derive(Debug)]
pub struct MediaInfo {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration: Option<f64>,   // seconds
    pub file_size: Option<u64>,  // bytes
    pub codec: Option<String>,
}
```

---

## Pattern 4: Video + Audio Merge (Post-Processing)

When yt-dlp downloads 4K+ content, it often downloads video and audio as separate files. While yt-dlp can merge them itself (if ffmpeg is in PATH), sometimes you need manual control.

```rust
/// Merge separate video and audio files into a single container.
/// Common when downloading 4K YouTube content (video: .webm, audio: .m4a).
async fn merge_video_audio(
    ffmpeg: &PathBuf,
    video_path: &PathBuf,
    audio_path: &PathBuf,
    output_path: &PathBuf,
) -> Result<(), MediaError> {
    let video = video_path.to_string_lossy();
    let audio = audio_path.to_string_lossy();
    let output = output_path.to_string_lossy();

    run_ffmpeg(ffmpeg, &[
        "-i", &video,
        "-i", &audio,
        "-c", "copy",           // No re-encoding (fast, lossless)
        "-movflags", "+faststart",  // Optimize for streaming/seeking
        "-y",
        &output,
    ]).await?;

    Ok(())
}
```

> **NOTE**: `-c copy` performs a stream copy (no re-encoding). This is nearly instant and preserves original quality. Only use re-encoding when format conversion is required.

---

## Pattern 5: Format Conversion

```rust
/// Convert a video to MP4 (H.264 + AAC) for maximum compatibility.
async fn convert_to_mp4(
    ffmpeg: &PathBuf,
    input_path: &PathBuf,
    output_path: &PathBuf,
) -> Result<(), MediaError> {
    let input = input_path.to_string_lossy();
    let output = output_path.to_string_lossy();

    run_ffmpeg(ffmpeg, &[
        "-i", &input,
        "-c:v", "libx264",       // H.264 video codec
        "-preset", "medium",      // Encoding speed vs quality trade-off
        "-crf", "23",             // Quality (18=near-lossless, 23=default, 28=low)
        "-c:a", "aac",            // AAC audio codec
        "-b:a", "192k",           // Audio bitrate
        "-movflags", "+faststart",
        "-y",
        &output,
    ]).await?;

    Ok(())
}

/// Extract audio only (MP3).
async fn extract_audio_mp3(
    ffmpeg: &PathBuf,
    input_path: &PathBuf,
    output_path: &PathBuf,
) -> Result<(), MediaError> {
    let input = input_path.to_string_lossy();
    let output = output_path.to_string_lossy();

    run_ffmpeg(ffmpeg, &[
        "-i", &input,
        "-vn",                    // No video
        "-c:a", "libmp3lame",     // MP3 codec
        "-q:a", "2",              // VBR quality (0=best, 9=worst). 2 ≈ 190kbps
        "-y",
        &output,
    ]).await?;

    Ok(())
}

/// Extract audio only (M4A/AAC — better quality than MP3 at same bitrate).
async fn extract_audio_m4a(
    ffmpeg: &PathBuf,
    input_path: &PathBuf,
    output_path: &PathBuf,
) -> Result<(), MediaError> {
    let input = input_path.to_string_lossy();
    let output = output_path.to_string_lossy();

    run_ffmpeg(ffmpeg, &[
        "-i", &input,
        "-vn",
        "-c:a", "aac",
        "-b:a", "256k",
        "-y",
        &output,
    ]).await?;

    Ok(())
}
```

---

## Pattern 6: Post-Download Processing Pipeline

```rust
use std::path::Path;

/// Full post-processing pipeline run after a successful download.
pub async fn post_process_download(
    ffmpeg: &PathBuf,
    ffprobe: &PathBuf,
    media_file: &PathBuf,
    thumbnail_original: Option<&PathBuf>,
) -> Result<PostProcessResult, MediaError> {
    // Step 1: Probe the downloaded media for dimensions/duration
    let probe = probe_media(ffprobe, media_file).await?;
    let info = extract_media_info(&probe);

    // Step 2: Generate reduced thumbnail for Wall
    let thumb_sm_path = media_file.with_extension("thumb_sm.jpg");

    if let Some(thumb) = thumbnail_original {
        // Resize existing thumbnail from yt-dlp
        generate_wall_thumbnail(ffmpeg, thumb, &thumb_sm_path).await?;
    } else {
        // No thumbnail downloaded — extract frame from video
        generate_thumbnail_from_video(
            ffmpeg, media_file, &thumb_sm_path, "00:00:03"
        ).await?;
    }

    // Step 3: Compute SHA-256 checksum
    let checksum = compute_sha256(media_file).await?;

    Ok(PostProcessResult {
        media_info: info,
        thumbnail_sm_path: thumb_sm_path,
        checksum,
    })
}

pub struct PostProcessResult {
    pub media_info: MediaInfo,
    pub thumbnail_sm_path: PathBuf,
    pub checksum: String,
}

/// Compute SHA-256 checksum of a file (for deduplication/integrity).
async fn compute_sha256(path: &PathBuf) -> Result<String, MediaError> {
    use sha2::{Sha256, Digest};
    use tokio::io::AsyncReadExt;

    let mut file = tokio::fs::File::open(path).await
        .map_err(|e| MediaError::IoError(e.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192];

    loop {
        let n = file.read(&mut buffer).await
            .map_err(|e| MediaError::IoError(e.to_string()))?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}
```

---

## Error Handling

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MediaError {
    #[error("Failed to spawn process: {0}")]
    ProcessSpawn(String),

    #[error("FFmpeg error: {0}")]
    FfmpegError(String),

    #[error("Failed to parse output: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("File not found: {0}")]
    FileNotFound(String),
}

impl serde::Serialize for MediaError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::ser::Serializer {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
```

---

## Best Practices

### Do's
- **Always use `-y`** — prevents ffmpeg from blocking on overwrite prompts
- **Use `-v quiet`** with ffprobe — avoids noisy banner output
- **Use `-c copy` when possible** — stream copy is 100x faster than re-encoding
- **Use `CREATE_NO_WINDOW`** on Windows — same as yt-dlp
- **Probe before converting** — check if conversion is actually needed
- **Use `+faststart`** for MP4 — moves metadata to file start for better seeking

### Don'ts
- **Don't re-encode unnecessarily** — check if the input format is already acceptable
- **Don't use absolute quality values blindly** — CRF 18 produces huge files
- **Don't forget to clean up temp files** — delete intermediate merge inputs after success
- **Don't block the main thread** — all ffmpeg operations must be async

## Dependencies

```toml
# Cargo.toml additions for this skill
[dependencies]
sha2 = "0.10"       # For SHA-256 checksum computation
```
