use serde::Serialize;

/// Identifies which sidecar binary to target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SidecarBinary {
    YtDlp,
    Ffmpeg,
    Qjs,
}

impl SidecarBinary {
    /// The sidecar program name as registered in `tauri.conf.json`.
    pub fn program_name(&self) -> &'static str {
        match self {
            Self::YtDlp => "binaries/yt-dlp",
            Self::Ffmpeg => "binaries/ffmpeg",
            Self::Qjs => "binaries/qjs",
        }
    }

    /// The CLI flag to print the version.
    pub fn version_args(&self) -> &'static [&'static str] {
        match self {
            Self::YtDlp => &["--version"],
            Self::Ffmpeg => &["-version"],
            Self::Qjs => &["--version"], // qjs supports --version? Yes usually.
        }
    }

    /// Display name for logging / error messages.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::YtDlp => "yt-dlp",
            Self::Ffmpeg => "ffmpeg",
            Self::Qjs => "qjs",
        }
    }
}

/// Health status of a single sidecar binary.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SidecarInfo {
    pub binary: SidecarBinary,
    pub available: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

/// Combined health check result for all sidecars.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SidecarStatus {
    pub yt_dlp: SidecarInfo,
    pub ffmpeg: SidecarInfo,
    pub qjs: SidecarInfo,
}
