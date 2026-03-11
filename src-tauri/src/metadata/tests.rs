#[cfg(test)]
mod tests {
    use crate::db;
    use crate::entity::{creator, post};
    use crate::metadata::models::{YtDlpOutput, YtDlpPlaylist, YtDlpVideo};
    use crate::metadata::store::save_metadata;
    use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};

    #[tokio::test]
    async fn test_save_video_metadata() {
        let db = db::init_test_db().await.expect("Failed to init test db");

        let video = YtDlpVideo {
            id: "video123".to_string(),
            title: "Test Video".to_string(),
            description: Some("Project description".to_string()),
            uploader: Some("Test Channel".to_string()),
            uploader_id: Some("channel123".to_string()),
            uploader_url: Some("https://youtube.com/@channel123".to_string()),
            webpage_url: Some("https://youtube.com/watch?v=video123".to_string()),
            upload_date: Some("20230101".to_string()),
            duration: Some(120.5),
            view_count: Some(100),
            like_count: Some(10),
            thumbnails: None,
            formats: None,
            subtitles: None,
            automatic_captions: None,
            requested_subtitles: None,
            original_url: None,
            url: None,
            channel: None,
            channel_id: None,
            channel_url: None,
            playlist_index: None,
            playlist_title: None,
            playlist_id: None,
        };

        let output = YtDlpOutput::Video(video);
        let post_id = save_metadata(&db, output)
            .await
            .expect("Failed to save metadata");

        assert_eq!(post_id, 1);

        // Verify DB content
        let saved_post = post::Entity::find_by_id(post_id)
            .one(&db)
            .await
            .expect("DB error")
            .unwrap();
        assert_eq!(saved_post.title, Some("Test Video".to_string()));
        assert_eq!(saved_post.status, "PENDING"); // Should be PENDING

        let saved_creator = creator::Entity::find()
            .filter(creator::Column::ExternalId.eq("channel123"))
            .one(&db)
            .await
            .expect("DB error")
            .unwrap();
        assert_eq!(saved_creator.name, "Test Channel");
    }

    #[tokio::test]
    async fn test_save_playlist_metadata() {
        let db = db::init_test_db().await.expect("Failed to init test db");

        // Simulate a flat-playlist entry: has `url` but NOT `webpage_url`
        let video1 = YtDlpVideo {
            id: "v1".to_string(),
            title: "Video 1".to_string(),
            webpage_url: None,
            url: Some("https://youtube.com/watch?v=v1".to_string()),
            uploader_id: Some("c1".to_string()),
            uploader: Some("Channel 1".to_string()),
            description: None,
            uploader_url: Some("https://youtube.com/@channel1".to_string()),
            upload_date: None,
            duration: None,
            view_count: None,
            like_count: None,
            thumbnails: None,
            formats: None,
            subtitles: None,
            automatic_captions: None,
            requested_subtitles: None,
            original_url: None,
            channel: None,
            channel_id: None,
            channel_url: None,
            playlist_index: None,
            playlist_title: None,
            playlist_id: None,
        };

        let playlist = YtDlpPlaylist {
            id: "playlist1".to_string(),
            title: "My Playlist".to_string(),
            description: Some("Desc".to_string()),
            uploader: Some("Playlist Creator".to_string()),
            uploader_id: Some("pc1".to_string()),
            webpage_url: Some("https://youtube.com/playlist?list=playlist1".to_string()),
            entries: Some(vec![YtDlpOutput::Video(video1)]),
        };

        let source_id = save_metadata(&db, YtDlpOutput::Playlist(playlist))
            .await
            .expect("Failed to save playlist");
        assert_eq!(source_id, 1);

        // Verify Source
        // Note: I need to import source entity to verify, but checking posts exists is good enough
        let saved_post = post::Entity::find()
            .filter(post::Column::ExternalId.eq("v1"))
            .one(&db)
            .await
            .expect("DB error")
            .unwrap();
        assert_eq!(saved_post.source_id, Some(source_id));
        // URL should be populated from `url` fallback (webpage_url was None)
        assert_eq!(saved_post.original_url, "https://youtube.com/watch?v=v1");

        let saved_creator = creator::Entity::find()
            .filter(creator::Column::ExternalId.eq("pc1"))
            .one(&db)
            .await
            .expect("DB error")
            .unwrap();
        assert_eq!(saved_creator.name, "Playlist Creator");
    }

    /// Diagnostic: verify serde handles `_type: "url"` entries from `--flat-playlist`.
    #[test]
    fn test_flat_playlist_entry_deserialization() {
        // This is what yt-dlp actually returns for --flat-playlist entries
        let json = r#"{
            "_type": "playlist",
            "id": "PLtest123",
            "title": "Test Playlist",
            "entries": [
                {
                    "_type": "url",
                    "id": "pXeF-IUr46M",
                    "url": "https://www.youtube.com/watch?v=pXeF-IUr46M",
                    "title": "First Video",
                    "ie_key": "Youtube",
                    "duration": 300
                },
                {
                    "_type": "url",
                    "id": "jUS4cyOA8To",
                    "url": "https://www.youtube.com/watch?v=jUS4cyOA8To",
                    "title": "Second Video"
                }
            ]
        }"#;

        let output: YtDlpOutput =
            serde_json::from_str(json).expect("Should deserialize flat-playlist JSON");

        match output {
            YtDlpOutput::Playlist(p) => {
                assert_eq!(p.id, "PLtest123");
                let entries = p.entries.expect("Should have entries");
                assert_eq!(entries.len(), 2);

                // Check that entries were parsed and have URLs
                for entry in &entries {
                    match entry {
                        YtDlpOutput::Video(v) | YtDlpOutput::VideoFallback(v) => {
                            eprintln!(
                                "Entry id={}, url={:?}, webpage_url={:?}",
                                v.id, v.url, v.webpage_url
                            );
                            assert!(v.url.is_some(), "Entry {} should have url field", v.id);
                        }
                        _ => panic!("Entry should be Video or VideoFallback"),
                    }
                }
            }
            _ => panic!("Should be a Playlist"),
        }
    }
}
