use tauri::AppHandle;

use crate::sidecar;
use crate::sidecar::types::{SidecarBinary, SidecarStatus};

/// IPC: Returns the health status of all sidecar binaries.
#[tauri::command]
pub async fn get_sidecar_status(handle: AppHandle) -> SidecarStatus {
    sidecar::check_all(&handle).await
}

/// IPC: Returns the version string of a specific sidecar binary.
/// `binary` must be `"yt_dlp"` or `"ffmpeg"`.
#[tauri::command]
pub async fn get_sidecar_version(handle: AppHandle, binary: String) -> Result<String, String> {
    let bin = match binary.as_str() {
        "yt_dlp" => SidecarBinary::YtDlp,
        "ffmpeg" => SidecarBinary::Ffmpeg,
        other => {
            return Err(format!(
                "Unknown sidecar binary: '{other}'. Use 'yt_dlp' or 'ffmpeg'."
            ))
        }
    };

    sidecar::get_version(&handle, bin)
        .await
        .map_err(|e| e.to_string())
}

/// IPC: Triggers yt-dlp self-update and returns the new version.
#[tauri::command]
pub async fn update_sidecar(handle: AppHandle) -> Result<String, String> {
    sidecar::update_yt_dlp(&handle)
        .await
        .map_err(|e| e.to_string())
}
