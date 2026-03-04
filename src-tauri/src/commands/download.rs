use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::auth::cookie_manager::CookieManager;
use crate::entity::{download_task, post};
use crate::metadata::fetcher;
use crate::metadata::format_processor::{self, ProcessedMetadata};
use crate::metadata::models::{YtDlpOutput, YtDlpVideo};
use crate::queue::DownloadQueue;
use crate::AppState;
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct CreateDownloadTaskRequest {
    pub url: String,
    pub format_selection: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct DownloadTaskInfo {
    pub id: String,
    pub url: String,
    pub status: String,
    pub priority: i32,
    pub progress: f32,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub error_message: Option<String>,
    pub retries: i32,
    pub max_retries: i32,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub downloaded_bytes: Option<i64>,
    pub total_bytes: Option<i64>,
    pub title: Option<String>,
    pub thumbnail: Option<String>,
}

impl DownloadTaskInfo {
    fn new(m: download_task::Model, p: Option<post::Model>) -> Self {
        let (title, thumbnail) = if let Some(post) = p {
            let title = post.title;
            // Extract thumbnail from raw_json if available
            let thumbnail = post.raw_json.and_then(|json| {
                let video: Result<YtDlpVideo, _> = serde_json::from_str(&json);
                video.ok().and_then(|v| v.best_thumbnail())
            });
            (title, thumbnail)
        } else {
            (None, None)
        };

        Self {
            id: m.id,
            url: m.url,
            status: m.status,
            priority: m.priority,
            progress: m.progress,
            speed: m.speed,
            eta: m.eta,
            error_message: m.error_message,
            retries: m.retries,
            max_retries: m.max_retries,
            created_at: m.created_at.to_rfc3339(),
            started_at: m.started_at.map(|t| t.to_rfc3339()),
            completed_at: m.completed_at.map(|t| t.to_rfc3339()),
            downloaded_bytes: m.downloaded_bytes,
            total_bytes: m.total_bytes,
            title,
            thumbnail,
        }
    }
}

#[tauri::command]
pub async fn create_download_task(
    state: State<'_, AppState>,
    queue: State<'_, DownloadQueue>,
    request: CreateDownloadTaskRequest,
) -> Result<String, String> {
    let task_id = Uuid::new_v4().to_string();

    // 1. Create task in DB
    let new_task: download_task::ActiveModel = download_task::ActiveModel {
        id: Set(task_id.clone()),
        url: Set(request.url.clone()),
        status: Set("QUEUED".to_string()),
        priority: Set(10), // Default high priority for manual
        progress: Set(0.0),
        retries: Set(0),
        max_retries: Set(3),
        format_selection: Set(request.format_selection),
        created_at: Set(Utc::now()),
        ..Default::default()
    };

    new_task
        .insert(&state.db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    // 2. Notify queue scheduler
    queue.add_task();

    tracing::info!("Task created and queued: {}", task_id);

    Ok(task_id)
}

#[tauri::command]
pub async fn cancel_download_task(
    state: State<'_, AppState>,
    queue: State<'_, DownloadQueue>,
    task_id: String,
) -> Result<(), String> {
    // Try to cancel a running task
    let was_running = queue.cancel_task(&task_id).await;

    if !was_running {
        // Task may be QUEUED but not yet picked up — mark it directly
        let _ = download_task::Entity::update(download_task::ActiveModel {
            id: Set(task_id.clone()),
            status: Set("CANCELLED".to_string()),
            ..Default::default()
        })
        .exec(&state.db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;
    }

    tracing::info!("Task cancelled: {}", task_id);
    Ok(())
}

#[tauri::command]
pub async fn retry_download_task(
    state: State<'_, AppState>,
    queue: State<'_, DownloadQueue>,
    task_id: String,
) -> Result<(), String> {
    // Only allow retry for FAILED or CANCELLED tasks
    let task = download_task::Entity::find_by_id(&task_id)
        .one(&state.db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Task not found")?;

    if task.status != "FAILED" && task.status != "CANCELLED" {
        return Err(format!(
            "Cannot retry task with status '{}'. Only FAILED or CANCELLED tasks can be retried.",
            task.status
        ));
    }

    // Reset task
    let _ = download_task::Entity::update(download_task::ActiveModel {
        id: Set(task_id.clone()),
        status: Set("QUEUED".to_string()),
        retries: Set(0),
        progress: Set(0.0),
        error_message: Set(None),
        started_at: Set(None),
        completed_at: Set(None),
        speed: Set(None),
        eta: Set(None),
        ..Default::default()
    })
    .exec(&state.db)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    // Wake scheduler
    queue.add_task();

    tracing::info!("Task retried: {}", task_id);
    Ok(())
}

#[tauri::command]
pub async fn get_queue_status(
    state: State<'_, AppState>,
    queue: State<'_, DownloadQueue>,
) -> Result<QueueStatusResponse, String> {
    let tasks_with_posts = download_task::Entity::find()
        .find_also_related(post::Entity)
        .order_by_desc(download_task::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(QueueStatusResponse {
        is_paused: queue.is_paused(),
        tasks: tasks_with_posts
            .into_iter()
            .map(|(task, post)| DownloadTaskInfo::new(task, post))
            .collect(),
    })
}

#[derive(Clone, Serialize)]
pub struct QueueStatusResponse {
    pub is_paused: bool,
    pub tasks: Vec<DownloadTaskInfo>,
}

#[tauri::command]
pub async fn pause_download_task(
    state: State<'_, AppState>,
    queue: State<'_, DownloadQueue>,
    task_id: String,
) -> Result<(), String> {
    let task = download_task::Entity::find_by_id(&task_id)
        .one(&state.db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Task not found")?;

    match task.status.as_str() {
        "PROCESSING" => {
            // Update DB first to avoid race condition where manager sees "PROCESSING"
            // and marks it as CANCELLED
            let _ = download_task::Entity::update(download_task::ActiveModel {
                id: Set(task_id.clone()),
                status: Set("PAUSED".to_string()),
                ..Default::default()
            })
            .exec(&state.db)
            .await;

            // Then cancel the running worker
            queue.cancel_task(&task_id).await;
        }
        "QUEUED" => {
            let _ = download_task::Entity::update(download_task::ActiveModel {
                id: Set(task_id.clone()),
                status: Set("PAUSED".to_string()),
                ..Default::default()
            })
            .exec(&state.db)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        }
        _ => {
            return Err(format!(
                "Cannot pause task with status '{}'. Only QUEUED or PROCESSING tasks can be paused.",
                task.status
            ));
        }
    }

    tracing::info!("Task paused: {}", task_id);
    Ok(())
}

#[tauri::command]
pub async fn resume_download_task(
    state: State<'_, AppState>,
    queue: State<'_, DownloadQueue>,
    task_id: String,
) -> Result<(), String> {
    let task = download_task::Entity::find_by_id(&task_id)
        .one(&state.db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Task not found")?;

    if task.status != "PAUSED" {
        return Err(format!(
            "Cannot resume task with status '{}'. Only PAUSED tasks can be resumed.",
            task.status
        ));
    }

    // Reset to QUEUED — yt-dlp's -c flag will resume partial downloads
    let _ = download_task::Entity::update(download_task::ActiveModel {
        id: Set(task_id.clone()),
        status: Set("QUEUED".to_string()),
        error_message: Set(None),
        speed: Set(None),
        eta: Set(None),
        ..Default::default()
    })
    .exec(&state.db)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    queue.add_task();

    tracing::info!("Task resumed: {}", task_id);
    Ok(())
}

#[tauri::command]
pub async fn pause_queue(queue: State<'_, DownloadQueue>) -> Result<(), String> {
    queue.pause_queue();
    Ok(())
}

#[tauri::command]
pub async fn resume_queue(queue: State<'_, DownloadQueue>) -> Result<(), String> {
    queue.resume_queue();
    Ok(())
}

/// Fetches metadata for a URL and returns processed, UI-ready format information.
///
/// Returns [`ProcessedMetadata`] for single videos, which includes deduplicated
/// video qualities, audio tracks, and subtitle tracks. For playlists or
/// unrecognized output types, returns a minimal `ProcessedMetadata` so the
/// frontend can still offer fallback mode.
#[tauri::command]
pub async fn fetch_metadata_command(
    app: tauri::AppHandle,
    cookie_manager: State<'_, Arc<CookieManager>>,
    url: String,
) -> Result<ProcessedMetadata, String> {
    // Generate temp cookie path if applicable (e.g. for age-gated/member videos)
    let mut temp_cookie_path = None;
    let platform_id = crate::platform::detect_platform(&url);

    if let Some(pid) = platform_id {
        if let Ok(Some(path)) = cookie_manager.create_temp_cookie_file(pid).await {
            tracing::info!("Using cookies for fetch_metadata_command on {}", pid);
            temp_cookie_path = Some(path);
        }
    }

    // Fetch raw metadata
    let raw_output = fetcher::fetch_metadata(&app, &url, temp_cookie_path.as_ref())
        .await
        .map_err(|e| e.to_string());

    // Cleanup cookies
    if let Some(path) = temp_cookie_path {
        let _ = cookie_manager.cleanup_temp_file(&path).await;
    }

    let raw_output = raw_output?;

    // Transform to ProcessedMetadata
    match raw_output {
        YtDlpOutput::Video(ref video) | YtDlpOutput::VideoFallback(ref video) => {
            Ok(format_processor::process_metadata(video))
        }
        YtDlpOutput::Playlist(ref playlist) => {
            // For playlists, return minimal metadata so frontend shows fallback mode
            Ok(ProcessedMetadata {
                id: playlist.id.clone(),
                title: playlist.title.clone(),
                uploader: playlist.uploader.clone(),
                duration: None,
                thumbnail_url: None,
                video_qualities: vec![],
                audio_tracks: vec![],
                subtitle_tracks: vec![],
                is_playlist: true,
            })
        }
    }
}

/// Removes completed and cancelled download tasks from the database.
///
/// Failed tasks are intentionally preserved so the user can retry them.
/// This only clears the download *log* — it never deletes downloaded files.
#[tauri::command]
pub async fn clear_download_history(state: State<'_, AppState>) -> Result<u64, String> {
    let result = download_task::Entity::delete_many()
        .filter(download_task::Column::Status.is_in(["COMPLETED", "CANCELLED"]))
        .exec(&state.db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    tracing::info!(
        "Cleared {} completed/cancelled tasks from history",
        result.rows_affected
    );

    Ok(result.rows_affected)
}

/// Bulk-requeues every failed download task.
///
/// Resets each failed task's progress, retries counter, and error message,
/// then wakes the scheduler so they get picked up immediately.
#[tauri::command]
pub async fn retry_all_failed(
    state: State<'_, AppState>,
    queue: State<'_, DownloadQueue>,
) -> Result<u64, String> {
    let result = download_task::Entity::update_many()
        .col_expr(
            download_task::Column::Status,
            sea_orm::sea_query::Expr::value("QUEUED"),
        )
        .col_expr(
            download_task::Column::Retries,
            sea_orm::sea_query::Expr::value(0),
        )
        .col_expr(
            download_task::Column::Progress,
            sea_orm::sea_query::Expr::value(0.0_f32),
        )
        .col_expr(
            download_task::Column::ErrorMessage,
            sea_orm::sea_query::Expr::value(Option::<String>::None),
        )
        .col_expr(
            download_task::Column::StartedAt,
            sea_orm::sea_query::Expr::value(Option::<chrono::DateTime<chrono::Utc>>::None),
        )
        .col_expr(
            download_task::Column::CompletedAt,
            sea_orm::sea_query::Expr::value(Option::<chrono::DateTime<chrono::Utc>>::None),
        )
        .col_expr(
            download_task::Column::Speed,
            sea_orm::sea_query::Expr::value(Option::<String>::None),
        )
        .col_expr(
            download_task::Column::Eta,
            sea_orm::sea_query::Expr::value(Option::<String>::None),
        )
        .filter(download_task::Column::Status.eq("FAILED"))
        .exec(&state.db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if result.rows_affected > 0 {
        // Single wake is enough — the scheduler loops until no QUEUED tasks remain.
        queue.add_task();
        tracing::info!("Re-queued {} failed tasks for retry", result.rows_affected);
    }

    Ok(result.rows_affected)
}
