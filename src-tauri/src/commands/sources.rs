use chrono::Utc;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Set,
    Value,
};
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

    let mut count_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let posts_source_ids = post::Entity::find()
        .select_only()
        .column(post::Column::SourceId)
        .into_tuple::<Option<String>>()
        .all(&state.db)
        .await
        .unwrap_or_default();

    for id_opt in posts_source_ids {
        if let Some(id) = id_opt {
            *count_map.entry(id).or_insert(0) += 1;
        }
    }

    let mut response = Vec::with_capacity(sources.len());

    for s in sources {
        let count = count_map.get(&s.id).copied().unwrap_or(0);

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
    // Detach posts from this source (set source_id = NULL) to avoid FK violation
    post::Entity::update_many()
        .col_expr(post::Column::SourceId, Expr::value(Value::String(None)))
        .filter(post::Column::SourceId.eq(source_id.clone()))
        .exec(&state.db)
        .await
        .map_err(|e| format!("Failed to detach posts: {}", e))?;

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

const QUEUE_PRIORITY: i32 = 5;
const QUEUE_MAX_RETRIES: i32 = 3;

async fn handle_tiktok_source(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
    url: &str,
    section: crate::metadata::tiktok::TikTokSection,
) -> Result<String, String> {
    let username = crate::metadata::tiktok::helpers::extract_tiktok_username(url)
        .ok_or("Could not parse TikTok username from URL")?;

    let cookie_manager = app.state::<Arc<CookieManager>>();
    let cookies = cookie_manager
        .get_session("tiktok")
        .await
        .map_err(|e| e.to_string())?
        .ok_or("TikTok session not found. Please log in first.")?;

    let output = crate::metadata::tiktok::TikTokFetcher::new()
        .fetch_section(&cookies, &username, section, None)
        .await
        .map_err(|e| e.to_string())?;

    let saved_id = store::save_metadata(&state.db, output)
        .await
        .map_err(|e| format!("DB Error: {}", e))?;
    Ok(saved_id)
}

async fn handle_ytdlp_source(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
    url: &str,
) -> Result<String, String> {
    let cookie_manager = app.state::<Arc<CookieManager>>();
    let mut temp_cookie_path = None;
    let platform_id = crate::platform::detect_platform(url);

    if let Some(pid) = platform_id {
        if let Ok(Some(path)) = cookie_manager.create_temp_cookie_file(pid).await {
            temp_cookie_path = Some(path);
        }
    }

    let raw_output = fetcher::fetch_metadata(app, url, temp_cookie_path.as_ref())
        .await
        .map_err(|e| e.to_string());

    if let Some(path) = temp_cookie_path {
        if let Err(e) = cookie_manager.cleanup_temp_file(&path).await {
            tracing::warn!("Failed to cleanup cookies: {}", e);
        }
    }

    let output = raw_output?;

    if !matches!(output, YtDlpOutput::Playlist(_)) {
        return Err(
            "This URL points to a single video, not a playlist or channel. Use the download button instead."
                .to_string(),
        );
    }

    let saved_id = store::save_metadata(&state.db, output)
        .await
        .map_err(|e| format!("DB Error: {}", e))?;
    Ok(saved_id)
}

async fn queue_posts(
    state: &State<'_, AppState>,
    queue: &State<'_, DownloadQueue>,
    source_id: String,
    selected_ids: Option<Vec<String>>,
) -> Result<usize, String> {
    let mut items_queued = 0;
    let selection_filter: Option<std::collections::HashSet<String>> =
        selected_ids.map(|ids| ids.into_iter().collect());

    let child_posts = post::Entity::find()
        .filter(post::Column::SourceId.eq(source_id))
        .all(&state.db)
        .await
        .map_err(|e| format!("Database error fetching posts: {}", e))?;

    for p in child_posts {
        if p.original_url.is_empty() {
            tracing::warn!("Skipping post {} — no download URL available", p.id);
            continue;
        }

        if let Some(ref filter) = selection_filter {
            if !filter.contains(&p.id) {
                continue;
            }
        }

        if p.status == "PENDING" {
            let task_id = Uuid::new_v4().to_string();
            let new_task = download_task::ActiveModel {
                id: Set(task_id),
                url: Set(p.original_url),
                post_id: Set(Some(p.id)),
                status: Set("QUEUED".to_string()),
                priority: Set(QUEUE_PRIORITY),
                progress: Set(0.0),
                retries: Set(0),
                max_retries: Set(QUEUE_MAX_RETRIES),
                format_selection: Set(None),
                created_at: Set(Utc::now()),
                ..Default::default()
            };

            if new_task.insert(&state.db).await.is_ok() {
                items_queued += 1;
                queue.add_task();
            }
        }
    }

    Ok(items_queued)
}

#[tauri::command]
pub async fn add_source_command(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    queue: State<'_, DownloadQueue>,
    url: String,
    // Optional list of video IDs (from `PlaylistEntry.id`) to queue.
    // When `None`, all videos in the playlist are queued.
    selected_ids: Option<Vec<String>>,
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

    let tiktok_section = crate::metadata::tiktok::helpers::detect_tiktok_section(&url);
    let saved_id = if let Some(section) = tiktok_section {
        handle_tiktok_source(&app, &state, &url, section).await?
    } else {
        handle_ytdlp_source(&app, &state, &url).await?
    };

    let items_queued = queue_posts(&state, &queue, saved_id.clone(), selected_ids).await?;

    Ok(AddSourceResponse {
        source_id: saved_id,
        items_queued,
    })
}
