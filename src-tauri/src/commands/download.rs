use chrono::Utc;
use sea_orm::{ActiveModelTrait, Set};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::entity::download_task;
use crate::queue::DownloadQueue;
use crate::AppState;

#[derive(serde::Deserialize)]
pub struct CreateDownloadTaskRequest {
    pub url: String,
    pub format_selection: Option<String>,
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
