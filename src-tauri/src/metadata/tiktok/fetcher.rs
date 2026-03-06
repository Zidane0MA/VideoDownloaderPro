//! Core TikTok extractor for liked videos.
//!
//! Uses reqwest to call TikTok's internal `/api/favorite/item_list/` endpoint
//! and converts the response into `YtDlpOutput::Playlist` for seamless pipeline reuse.

use reqwest::header::{HeaderMap, HeaderValue, COOKIE, REFERER, USER_AGENT};
use thiserror::Error;

use super::helpers::netscape_to_header;
use super::models::{TikTokFavResponse, TikTokItem};
use crate::metadata::models::{YtDlpOutput, YtDlpPlaylist, YtDlpThumbnail, YtDlpVideo};

// ── Error ──────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum TikTokError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Failed to resolve secUid for @{username}: {reason}")]
    SecUidResolution { username: String, reason: String },

    #[error("TikTok returned HTTP {status}: {hint}")]
    ApiError { status: u16, hint: String },

    #[error("Failed to parse API response: {0}")]
    Parse(String),

    #[error("No liked videos found (profile may be private or have no public likes)")]
    EmptyResult,
}

// ── Fetcher ────────────────────────────────────────────────────────────

const USER_AGENT_STR: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const PAGE_SIZE: u32 = 30;
const MAX_PAGES: u32 = 100; // Safety limit: 3000 items max

pub struct TikTokFetcher {
    client: reqwest::Client,
}

impl TikTokFetcher {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("Failed to build reqwest client");

