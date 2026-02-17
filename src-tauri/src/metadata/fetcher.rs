use super::models::YtDlpOutput;
use super::MetadataError;
use crate::sidecar::types::SidecarBinary;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

/// Runs `yt-dlp --dump-single-json --flat-playlist <url>` and returns the parsed metadata.
pub async fn fetch_metadata(app: &AppHandle, url: &str) -> Result<YtDlpOutput, MetadataError> {
    let sidecar = SidecarBinary::YtDlp;

    // Construct arguments
    // --dump-single-json: Ensure we get a single JSON object (Video or Playlist)
    // --flat-playlist: Don't recurse into playlist items (fast)
    // --no-warnings: Keep stderr clean
    let args = vec![
        "--dump-single-json",
        "--flat-playlist",
        "--no-warnings",
        url,
    ];

    tracing::info!("Fetching metadata for URL: {}", url);

    let output = app
        .shell()
        .sidecar(sidecar.program_name())
        .map_err(|e| MetadataError::Sidecar(e.to_string()))?
        .args(&args)
        .output()
        .await
        .map_err(|e| MetadataError::Execution(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MetadataError::Execution(format!(
            "yt-dlp failed (code {:?}): {}",
            output.status.code(),
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse JSON
    serde_json::from_str::<YtDlpOutput>(&stdout).map_err(|e| MetadataError::Parse(e.to_string()))
}
