#[cfg(test)]
mod tests {

    #[tokio::test]
    #[ignore]
    async fn test_extract_from_local_db() -> Result<(), Box<dyn std::error::Error>> {
        use crate::auth::encryption;
        use crate::entity::platform_session;
        use sea_orm::{Database, EntityTrait};
        use std::path::PathBuf;

        println!("Starting local DB test...");

        // Start with %APPDATA% (Roaming)
        let app_data = std::env::var("APPDATA").map_err(|_| "Could not find APPDATA env var")?;

        // Correct path construction for Tauri app
        let db_path = PathBuf::from(app_data)
            .join("com.videodownloaderpro.app")
            .join("videodownloaderpro.db");

        println!("Looking for DB at: {:?}", db_path);

        if !db_path.exists() {
            println!("Skipping test: DB not found at {:?}", db_path);
            return Ok(());
        }

        let db_url = format!("sqlite://{}?mode=ro", db_path.to_string_lossy());
        println!("Connecting to DB URL: {}", db_url);

        let db = Database::connect(db_url).await?;

        let sessions = platform_session::Entity::find().all(&db).await?;
        println!("Found {} sessions in DB.", sessions.len());

        for session in sessions {
            println!("Processing session for platform: {}", session.platform_id);

            if let Some(encrypted) = session.encrypted_cookies {
                match encryption::decrypt_string(&encrypted) {
                    Ok(cookies) => {
                        println!(
                            "  -> Decrypted cookies successfully. Length: {}",
                            cookies.len()
                        );

                        // Verbose Debugging: Print all cookies
                        println!("  -> Cookie Dump for {}:", session.platform_id);
                        for line in cookies.lines() {
                            let trimmed = line.trim();
                            if trimmed.is_empty()
                                || (trimmed.starts_with('#') && !trimmed.starts_with("#HttpOnly_"))
                            {
                                continue;
                            }
                            let parts: Vec<&str> = line.split('\t').collect();
                            if parts.len() >= 7 {
                                let domain = parts[0];
                                let name = parts[5];
                                let value = parts[6];
                                let masked_value = if value.len() > 5 {
                                    format!("{}...", &value[0..5])
                                } else {
                                    "***".to_string()
                                };
                                println!("     - {} | {} = {}", domain, name, masked_value);
                            }
                        }

                        if session.platform_id == "tiktok" {
                            println!("  -> Attempting TikTok API fetch (no ID required)...");
                            let api_user =
                                crate::auth::api::UsernameFetcher::fetch_tiktok_username(&cookies)
                                    .await;
                            println!("  -> API Fetched Username: {:?}", api_user);
                        } else if session.platform_id == "x" || session.platform_id == "twitter" {
                            println!("  -> Attempting X API fetch");
                            let api_user =
                                crate::auth::api::UsernameFetcher::fetch_x_username(&cookies).await;
                            println!("  -> API Fetched Username: {:?}", api_user);
                        }

                        // YouTube (often doesn't have a simple username cookie)
                        if session.platform_id == "youtube" {
                            println!("  -> Attempting YouTube API fetch...");
                            let api_user =
                                crate::auth::api::UsernameFetcher::fetch_youtube_username(&cookies)
                                    .await;
                            println!("  -> API Fetched Username: {:?}", api_user);
                        }

                        if let Some(db_user) = &session.username {
                            println!("  -> DB Stored Username: {}", db_user);
                        }
                    }
                    Err(e) => println!("  -> Failed to decrypt cookies: {}", e),
                }
            } else {
                println!("  -> No encrypted cookies found for this session.");
            }
        }

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_tiktok_liked_api() -> Result<(), Box<dyn std::error::Error>> {
        use crate::auth::encryption;
        use crate::entity::platform_session;
        use crate::metadata::tiktok::TikTokFetcher;
        use sea_orm::{Database, EntityTrait};
        use std::path::PathBuf;

        println!("Starting TikTok Liked API integration test...");

        let app_data = std::env::var("APPDATA").map_err(|_| "Could not find APPDATA env var")?;
        let db_path = PathBuf::from(app_data)
            .join("com.videodownloaderpro.app")
            .join("videodownloaderpro.db");

        if !db_path.exists() {
            println!("DB not found at {:?} — skipping", db_path);
            return Ok(());
        }

        let db_url = format!("sqlite://{}?mode=ro", db_path.to_string_lossy());
        let db = Database::connect(db_url).await?;

        let session = platform_session::Entity::find_by_id("tiktok")
            .one(&db)
            .await?;

        let session = match session {
            Some(s) => s,
            None => {
                println!("No TikTok session found in DB — skipping");
                return Ok(());
            }
        };

        let encrypted = session
            .encrypted_cookies
            .ok_or("No encrypted cookies found")?;
        let cookies = encryption::decrypt_string(&encrypted)?;

        let username = session
            .username
            .ok_or("No username stored for TikTok session")?;

        println!("Using username: @{}", username);
        println!("Cookie length: {} bytes", cookies.len());

        let fetcher = TikTokFetcher::new();
        let result = fetcher
            .fetch_liked_videos(&cookies, &username, Some(5))
            .await;

        match result {
            Ok(output) => {
                if let crate::metadata::models::YtDlpOutput::Playlist(playlist) = output {
                    println!("✅ Playlist title: {}", playlist.title);
                    println!("   Playlist ID: {}", playlist.id);
                    if let Some(entries) = &playlist.entries {
                        println!("   Entries fetched: {}", entries.len());
                        assert!(!entries.is_empty(), "Expected at least 1 liked video");

                        // Print first entry details
                        if let Some(crate::metadata::models::YtDlpOutput::Video(v)) =
                            entries.first()
                        {
                            println!("   First video: {} ({})", v.title, v.id);
                            println!("   Uploader: {:?}", v.uploader);
                            println!("   URL: {:?}", v.webpage_url);
                        }
                    } else {
                        panic!("Playlist entries is None");
                    }
                } else {
                    panic!("Expected YtDlpOutput::Playlist, got Video");
                }
            }
            Err(e) => {
                println!("❌ Fetcher error: {}", e);
                // Don't panic — session may be expired in CI
                println!("   (This may be expected if cookies are expired)");
            }
        }

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_tiktok_saved_api() -> Result<(), Box<dyn std::error::Error>> {
        use crate::auth::encryption;
        use crate::entity::platform_session;
        use crate::metadata::tiktok::helpers::TikTokSection;
        use crate::metadata::tiktok::TikTokFetcher;
        use sea_orm::{Database, EntityTrait};
        use std::path::PathBuf;

        println!("Starting TikTok Saved API integration test...");

        let app_data = std::env::var("APPDATA").map_err(|_| "Could not find APPDATA env var")?;
        let db_path = PathBuf::from(app_data)
            .join("com.videodownloaderpro.app")
            .join("videodownloaderpro.db");

        if !db_path.exists() {
            println!("DB not found at {:?} — skipping", db_path);
            return Ok(());
        }

        let db_url = format!("sqlite://{}?mode=ro", db_path.to_string_lossy());
        let db = Database::connect(db_url).await?;

        let session = platform_session::Entity::find_by_id("tiktok")
            .one(&db)
            .await?;

        let session = match session {
            Some(s) => s,
            None => {
                println!("No TikTok session found in DB — skipping");
                return Ok(());
            }
        };

        let encrypted = session
            .encrypted_cookies
            .ok_or("No encrypted cookies found")?;
        let cookies = encryption::decrypt_string(&encrypted)?;

        let username = session
            .username
            .ok_or("No username stored for TikTok session")?;

        println!("Using username: @{}", username);
        println!("Cookie length: {} bytes", cookies.len());

        let fetcher = TikTokFetcher::new();
        let result = fetcher
            .fetch_section(&cookies, &username, TikTokSection::Saved, Some(5))
            .await;

        match result {
            Ok(output) => {
                if let crate::metadata::models::YtDlpOutput::Playlist(playlist) = output {
                    println!("✅ Playlist title: {}", playlist.title);
                    println!("   Playlist ID: {}", playlist.id);
                    assert!(
                        playlist.title.contains("Saved Videos"),
                        "Expected 'Saved Videos' in title, got: {}",
                        playlist.title
                    );
                    if let Some(entries) = &playlist.entries {
                        println!("   Entries fetched: {}", entries.len());
                        assert!(!entries.is_empty(), "Expected at least 1 saved video");

                        if let Some(crate::metadata::models::YtDlpOutput::Video(v)) =
                            entries.first()
                        {
                            println!("   First video: {} ({})", v.title, v.id);
                            println!("   Uploader: {:?}", v.uploader);
                            println!("   URL: {:?}", v.webpage_url);
                        }
                    } else {
                        panic!("Playlist entries is None");
                    }
                } else {
                    panic!("Expected YtDlpOutput::Playlist, got Video");
                }
            }
            Err(e) => {
                println!("❌ Fetcher error: {}", e);
                // Don't panic — session may be expired or saved list may be empty
                println!("   (This may be expected if cookies are expired or list is empty)");
            }
        }

        Ok(())
    }
}
