use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtDlpThumbnail {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtDlpFormat {
    pub format_id: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub ext: Option<String>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub tbr: Option<f64>, // Total bitrate (kbps)
    #[serde(default)]
    pub vbr: Option<f64>, // Video bitrate (kbps)
    #[serde(default)]
    pub abr: Option<f64>, // Audio bitrate (kbps)
    #[serde(default)]
    pub asr: Option<u32>, // Audio sample rate (Hz), e.g. 44100, 48000
    #[serde(default)]
    pub fps: Option<f64>, // Frames per second
    #[serde(default)]
    pub vcodec: Option<String>, // e.g. "h264", "vp9", "av01"
    #[serde(default)]
    pub acodec: Option<String>, // e.g. "opus", "aac", "mp4a"
    #[serde(default)]
    pub audio_channels: Option<u32>, // e.g. 2 (stereo), 6 (5.1)
    #[serde(default)]
    pub container: Option<String>, // e.g. "mp4_dash", "webm_dash"
    #[serde(default)]
    pub protocol: Option<String>, // e.g. "https", "m3u8_native"
    #[serde(default)]
    pub dynamic_range: Option<String>, // e.g. "SDR", "HDR10", "HDR"
    #[serde(default)]
    pub resolution: Option<String>, // e.g. "1920x1080"
    #[serde(default)]
    pub format_note: Option<String>, // e.g. "1080p", "premium"
    #[serde(default)]
    pub language: Option<String>, // ISO 639 language code
    #[serde(default)]
    pub filesize: Option<u64>,
    #[serde(default)]
    pub filesize_approx: Option<u64>,
}

/// Single subtitle track entry from yt-dlp's `subtitles` / `automatic_captions` map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtDlpSubtitle {
    pub ext: String, // "vtt", "json3", "srv3"
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub name: Option<String>, // Human-readable name e.g. "English"
}

/// Subtitle map: language_code → Vec<YtDlpSubtitle>
pub type SubtitleMap = HashMap<String, Vec<YtDlpSubtitle>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtDlpVideo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,

    // Creator info
    pub uploader: Option<String>,
    pub uploader_id: Option<String>,
    pub uploader_url: Option<String>,
    pub channel: Option<String>,
    pub channel_id: Option<String>,
    pub channel_url: Option<String>,

    // Dates
    pub upload_date: Option<String>, // YYYYMMDD
    pub duration: Option<f64>,

    pub view_count: Option<u64>,
    pub like_count: Option<u64>,

    pub webpage_url: Option<String>,
    pub original_url: Option<String>,
    /// Generic URL field — yt-dlp populates this in `--flat-playlist` entries
    /// where `webpage_url` is absent. Used as fallback for post `original_url`.
    #[serde(default)]
    pub url: Option<String>,

    pub thumbnails: Option<Vec<YtDlpThumbnail>>,
    pub formats: Option<Vec<YtDlpFormat>>,

    // Subtitle tracks
    #[serde(default)]
    pub subtitles: Option<SubtitleMap>,
    #[serde(default)]
    pub automatic_captions: Option<SubtitleMap>,
    #[serde(default)]
    pub requested_subtitles: Option<SubtitleMap>,

    // Playlist info (if it's an item in a playlist)
    pub playlist_index: Option<u32>,
    pub playlist_title: Option<String>,
    pub playlist_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "_type")]
pub enum YtDlpOutput {
    #[serde(rename = "video")]
    Video(YtDlpVideo),
    #[serde(rename = "playlist")]
    Playlist(YtDlpPlaylist),
    // Fallback for when _type is missing (common in direct video downloads)
    #[serde(untagged)]
    VideoFallback(YtDlpVideo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtDlpPlaylist {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub entries: Option<Vec<YtDlpOutput>>,

    pub uploader: Option<String>,
    pub uploader_id: Option<String>,
    pub webpage_url: Option<String>,
}

impl YtDlpVideo {
    pub fn best_thumbnail(&self) -> Option<String> {
        self.thumbnails
            .as_ref()?
            .iter()
            // Prefer thumbnails with ID (often '0' or 'maxresdefault') or largest dimensions
            .max_by_key(|t| t.width.unwrap_or(0) * t.height.unwrap_or(0))
            .map(|t| t.url.clone())
            // Fallback to last one if no dimensions (yt-dlp conventions)
            .or_else(|| self.thumbnails.as_ref()?.last().map(|t| t.url.clone()))
    }
}
