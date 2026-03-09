use std::path::PathBuf;
use tokio::process::Command;

/// Result of post-processing a downloaded media file.
pub struct ThumbnailResult {
    /// Path to the 300px thumbnail for the Wall gallery.
    pub thumbnail_path: Option<String>,
}

/// Process thumbnails after a download completes:
/// - Extract a frame from the video and scale to 300px for the Wall gallery
pub async fn process_thumbnails(
    ffmpeg_path: &std::path::Path,
    media_file: &std::path::Path,
    media_type: &str,
) -> ThumbnailResult {
    if media_type == "VIDEO" {
        tracing::info!(
            "Extracting frame from video for thumbnail: {}",
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

/// Extract a frame from the video at ~3 seconds and save as a 300px thumbnail.
async fn extract_frame_thumbnail(
    ffmpeg_path: &std::path::Path,
    video_file: &std::path::Path,
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
