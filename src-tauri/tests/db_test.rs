use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use video_downloader_pro_lib::db;

#[tokio::test]
async fn test_migrations_apply_and_seed_data() {
    // Create an in-memory database and run migrations
    let db = db::init_test_db()
        .await
        .expect("Failed to initialize test database");

    // ── Verify all 8 tables exist ─────────────────────────────
    let tables_query = Statement::from_string(
        DatabaseBackend::Sqlite,
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != 'seaql_migrations' ORDER BY name".to_string(),
    );

    let rows = db
        .query_all(tables_query)
        .await
        .expect("Failed to query tables");

    let table_names: Vec<String> = rows
        .iter()
        .map(|row| row.try_get_by_index::<String>(0).unwrap())
        .collect();

    let expected_tables = vec![
        "creators",
        "download_tasks",
        "media",
        "platform_sessions",
        "platforms",
        "posts",
        "settings",
        "sources",
    ];

    assert_eq!(
        table_names, expected_tables,
        "Expected 8 tables, got: {:?}",
        table_names
    );

    // ── Verify platforms seed data (4 rows) ───────────────────
    let platforms_query = Statement::from_string(
        DatabaseBackend::Sqlite,
        "SELECT COUNT(*) FROM platforms".to_string(),
    );
    let result = db
        .query_one(platforms_query)
        .await
        .expect("Failed to query platforms")
        .expect("No result from platforms count");
    let count: i32 = result.try_get_by_index(0).unwrap();
    assert_eq!(count, 4, "Expected 4 seeded platforms, got {}", count);

    // ── Verify settings seed data (14 rows) ───────────────────
    let settings_query = Statement::from_string(
        DatabaseBackend::Sqlite,
        "SELECT COUNT(*) FROM settings".to_string(),
    );
    let result = db
        .query_one(settings_query)
        .await
        .expect("Failed to query settings")
        .expect("No result from settings count");
    let count: i32 = result.try_get_by_index(0).unwrap();
    assert_eq!(count, 14, "Expected 14 default settings, got {}", count);

    // ── Verify specific platform exists ───────────────────────
    let yt_query = Statement::from_string(
        DatabaseBackend::Sqlite,
        "SELECT name FROM platforms WHERE id = 'youtube'".to_string(),
    );
    let result = db
        .query_one(yt_query)
        .await
        .expect("Failed to query YouTube platform")
        .expect("YouTube platform not found");
    let name: String = result.try_get_by_index(0).unwrap();
    assert_eq!(name, "YouTube");

    // ── Verify specific setting exists ────────────────────────
    let setting_query = Statement::from_string(
        DatabaseBackend::Sqlite,
        "SELECT value FROM settings WHERE key = 'max_concurrent_downloads'".to_string(),
    );
    let result = db
        .query_one(setting_query)
        .await
        .expect("Failed to query setting")
        .expect("max_concurrent_downloads setting not found");
    let value: String = result.try_get_by_index(0).unwrap();
    assert_eq!(value, "3");

    // ── Verify indexes exist ──────────────────────────────────
    let idx_query = Statement::from_string(
        DatabaseBackend::Sqlite,
        "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'".to_string(),
    );
    let result = db
        .query_one(idx_query)
        .await
        .expect("Failed to query indexes")
        .expect("No result from index count");
    let count: i32 = result.try_get_by_index(0).unwrap();
    assert_eq!(count, 7, "Expected 7 custom indexes, got {}", count);
}
