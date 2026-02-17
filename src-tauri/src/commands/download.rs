use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::entity::download_task;
use crate::queue::DownloadQueue;
use crate::AppState;

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
}

impl From<download_task::Model> for DownloadTaskInfo {
    fn from(m: download_task::Model) -> Self {
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
    let tasks = download_task::Entity::find()
        .order_by_desc(download_task::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(QueueStatusResponse {
        is_paused: queue.is_paused(),
        tasks: tasks.into_iter().map(DownloadTaskInfo::from).collect(),
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
