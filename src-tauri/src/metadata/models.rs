use serde::{Deserialize, Serialize};

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
    pub url: Option<String>,
    pub ext: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub tbr: Option<f64>, // Total bitrate
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub filesize: Option<u64>,
    pub filesize_approx: Option<u64>,
}

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

    pub thumbnails: Option<Vec<YtDlpThumbnail>>,
    pub formats: Option<Vec<YtDlpFormat>>,

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
