use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Manager, State};
use uuid::Uuid;

use crate::{
    auth::cookie_manager::CookieManager,
    entity::{download_task, post, source},
    metadata::{fetcher, models::YtDlpOutput, store},
    queue::DownloadQueue,
    AppState,
};

#[derive(Serialize)]
pub struct SourceResponse {
    pub id: String,
    pub platform_id: String,
    pub name: String,
    pub url: String,
    pub source_type: String,
    pub sync_mode: String,
    pub is_active: bool,
    pub last_checked: Option<String>,
    pub post_count: i64,
}

#[derive(Serialize)]
pub struct AddSourceResponse {
    pub source_id: String,
    pub items_queued: usize,
}

#[derive(Deserialize)]
pub struct UpdateSourceRequest {
    pub source_id: String,
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

#[tauri::command]
pub async fn get_sources_command(
    state: State<'_, AppState>,
) -> Result<Vec<SourceResponse>, String> {
    let sources = source::Entity::find()
        .all(&state.db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let mut response = Vec::with_capacity(sources.len());

    for s in sources {
        // Count posts linked to this source
        let count = post::Entity::find()
            .filter(post::Column::SourceId.eq(s.id.clone()))
            .count(&state.db)
            .await
            .unwrap_or(0) as i64;

        response.push(SourceResponse {
            id: s.id,
            platform_id: s.platform_id,
            name: s.name,
            url: s.url,
            source_type: s.source_type,
            sync_mode: s.sync_mode,
            is_active: s.is_active,
            last_checked: s.last_checked.map(|t| t.to_rfc3339()),
            post_count: count,
        });
    }

    Ok(response)
}

#[tauri::command]
pub async fn delete_source_command(
    state: State<'_, AppState>,
    source_id: String,
) -> Result<(), String> {
    source::Entity::delete_by_id(source_id)
        .exec(&state.db)
        .await
        .map_err(|e| format!("Failed to delete source: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn update_source_command(
    state: State<'_, AppState>,
    request: UpdateSourceRequest,
) -> Result<(), String> {
    // Build the update model with only the fields that were provided
    let mut active = source::ActiveModel {
        id: Set(request.source_id.clone()),
        ..Default::default()
    };

    if let Some(name) = request.name {
        active.name = Set(name);
    }

    if let Some(is_active) = request.is_active {
        active.is_active = Set(is_active);
    }

    source::Entity::update(active)
        .exec(&state.db)
        .await
        .map_err(|e| format!("Failed to update source: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn add_source_command(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    queue: State<'_, DownloadQueue>,
    url: String,
) -> Result<AddSourceResponse, String> {
    // --- Dedup check: reject if a source with this URL already exists ---
    let existing = source::Entity::find()
        .filter(source::Column::Url.eq(url.clone()))
        .one(&state.db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if let Some(existing_source) = existing {
        return Err(format!(
            "A source with this URL already exists: \"{}\"",
            existing_source.name
        ));
    }

    let cookie_manager = app.state::<Arc<CookieManager>>();

    // Cookie prep
    let mut temp_cookie_path = None;
    let platform_id = crate::platform::detect_platform(&url);

    if let Some(pid) = platform_id {
        if let Ok(Some(path)) = cookie_manager.create_temp_cookie_file(pid).await {
            temp_cookie_path = Some(path);
        }
    }

    // Fetch raw metadata (playlist format)
    let raw_output = fetcher::fetch_metadata(&app, &url, temp_cookie_path.as_ref())
        .await
        .map_err(|e| e.to_string());

    if let Some(path) = temp_cookie_path {
        let _ = cookie_manager.cleanup_temp_file(&path).await;
    }

    let output = raw_output?;

    // --- Single-video guard: reject non-playlist URLs ---
    if !matches!(output, YtDlpOutput::Playlist(_)) {
        return Err(
            "This URL points to a single video, not a playlist or channel. Use the download button instead."
                .to_string(),
        );
    }

    // Save metadata internally - this handles generating source and posts
    let saved_id = store::save_metadata(&state.db, output)
        .await
        .map_err(|e| format!("DB Error: {}", e))?;

    let mut items_queued = 0;

    // Find all the posts that have this source_id
    let child_posts = post::Entity::find()
        .filter(post::Column::SourceId.eq(saved_id.clone()))
        .all(&state.db)
        .await
        .map_err(|e| format!("Database error fetching posts: {}", e))?;

    for p in child_posts {
        // Only queue pending
        if p.status == "PENDING" {
            let task_id = Uuid::new_v4().to_string();
            let new_task = download_task::ActiveModel {
                id: Set(task_id.clone()),
                url: Set(p.original_url.clone()),
                post_id: Set(Some(p.id.clone())),
                status: Set("QUEUED".to_string()),
                priority: Set(5), // Background priority
                progress: Set(0.0),
                retries: Set(0),
                max_retries: Set(3),
                format_selection: Set(None), // Best auto for playlist items
                created_at: Set(Utc::now()),
                ..Default::default()
            };

            if new_task.insert(&state.db).await.is_ok() {
                items_queued += 1;
                queue.add_task();
            }
        }
    }

    Ok(AddSourceResponse {
        source_id: saved_id,
        items_queued,
    })
}
