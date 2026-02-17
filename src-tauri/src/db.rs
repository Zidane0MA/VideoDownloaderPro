use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use std::path::PathBuf;
use std::time::Duration;

use crate::migration::Migrator;

/// Initializes the SQLite database, runs pending migrations, and returns the connection.
///
/// The database file is stored at `<app_data_dir>/videodownloaderpro.db`.
pub async fn init_db(app_data_dir: PathBuf) -> Result<DatabaseConnection, DbErr> {
    // Ensure the directory exists
    std::fs::create_dir_all(&app_data_dir).map_err(|e| {
        DbErr::Custom(format!(
            "Failed to create app data directory {}: {}",
            app_data_dir.display(),
            e
        ))
    })?;

    let db_path = app_data_dir.join("videodownloaderpro.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    tracing::info!(path = %db_path.display(), "Connecting to SQLite database");

    // Configure connection options to reduce log verbosity
    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false); // Disable verbose SQL logging

    let db = Database::connect(opt).await?;

    // Run pending migrations
    tracing::info!("Running pending database migrations...");
    Migrator::up(&db, None).await?;
    tracing::info!("Database migrations complete");

    Ok(db)
}

/// Creates an in-memory SQLite database for testing.
pub async fn init_test_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect("sqlite::memory:").await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}
