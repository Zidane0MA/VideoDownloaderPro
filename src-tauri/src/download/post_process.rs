use std::path::PathBuf;
use tokio::process::Command;

/// Result of post-processing a downloaded media file.
pub struct ThumbnailResult {
    /// Path to the 300px thumbnail for the Wall gallery.
    pub thumbnail_path: Option<String>,
}

/// Find the yt-dlp-generated thumbnail for a media file.
/// yt-dlp names it `<title>.jpg` when using `--write-thumbnail --convert-thumbnails jpg`
/// alongside the media file `<title>.<ext>`.
fn find_ytdlp_thumbnail(media_file: &PathBuf) -> Option<PathBuf> {
    let stem = media_file.file_stem()?.to_str()?;
    let parent = media_file.parent()?;

    // yt-dlp saves thumbnail as <stem>.jpg
    let thumb = parent.join(format!("{}.jpg", stem));
    if thumb.exists() {
        return Some(thumb);
    }

    // Sometimes yt-dlp adds a suffix like <stem>.webp or <stem>.png before converting
    // Check common patterns
    for ext in ["png", "webp", "jpeg"] {
        let alt = parent.join(format!("{}.{}", stem, ext));
        if alt.exists() {
            return Some(alt);
        }
    }

    None
}

/// Process thumbnails after a download completes:
/// 1. Locate the yt-dlp original thumbnail (platform thumbnail)
/// 2. Resize it to 300px for the Wall gallery
/// 3. Fall back to ffmpeg frame extraction if no thumbnail was downloaded
pub async fn process_thumbnails(
    ffmpeg_path: &PathBuf,
    media_file: &PathBuf,
    media_type: &str,
) -> ThumbnailResult {
    let original_thumb = find_ytdlp_thumbnail(media_file);

    match original_thumb {
        Some(thumb_path) => {
            // We have the original platform thumbnail — resize it for the Wall
            let thumb_sm = generate_resized_thumbnail(ffmpeg_path, &thumb_path).await;

            ThumbnailResult {
                thumbnail_path: thumb_sm
                    .map(|p| p.to_string_lossy().to_string())
                    .ok()
                    .or_else(|| Some(thumb_path.to_string_lossy().to_string())),
            }
        }
        None => {
            // No platform thumbnail found — extract a frame from video as fallback
            if media_type == "VIDEO" {
                tracing::info!(
                    "No platform thumbnail found for {}, extracting frame from video",
                    media_file.display()
                );
                let extracted = extract_frame_thumbnail(ffmpeg_path, media_file).await;
                match extracted {
                    Ok(path) => ThumbnailResult {
                        thumbnail_path: Some(path.to_string_lossy().to_string()),
                    },
                    Err(e) => {
                        tracing::warn!("Frame extraction failed: {}", e);
                        ThumbnailResult {
                            thumbnail_path: None,
                        }
                    }
                }
            } else {
                ThumbnailResult {
                    thumbnail_path: None,
                }
            }
        }
    }
}

/// Resize an existing thumbnail to 300px wide for the Wall gallery.
async fn generate_resized_thumbnail(
    ffmpeg_path: &PathBuf,
    original_thumb: &PathBuf,
) -> Result<PathBuf, String> {
    let stem = original_thumb
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("thumb");
    let output_path = original_thumb
        .parent()
        .unwrap_or(original_thumb)
        .join(format!("{}.thumb_sm.jpg", stem));

    let input = original_thumb.to_string_lossy();
    let output = output_path.to_string_lossy();

    tracing::info!("Resizing thumbnail: {} -> {}", input, output);

    let mut cmd = Command::new(ffmpeg_path);
    cmd.args([
        "-i",
        &input,
        "-vf",
        "scale=300:-1", // 300px wide, maintain aspect ratio
        "-q:v",
        "3",  // Good JPEG quality
        "-y", // Overwrite if exists
        &output,
    ]);

    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd.stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());

    let result = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to spawn ffmpeg for resize: {}", e))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!("ffmpeg resize failed: {}", stderr.trim()));
    }

    if output_path.exists() {
        tracing::info!("Resized thumbnail: {}", output_path.display());
        Ok(output_path)
    } else {
        Err("ffmpeg succeeded but resized thumbnail was not created".to_string())
    }
}

/// Fallback: extract a frame from the video at ~3 seconds and save as a 300px thumbnail.
async fn extract_frame_thumbnail(
    ffmpeg_path: &PathBuf,
    video_file: &PathBuf,
) -> Result<PathBuf, String> {
    let stem = video_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("thumb");
    let output_path = video_file
        .parent()
        .unwrap_or(video_file)
        .join(format!("{}.thumb_sm.jpg", stem));

    // Try at 3 seconds, fallback to 0 seconds for very short videos
    for timestamp in ["00:00:03", "00:00:00"] {
        let input = video_file.to_string_lossy();
        let output = output_path.to_string_lossy();

        let mut cmd = Command::new(ffmpeg_path);
        cmd.args([
            "-ss",
            timestamp,
            "-i",
            &input,
            "-vframes",
            "1",
            "-vf",
            "scale=300:-1",
            "-q:v",
            "3",
            "-y",
            &output,
        ]);

        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        cmd.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped());

        let result = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to spawn ffmpeg: {}", e))?;

        if result.status.success() && output_path.exists() {
            tracing::info!(
                "Frame thumbnail extracted at {}: {}",
                timestamp,
                output_path.display()
            );
            return Ok(output_path);
        }
    }

    Err("Failed to extract frame at any timestamp".to_string())
}