        Self { client }
    }

    /// Fetch liked videos for a TikTok user.
    ///
    /// - `netscape_cookies`: Netscape-format cookie jar text (from `CookieManager::get_session`)
    /// - `username`: TikTok username (without @)
    /// - `limit`: Max number of items to fetch. `None` = fetch all.
    pub async fn fetch_liked_videos(
        &self,
        netscape_cookies: &str,
        username: &str,
        limit: Option<u32>,
    ) -> Result<YtDlpOutput, TikTokError> {
        let cookie_header = netscape_to_header(netscape_cookies);

        // Step 1: Resolve secUid from profile page
        let sec_uid = self.resolve_sec_uid(username, &cookie_header).await?;

        tracing::info!(
            "Resolved secUid for @{}: {}...",
            username,
            &sec_uid[..sec_uid.len().min(20)]
        );

        // Step 2: Paginate liked videos
        let items = self
            .paginate_liked_items(&sec_uid, username, &cookie_header, limit)
            .await?;

        if items.is_empty() {
            return Err(TikTokError::EmptyResult);
        }

        tracing::info!("Fetched {} liked videos for @{}", items.len(), username);

        // Step 3: Convert to YtDlpOutput::Playlist
        let entries: Vec<YtDlpOutput> = items
            .into_iter()
            .map(|item| YtDlpOutput::Video(tiktok_item_to_video(item, username)))
            .collect();

        let playlist = YtDlpPlaylist {
            id: format!("tiktok_liked_{}", username),
            title: format!("@{}'s Liked Videos", username),
            description: None,
            uploader: Some(username.to_string()),
            uploader_id: Some(username.to_string()),
            webpage_url: Some(format!("https://www.tiktok.com/@{}/liked", username)),
            entries: Some(entries),
        };

        Ok(YtDlpOutput::Playlist(playlist))
    }

    /// Fetch the profile page and extract `secUid` from embedded JSON.
    async fn resolve_sec_uid(
        &self,
        username: &str,
        cookie_header: &str,
    ) -> Result<String, TikTokError> {
        let profile_url = format!("https://www.tiktok.com/@{}", username);

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
        if let Ok(val) = HeaderValue::from_str(cookie_header) {
            headers.insert(COOKIE, val);
        }

        let response = self
            .client
            .get(&profile_url)
            .headers(headers)
            .send()
            .await?;

        let status = response.status().as_u16();
        if status == 403 {
            return Err(TikTokError::ApiError {
                status,
                hint: "cookies expired or invalid — please log in again".to_string(),
            });
        }
        if status == 429 {
            return Err(TikTokError::ApiError {
                status,
                hint: "rate limited by TikTok — wait and try again".to_string(),
            });
        }
        if !response.status().is_success() {
            return Err(TikTokError::ApiError {
                status,
                hint: format!("unexpected status fetching profile for @{}", username),
            });
        }

        let html = response.text().await?;

        // Look for "secUid":"<value>" in the embedded JSON
        extract_sec_uid(&html).ok_or_else(|| TikTokError::SecUidResolution {
            username: username.to_string(),
            reason: if html.contains("verify") || html.contains("captcha") {
                "TikTok is showing a CAPTCHA. Try logging in again in the browser.".to_string()
            } else {
                "secUid not found in profile HTML".to_string()
            },
        })
    }

    /// Paginate through the favorite item_list API.
    async fn paginate_liked_items(
        &self,
        sec_uid: &str,
        username: &str,
        cookie_header: &str,
        limit: Option<u32>,
    ) -> Result<Vec<TikTokItem>, TikTokError> {
        let mut all_items = Vec::new();
        let mut cursor = "0".to_string();
        let max_items = limit.unwrap_or(u32::MAX);
        let referer = format!("https://www.tiktok.com/@{}", username);

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
        if let Ok(val) = HeaderValue::from_str(cookie_header) {
            headers.insert(COOKIE, val);
        }
        if let Ok(val) = HeaderValue::from_str(&referer) {
            headers.insert(REFERER, val);
        }

        for page in 0..MAX_PAGES {
            let api_url = format!(
                "https://www.tiktok.com/api/favorite/item_list/\
                 ?aid=1988\
                 &count={PAGE_SIZE}\
                 &cursor={cursor}\
                 &secUid={sec_uid}\
                 &cookie_enabled=true"
            );

            tracing::debug!("Fetching page {} (cursor={})", page, cursor);

            let response = self
                .client
                .get(&api_url)
                .headers(headers.clone())
                .send()
                .await?;

            let status = response.status().as_u16();
            if status == 403 {
                return Err(TikTokError::ApiError {
                    status,
                    hint: "cookies expired or invalid".to_string(),
                });
            }
            if status == 429 {
                return Err(TikTokError::ApiError {
                    status,
                    hint: "rate limited — wait and try again".to_string(),
                });
            }
            if !response.status().is_success() {
                return Err(TikTokError::ApiError {
                    status,
                    hint: "unexpected API error".to_string(),
                });
            }

            let text = response.text().await?;
            let fav_response: TikTokFavResponse =
                serde_json::from_str(&text).map_err(|e| TikTokError::Parse(e.to_string()))?;

            if let Some(items) = fav_response.item_list {
                all_items.extend(items);
            }

            // Check if we've hit the limit
            if all_items.len() as u32 >= max_items {
                all_items.truncate(max_items as usize);
                break;
            }

            // Check if there are more pages
            let has_more = fav_response.has_more.unwrap_or(false);
            if !has_more {
                break;
            }

            cursor = fav_response.cursor.unwrap_or_else(|| "0".to_string());
        }

        Ok(all_items)
    }
}

// ── Conversion helpers ─────────────────────────────────────────────────

/// Extract `secUid` from TikTok profile HTML.
///
/// Looks for `"secUid":"<value>"` in the embedded `__UNIVERSAL_DATA_FOR_REHYDRATION__` JSON.
fn extract_sec_uid(html: &str) -> Option<String> {
    let marker = "\"secUid\":\"";
    let start = html.find(marker)?;
    let rest = &html[start + marker.len()..];
    let end = rest.find('"')?;
    let sec_uid = &rest[..end];

    if sec_uid.is_empty() {
        None
    } else {
        Some(sec_uid.to_string())
    }
}

