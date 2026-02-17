pub mod types;

use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use thiserror::Error;
use tokio::process::Command;

use types::{SidecarBinary, SidecarInfo, SidecarStatus};

// ── Error ────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum SidecarError {
    #[error("Sidecar binary '{0}' not found in app data")]
    BinaryNotFound(String),

    #[error("Failed to execute {binary}: {reason}")]
    ExecutionFailed { binary: String, reason: String },

    #[error("Failed to parse version output from {binary}: {raw}")]
    ParseError { binary: String, raw: String },

    #[error("yt-dlp self-update failed: {0}")]
    UpdateFailed(String),

    #[error("Failed to setup sidecars: {0}")]
    SetupFailed(String),
}

impl From<SidecarError> for String {
    fn from(e: SidecarError) -> String {
        e.to_string()
    }
}

// ── Public API ───────────────────────────────────────────────────────

/// Ensure sidecar binaries exist in `app_data/binaries/`.
/// If not, copy them from the bundled resources.
pub async fn setup_sidecars(handle: &AppHandle) -> Result<(), SidecarError> {
    let app_data_dir = handle
        .path()
        .app_data_dir()
        .map_err(|e| SidecarError::SetupFailed(format!("Failed to resolve app_data_dir: {}", e)))?;

    let binary_dir = app_data_dir.join("binaries");

    if !binary_dir.exists() {
        tokio::fs::create_dir_all(&binary_dir).await.map_err(|e| {
            SidecarError::SetupFailed(format!("Failed to create binary dir: {}", e))
        })?;
    }

    let resource_dir = handle
        .path()
        .resource_dir()
        .map_err(|e| SidecarError::SetupFailed(format!("Failed to resolve resource_dir: {}", e)))?;

    // We expect bundled binaries in specific locations relative to resource_dir
    // Specifically: resources/binaries/<name>-<target>
    // Since we don't know the exact target triple easily at runtime without build hacks,
    // we search for files starting with the program name in the bundled binaries folder.
    let possible_paths = [
        resource_dir.join("binaries"),
        PathBuf::from("src-tauri/binaries"),
        PathBuf::from("binaries"),
    ];

    let bundled_bin_dir = possible_paths.iter().find(|p| p.exists()).ok_or_else(|| {
        SidecarError::SetupFailed("Could not find bundled binaries directory".into())
    })?;

    for binary in [SidecarBinary::YtDlp, SidecarBinary::Ffmpeg] {
        let name = binary.display_name(); // "yt-dlp" or "ffmpeg"
        let target_filename = if cfg!(windows) {
            format!("{}.exe", name)
        } else {
            name.to_string()
        };
        let target_path = binary_dir.join(&target_filename);

        if !target_path.exists() {
            tracing::info!("Installing sidecar: {} -> {:?}", name, target_path);

            // Find source file: name-*.exe or name-*
            let mut source_path: Option<PathBuf> = None;
            let mut entries = tokio::fs::read_dir(&bundled_bin_dir).await.map_err(|e| {
                SidecarError::SetupFailed(format!("Failed to read bundled binaries dir: {}", e))
            })?;

            while let Some(entry) = entries.next_entry().await.map_err(|e| {
                SidecarError::SetupFailed(format!("Failed to iterate bundled entries: {}", e))
            })? {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.starts_with(name) && file_name_str.contains('-') {
                    // Simple heuristic: starts with name and has a dash (implies target triple suffix)
                    // e.g. yt-dlp-x86_64-pc-windows-msvc.exe
                    source_path = Some(entry.path());
                    break;
                }
            }

            if let Some(src) = source_path {
                tokio::fs::copy(&src, &target_path).await.map_err(|e| {
                    SidecarError::SetupFailed(format!(
                        "Failed to copy {} to {:?}: {}",
                        name, target_path, e
                    ))
                })?;
            } else {
                tracing::warn!(
                    "Bundled binary for {} not found in {:?}, skipping copy.",
                    name,
                    bundled_bin_dir
                );
            }
        }
    }

    Ok(())
}

/// Retrieve the version string of a sidecar binary.
pub async fn get_version(
    handle: &AppHandle,
    binary: SidecarBinary,
) -> Result<String, SidecarError> {
    let binary_path = get_binary_path(handle, binary)?;

    // Use tokio::process::Command directly to execute the binary from app_data
    // This bypasses tauri_plugin_shell's scope restriction on absolute paths,
    // relying on the fact that we completely control this backend execution.
    let output = Command::new(binary_path)
        .args(binary.version_args())
        // crucial: detach from console on windows to avoid popping up windows?
        // default tokio command shouldn't pop up window unless specifically told to?
        // usually needs creation_flags(0x08000000) for NO_WINDOW if gui app
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
    let binary_path = get_binary_path(handle, SidecarBinary::YtDlp)?;

    let output = Command::new(&binary_path)
        .arg("-U")
        .output()
        .await
        .map_err(|e| SidecarError::ExecutionFailed {
            binary: "yt-dlp".to_string(),
            reason: e.to_string(),
        })?;

    // Note: yt-dlp exit code might vary on update?
    // Usually 0 is success.
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SidecarError::UpdateFailed(stderr.trim().to_string()));
    }

    tracing::info!("yt-dlp self-update completed, fetching new version…");

    // Fetch the new version after update
    get_version(handle, SidecarBinary::YtDlp).await
}

/// Get the absolute path to the binary in app_data.
pub fn get_binary_path(handle: &AppHandle, binary: SidecarBinary) -> Result<PathBuf, SidecarError> {
    let app_data_dir = handle
        .path()
        .app_data_dir()
        .map_err(|_| SidecarError::BinaryNotFound("Could not resolve app_data".into()))?;

    let name = binary.display_name();
    let filename = if cfg!(windows) {
        format!("{}.exe", name)
    } else {
        name.to_string()
    };

    let path = app_data_dir.join("binaries").join(filename);
    if !path.exists() {
        return Err(SidecarError::BinaryNotFound(path.display().to_string()));
    }
    Ok(path)
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
        // ffmpeg -version prints: "ffmpeg version N-118193-g..."
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
