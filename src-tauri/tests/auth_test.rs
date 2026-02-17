#[cfg(test)]
mod tests {
    use sea_orm::{Database, DatabaseConnection};
    use sea_orm_migration::MigratorTrait;

    use std::sync::Arc;
    use tokio::fs;
    use video_downloader_pro_lib::auth::cookie_manager::CookieManager;
    use video_downloader_pro_lib::migration::Migrator;

    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_cookie_manager_lifecycle() {
        let db = Arc::new(setup_test_db().await);
        let temp_dir = std::env::temp_dir().join("video_downloader_pro_test");
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = CookieManager::new(db.clone(), temp_dir.clone());
        manager.init().await.unwrap();

        let platform_id = "youtube";
        let cookies = "domain.com\tTRUE\t/\tFALSE\t1234567890\tname\tvalue";
        let method = "manual";

        // 1. Set Session
        manager
            .set_session(
                platform_id.to_string(),
                cookies.to_string(),
                method.to_string(),
            )
            .await
            .expect("Failed to set session");

        // 2. Get Session (Decrypt)
        let retrieved = manager.get_session(platform_id).await.unwrap();
        assert_eq!(retrieved, Some(cookies.to_string()));

        // 3. Create Temp File
        let file_path_opt = manager.create_temp_cookie_file(platform_id).await.unwrap();
        assert!(file_path_opt.is_some());
        let file_path = file_path_opt.unwrap();
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, cookies);

        // 4. Cleanup
        manager.cleanup_temp_file(&file_path).await.unwrap();
        assert!(!file_path.exists());

        // Cleanup test dir
        let _ = fs::remove_dir_all(temp_dir).await;
    }
}
