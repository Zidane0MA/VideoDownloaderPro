//! Internal deserialization structs for TikTok's item list APIs.
//! These map the raw JSON into intermediate types that get converted to `YtDlpVideo`.

#![allow(dead_code)]

use serde::Deserialize;
use serde_json::Value;

/// Top-level response from TikTok item list endpoints
/// (e.g. `/api/favorite/item_list/`, `/api/bookmark/item_list/`).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TikTokItemListResponse {
    pub item_list: Option<Vec<TikTokItem>>,
    /// Cursor can arrive as a JSON string or number depending on the endpoint.
    pub cursor: Option<Value>,
    pub has_more: Option<bool>,
    /// Status code embedded in response (0 = success)
    pub status_code: Option<i32>,
}

impl TikTokItemListResponse {
    /// Returns the pagination cursor as a `String`, regardless of whether
    /// the API sent it as a JSON string or number.
    pub(crate) fn cursor_string(&self) -> Option<String> {
        self.cursor.as_ref().map(|v| match v {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        })
    }
}

/// A single liked video item
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TikTokItem {
    pub id: String,
    /// Video description / caption
    pub desc: Option<String>,
    pub author: Option<TikTokAuthor>,
    pub video: Option<TikTokVideoMeta>,
    pub stats: Option<TikTokStats>,
    /// Unix timestamp (seconds)
    pub create_time: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TikTokAuthor {
    pub unique_id: Option<String>,
    pub nickname: Option<String>,
    pub id: Option<String>,
    pub sec_uid: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TikTokVideoMeta {
    pub duration: Option<u64>,
    /// Direct cover image URL
    pub cover: Option<String>,
    /// Dynamic (animated) cover URL
    pub dynamic_cover: Option<String>,
    /// Wider cover image
    pub origin_cover: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TikTokStats {
    pub play_count: Option<u64>,
    pub digg_count: Option<u64>,
    pub comment_count: Option<u64>,
    pub share_count: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_fav_response() {
        let json = r#"{
            "itemList": [
                {
                    "id": "7300000000000000001",
                    "desc": "Test video caption",
                    "author": {
                        "uniqueId": "testuser",
                        "nickname": "Test User",
                        "id": "123456",
                        "secUid": "MS4wLjAB..."
                    },
                    "video": {
                        "duration": 15,
                        "cover": "https://p16-sign.tiktokcdn.com/cover.jpg",
                        "width": 576,
                        "height": 1024
                    },
                    "stats": {
                        "playCount": 1000,
                        "diggCount": 200,
                        "commentCount": 50,
                        "shareCount": 10
                    },
                    "createTime": 1700000000
                }
            ],
            "cursor": "30",
            "hasMore": true,
            "statusCode": 0
        }"#;

        let resp: TikTokItemListResponse = serde_json::from_str(json).unwrap();
        assert!(resp.has_more.unwrap());
        assert_eq!(resp.cursor_string().as_deref(), Some("30"));

        let items = resp.item_list.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "7300000000000000001");
        assert_eq!(items[0].desc.as_deref(), Some("Test video caption"));

        let author = items[0].author.as_ref().unwrap();
        assert_eq!(author.unique_id.as_deref(), Some("testuser"));

        let video = items[0].video.as_ref().unwrap();
        assert_eq!(video.duration, Some(15));
        assert_eq!(video.width, Some(576));

        let stats = items[0].stats.as_ref().unwrap();
        assert_eq!(stats.play_count, Some(1000));
    }

    #[test]
    fn test_deserialize_empty_response() {
        let json = r#"{
            "statusCode": 0,
            "hasMore": false,
            "cursor": "0"
        }"#;

        let resp: TikTokItemListResponse = serde_json::from_str(json).unwrap();
        assert!(resp.item_list.is_none());
        assert!(!resp.has_more.unwrap());
    }

    #[test]
    fn test_deserialize_numeric_cursor() {
        let json = r#"{
            "itemList": [],
            "cursor": 1700000000,
            "hasMore": true,
            "statusCode": 0
        }"#;

        let resp: TikTokItemListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.cursor_string().as_deref(), Some("1700000000"));
    }

    #[test]
    fn test_deserialize_null_cursor() {
        let json = r#"{
            "statusCode": 0,
            "cursor": null,
            "hasMore": false
        }"#;

        let resp: TikTokItemListResponse = serde_json::from_str(json).unwrap();
        // null deserializes to None for Option<Value>
        assert!(resp.cursor_string().is_none());
    }
}