/// Convert a `TikTokItem` from the API into a `YtDlpVideo` for pipeline reuse.
fn tiktok_item_to_video(item: TikTokItem, liked_by_username: &str) -> YtDlpVideo {
    let author_name = item
        .author
        .as_ref()
        .and_then(|a| a.nickname.clone())
        .or_else(|| item.author.as_ref().and_then(|a| a.unique_id.clone()));

    let author_id = item.author.as_ref().and_then(|a| a.unique_id.clone());

    let author_url = author_id
        .as_ref()
        .map(|id| format!("https://www.tiktok.com/@{}", id));

    // Build thumbnails from video cover URLs
    let mut thumbnails = Vec::new();
    if let Some(ref video) = item.video {
        if let Some(ref cover) = video.origin_cover {
            thumbnails.push(YtDlpThumbnail {
                url: cover.clone(),
                width: video.width,
                height: video.height,
                id: Some("origin_cover".to_string()),
            });
        }
        if let Some(ref cover) = video.cover {
            thumbnails.push(YtDlpThumbnail {
                url: cover.clone(),
                width: None,
                height: None,
                id: Some("cover".to_string()),
            });
        }
    }

    let duration = item
        .video
        .as_ref()
        .and_then(|v| v.duration)
        .map(|d| d as f64);

    // Convert Unix timestamp to yt-dlp's YYYYMMDD format
    let upload_date = item.create_time.map(|ts| {
        chrono::DateTime::from_timestamp(ts, 0)
            .map(|dt| dt.format("%Y%m%d").to_string())
            .unwrap_or_default()
    });

    let webpage_url = format!(
        "https://www.tiktok.com/@{}/video/{}",
        author_id.as_deref().unwrap_or(liked_by_username),
        item.id
    );

    YtDlpVideo {
        id: item.id,
        title: item.desc.clone().unwrap_or_else(|| "Untitled".to_string()),
        description: item.desc,

        uploader: author_name.clone(),
        uploader_id: author_id.clone(),
        uploader_url: author_url.clone(),
        channel: author_name,
        channel_id: author_id,
        channel_url: author_url,

        upload_date,
        duration,

        view_count: item.stats.as_ref().and_then(|s| s.play_count),
        like_count: item.stats.as_ref().and_then(|s| s.digg_count),

        webpage_url: Some(webpage_url.clone()),
        original_url: Some(webpage_url),
        url: None,

        thumbnails: if thumbnails.is_empty() {
            None
        } else {
            Some(thumbnails)
        },
        formats: None,

        subtitles: None,
        automatic_captions: None,
        requested_subtitles: None,

        playlist_index: None,
        playlist_title: None,
        playlist_id: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sec_uid_from_html() {
        let html = r#"<script>window.__UNIVERSAL_DATA_FOR_REHYDRATION__={"secUid":"MS4wLjABAAAAtest123","uniqueId":"user"}</script>"#;
        assert_eq!(
            extract_sec_uid(html),
            Some("MS4wLjABAAAAtest123".to_string())
        );
    }

    #[test]
    fn test_extract_sec_uid_not_found() {
        let html = "<html><body>No data here</body></html>";
        assert_eq!(extract_sec_uid(html), None);
    }

    #[test]
    fn test_tiktok_item_to_video_conversion() {
        use super::super::models::*;

        let item = TikTokItem {
            id: "123456".to_string(),
            desc: Some("Cool video 🎶".to_string()),
            author: Some(TikTokAuthor {
                unique_id: Some("cooluser".to_string()),
                nickname: Some("Cool User".to_string()),
                id: Some("999".to_string()),
                sec_uid: None,
            }),
            video: Some(TikTokVideoMeta {
                duration: Some(30),
                cover: Some("https://cdn.tiktok.com/cover.jpg".to_string()),
                dynamic_cover: None,
                origin_cover: Some("https://cdn.tiktok.com/origin.jpg".to_string()),
                width: Some(576),
                height: Some(1024),
            }),
            stats: Some(TikTokStats {
                play_count: Some(50000),
                digg_count: Some(5000),
                comment_count: Some(100),
                share_count: Some(50),
            }),
            create_time: Some(1700000000),
        };

        let video = tiktok_item_to_video(item, "myuser");

        assert_eq!(video.id, "123456");
        assert_eq!(video.title, "Cool video 🎶");
        assert_eq!(video.uploader.as_deref(), Some("Cool User"));
        assert_eq!(video.uploader_id.as_deref(), Some("cooluser"));
        assert_eq!(video.duration, Some(30.0));
        assert_eq!(video.view_count, Some(50000));
        assert_eq!(video.like_count, Some(5000));
        assert_eq!(
            video.webpage_url.as_deref(),
            Some("https://www.tiktok.com/@cooluser/video/123456")
        );
        assert_eq!(video.thumbnails.as_ref().unwrap().len(), 2);
        assert!(video.upload_date.is_some());
    }
}
