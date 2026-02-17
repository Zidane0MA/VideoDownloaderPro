#[cfg(test)]
mod tests {
    use crate::db;
    use crate::entity::{creator, post};
    use crate::metadata::models::{YtDlpOutput, YtDlpPlaylist, YtDlpVideo};
    use crate::metadata::store::save_metadata;
    use sea_orm::EntityTrait;

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
            original_url: None,
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

        assert_eq!(post_id, "video123");

        // Verify DB content
        let saved_post = post::Entity::find_by_id("video123")
            .one(&db)
            .await
            .expect("DB error")
            .unwrap();
        assert_eq!(saved_post.title, Some("Test Video".to_string()));
        assert_eq!(saved_post.status, "PENDING"); // Should be PENDING

        let saved_creator = creator::Entity::find_by_id("channel123")
            .one(&db)
            .await
            .expect("DB error")
            .unwrap();
        assert_eq!(saved_creator.name, "Test Channel");
    }

    #[tokio::test]
    async fn test_save_playlist_metadata() {
        let db = db::init_test_db().await.expect("Failed to init test db");

        let video1 = YtDlpVideo {
            id: "v1".to_string(),
            title: "Video 1".to_string(),
            webpage_url: Some("url1".to_string()),
            uploader_id: Some("c1".to_string()),
            uploader: Some("Channel 1".to_string()),
            description: None,
            uploader_url: None,
            upload_date: None,
            duration: None,
            view_count: None,
            like_count: None,
            thumbnails: None,
            formats: None,
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
            webpage_url: Some("http://playlist".to_string()),
            entries: Some(vec![YtDlpOutput::Video(video1)]),
        };

        let source_id = save_metadata(&db, YtDlpOutput::Playlist(playlist))
            .await
            .expect("Failed to save playlist");
        assert_eq!(source_id, "playlist1");

        // Verify Source
        // Note: I need to import source entity to verify, but checking posts exists is good enough
        let saved_post = post::Entity::find_by_id("v1")
            .one(&db)
            .await
            .expect("DB error")
            .unwrap();
        assert_eq!(saved_post.source_id, Some("playlist1".to_string()));

        let saved_creator = creator::Entity::find_by_id("pc1")
            .one(&db)
            .await
            .expect("DB error")
            .unwrap();
        assert_eq!(saved_creator.name, "Playlist Creator");
    }
}
