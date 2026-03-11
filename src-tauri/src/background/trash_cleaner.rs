use chrono::{Duration, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::path::Path;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::time::{sleep, Duration as StdDuration};
use trash::delete as move_to_trash;

use crate::entity::{media, post, setting};

pub fn start_trash_cleaner(app: &AppHandle, db: Arc<DatabaseConnection>) {
    let _app_handle = app.clone(); // In case we need it later, just prefix with _
    tauri::async_runtime::spawn(async move {
        tracing::info!("Starting background trash cleaner task");

        loop {
            // Run cleanup every 12 hours
            if let Err(e) = run_cleanup(&db).await {
                tracing::error!("Error during trash cleanup: {}", e);
            }
            sleep(StdDuration::from_secs(12 * 3600)).await;
        }
    });
}

async fn run_cleanup(db: &DatabaseConnection) -> Result<(), String> {
    // Read `trash_auto_clean_days` from settings table
    let setting_model = setting::Entity::find_by_id("trash_auto_clean_days")
        .one(db)
        .await
        .map_err(|e| format!("Database error fetching setting: {}", e))?;

    let days_to_keep = setting_model
        .and_then(|s| s.value.parse::<i64>().ok())
        .unwrap_or(30); // Default to 30 days if not set

    if days_to_keep <= 0 {
        // Feature disabled
        return Ok(());
    }

    let threshold = Utc::now() - Duration::days(days_to_keep);

    // Find all posts deleted before the threshold
    let posts_to_delete = post::Entity::find()
        .filter(post::Column::DeletedAt.is_not_null())
        .filter(post::Column::DeletedAt.lt(threshold))
        .all(db)
        .await
        .map_err(|e| format!("DB error finding posts to clean: {}", e))?;

    if posts_to_delete.is_empty() {
        return Ok(());
    }

    tracing::info!("Found {} old posts to hard-delete", posts_to_delete.len());

    for p in posts_to_delete {
        let medias = media::Entity::find()
            .filter(media::Column::PostId.eq(p.id))
            .all(db)
            .await
            .unwrap_or_default();

        for m in medias {
            if Path::new(&m.file_path).exists() {
                let _ = move_to_trash(&m.file_path);
            }
            if let Some(thumb) = &m.thumbnail_path {
                if Path::new(thumb).exists() {
                    let _ = move_to_trash(thumb);
                }
            }
        }

        let _ = media::Entity::delete_many()
            .filter(media::Column::PostId.eq(p.id))
            .exec(db)
            .await;

        let _ = post::Entity::delete_by_id(p.id).exec(db).await;
    }

    Ok(())
}
