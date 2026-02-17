use tauri::AppHandle;

use crate::sidecar;
use crate::sidecar::types::{SidecarBinary, SidecarStatus};

/// IPC: Returns the health status of all sidecar binaries.
#[tauri::command]
pub async fn get_sidecar_status(handle: AppHandle) -> SidecarStatus {
    sidecar::check_all(&handle).await
}

/// IPC: Returns the version string of yt-dlp.
#[tauri::command]
pub async fn get_ytdlp_version(handle: AppHandle) -> Result<String, String> {
    sidecar::get_version(&handle, SidecarBinary::YtDlp)
        .await
        .map_err(|e| e.to_string())
}

/// IPC: Triggers yt-dlp self-update and returns the new version.
#[tauri::command]
pub async fn update_ytdlp(handle: AppHandle) -> Result<String, String> {
    sidecar::update_yt_dlp(&handle)
        .await
        .map_err(|e| e.to_string())
}
