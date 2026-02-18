use super::models::YtDlpOutput;
use super::MetadataError;
use crate::sidecar::{get_binary_path, types::SidecarBinary};
use tauri::AppHandle;
use tokio::process::Command;

/// Runs `yt-dlp --dump-single-json --flat-playlist <url>` and returns the parsed metadata.
pub async fn fetch_metadata(
    app: &AppHandle,
    url: &str,
    cookie_path: Option<&std::path::PathBuf>,
) -> Result<YtDlpOutput, MetadataError> {
    let sidecar = SidecarBinary::YtDlp;
    let binary_path =
        get_binary_path(app, sidecar).map_err(|e| MetadataError::Sidecar(e.to_string()))?;

    let sidecar_deno = SidecarBinary::Deno;
    let deno_path = get_binary_path(app, sidecar_deno)
        .map_err(|e| MetadataError::Sidecar(format!("Deno not found: {}", e)))?;
    let deno_arg = format!("deno:{}", deno_path.to_string_lossy());

    // Construct arguments
    // --dump-single-json: Ensure we get a single JSON object (Video or Playlist)
    // --flat-playlist: Don't recurse into playlist items (fast)
    // --no-warnings: Keep stderr clean
    // Windows: hide console window
    let mut cmd = Command::new(binary_path);
    cmd.arg("--dump-single-json")
        .arg("--flat-playlist")
        .arg("--no-warnings")
        .arg("-f")
        .arg("bestvideo+bestaudio/best")
        .arg("--js-runtimes")
        .arg(deno_arg);

    if let Some(path) = cookie_path {
        cmd.arg("--cookies").arg(path);
    }

    cmd.arg(url);

    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    tracing::info!("Fetching metadata for URL: {}", url);

    let output = cmd
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
