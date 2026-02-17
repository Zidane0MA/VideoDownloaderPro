use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::download::DownloadWorker;
use crate::entity::download_task;
use crate::AppState;

#[derive(serde::Deserialize)]
pub struct CreateDownloadTaskRequest {
    pub url: String,
    pub format_selection: Option<String>,
}

#[tauri::command]
pub async fn create_download_task(
    app: AppHandle,
    state: State<'_, AppState>,
    request: CreateDownloadTaskRequest,
) -> Result<String, String> {
    let task_id = Uuid::new_v4().to_string();

    // 1. Create task in DB
    let new_task = download_task::ActiveModel {
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

    // 2. Spawn worker (Phase 3.3 simple implementation: immediate execution)
    // In Phase 3.4 (Queue), this will be just "notify scheduler"
    let app_handle = app.clone();
    let task_id_clone = task_id.clone();
    let url_clone = request.url.clone();

    tauri::async_runtime::spawn(async move {
        let worker = DownloadWorker::new(app_handle.clone());

        // Update status to DOWNLOADING (skipped FETCHING_META for now as per minimal worker scope)
        // Actually, let's update status to DOWNLOADING first.
        let db = &app_handle.state::<AppState>().db;

        // TODO: Implement proper state transitions in worker or manager
        // For now, we just run execute_download which doesn't update DB status yet, only emits events.

        // Resolve download path
        let download_dir = app_handle
            .path()
            .download_dir()
            .unwrap_or(PathBuf::from("downloads"));
        // Ensure dir exists
        if !download_dir.exists() {
            let _ = std::fs::create_dir_all(&download_dir);
        }

        match worker
            .execute_download(task_id_clone.clone(), url_clone, download_dir)
            .await
        {
            Ok(_) => {
                // Update status to COMPLETED
                let _ = download_task::Entity::update(download_task::ActiveModel {
                    id: Set(task_id_clone),
                    status: Set("COMPLETED".to_string()),
                    completed_at: Set(Some(Utc::now())),
                    progress: Set(100.0),
                    ..Default::default()
                })
                .exec(db)
                .await;
            }
            Err(e) => {
                // Update status to FAILED
                let _ = download_task::Entity::update(download_task::ActiveModel {
                    id: Set(task_id_clone),
                    status: Set("FAILED".to_string()),
                    error_message: Set(Some(e)),
                    ..Default::default()
                })
                .exec(db)
                .await;
            }
        }
    });

    Ok(task_id)
}
