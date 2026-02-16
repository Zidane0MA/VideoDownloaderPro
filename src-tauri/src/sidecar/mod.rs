pub mod types;

use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use thiserror::Error;

use types::{SidecarBinary, SidecarInfo, SidecarStatus};

// ── Error ────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum SidecarError {
    #[error("Sidecar binary '{0}' not found — run scripts/download-sidecars.ps1")]
    BinaryNotFound(String),

    #[error("Failed to execute {binary}: {reason}")]
    ExecutionFailed { binary: String, reason: String },

    #[error("Failed to parse version output from {binary}: {raw}")]
    ParseError { binary: String, raw: String },

    #[error("yt-dlp self-update failed: {0}")]
    UpdateFailed(String),
}

impl From<SidecarError> for String {
    fn from(e: SidecarError) -> String {
        e.to_string()
    }
}

// ── Public API ───────────────────────────────────────────────────────

/// Retrieve the version string of a sidecar binary.
///
/// - `yt-dlp --version`  → `"2025.01.15"`
/// - `ffmpeg -version`   → first line, e.g. `"ffmpeg version N-118193-g..."`.
pub async fn get_version(
    handle: &AppHandle,
    binary: SidecarBinary,
) -> Result<String, SidecarError> {
    let output = handle
        .shell()
        .sidecar(binary.program_name())
        .map_err(|_| SidecarError::BinaryNotFound(binary.display_name().to_string()))?
        .args(binary.version_args())
        .output()
        .await
        .map_err(|e| SidecarError::ExecutionFailed {
            binary: binary.display_name().to_string(),
            reason: e.to_string(),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SidecarError::ExecutionFailed {
            binary: binary.display_name().to_string(),
            reason: format!("exit code {:?}: {}", output.status.code(), stderr.trim()),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_version(binary, &stdout)
}

/// Trigger yt-dlp's built-in self-update (`yt-dlp -U`).
/// Returns the new version string on success.
pub async fn update_yt_dlp(handle: &AppHandle) -> Result<String, SidecarError> {
    tracing::info!("Starting yt-dlp self-update…");

    let output = handle
        .shell()
        .sidecar(SidecarBinary::YtDlp.program_name())
        .map_err(|_| SidecarError::BinaryNotFound("yt-dlp".to_string()))?
        .args(["-U"])
        .output()
        .await
        .map_err(|e| SidecarError::ExecutionFailed {
            binary: "yt-dlp".to_string(),
            reason: e.to_string(),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SidecarError::UpdateFailed(stderr.trim().to_string()));
    }

    tracing::info!("yt-dlp self-update completed, fetching new version…");

    // Fetch the new version after update
    get_version(handle, SidecarBinary::YtDlp).await
}

/// Run a full health check on both sidecar binaries.
pub async fn check_all(handle: &AppHandle) -> SidecarStatus {
    let yt_dlp = check_one(handle, SidecarBinary::YtDlp).await;
    let ffmpeg = check_one(handle, SidecarBinary::Ffmpeg).await;

    tracing::info!(
        yt_dlp_available = yt_dlp.available,
        ffmpeg_available = ffmpeg.available,
        "Sidecar health check complete"
    );

    SidecarStatus { yt_dlp, ffmpeg }
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Check a single sidecar binary; never errors — wraps failures into `SidecarInfo`.
async fn check_one(handle: &AppHandle, binary: SidecarBinary) -> SidecarInfo {
    match get_version(handle, binary).await {
        Ok(version) => SidecarInfo {
            binary,
            available: true,
            version: Some(version),
            error: None,
        },
        Err(e) => SidecarInfo {
            binary,
            available: false,
            version: None,
            error: Some(e.to_string()),
        },
    }
}

/// Extract a clean version string from the raw CLI output.
fn parse_version(binary: SidecarBinary, raw: &str) -> Result<String, SidecarError> {
    let first_line = raw.lines().next().unwrap_or("").trim();

    if first_line.is_empty() {
        return Err(SidecarError::ParseError {
            binary: binary.display_name().to_string(),
            raw: raw.to_string(),
        });
    }

    match binary {
        // yt-dlp --version prints just the date string: "2025.01.15"
        SidecarBinary::YtDlp => Ok(first_line.to_string()),
        // ffmpeg -version prints: "ffmpeg version N-118193-g... Copyright ..."
        // We extract everything after "ffmpeg version " up to the next space.
        SidecarBinary::Ffmpeg => {
            let version = first_line
                .strip_prefix("ffmpeg version ")
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or(first_line);
            Ok(version.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ytdlp_version() {
        let raw = "2025.01.15\n";
        let result = parse_version(SidecarBinary::YtDlp, raw).unwrap();
        assert_eq!(result, "2025.01.15");
    }

    #[test]
    fn test_parse_ffmpeg_version() {
        let raw =
            "ffmpeg version N-118193-gc660a3a5f6-20250213 Copyright (c) 2000-2025 the FFmpeg developers\nbuilt with gcc 14.2.0\n";
        let result = parse_version(SidecarBinary::Ffmpeg, raw).unwrap();
        assert_eq!(result, "N-118193-gc660a3a5f6-20250213");
    }

    #[test]
    fn test_parse_empty_output_errors() {
        let raw = "";
        let result = parse_version(SidecarBinary::YtDlp, raw);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ffmpeg_unknown_format() {
        // When the standard prefix is missing, we return the full first line.
        let raw = "some-unknown-format v1.2.3\n";
        let result = parse_version(SidecarBinary::Ffmpeg, raw).unwrap();
        assert_eq!(result, "some-unknown-format v1.2.3");
    }
}
