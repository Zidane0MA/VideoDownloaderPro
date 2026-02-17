use crate::download::DownloadWorker;
use crate::entity::download_task;
use crate::AppState;
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Notify, Semaphore};

#[derive(Clone)]
pub struct DownloadQueue {
    app_handle: AppHandle,
    notify: Arc<Notify>,
    semaphore: Arc<Semaphore>,
}

impl DownloadQueue {
    pub fn new(app_handle: AppHandle, max_concurrency: usize) -> Self {
        Self {
            app_handle,
            notify: Arc::new(Notify::new()),
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
        }
    }

    pub fn add_task(&self) {
        self.notify.notify_one();
    }

    pub async fn start_scheduler(&self) {
        tracing::info!("Starting download queue scheduler...");

        loop {
            // Wait for a slot to be available
            let permit = match self.semaphore.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => {
                    tracing::error!("Semaphore closed, stopping scheduler");
                    break;
                }
            };

            // Check for next queued task
            match self.get_next_task().await {
                Some(task_model) => {
                    tracing::info!("Starting task: {}", task_model.id);
                    let app = self.app_handle.clone();
                    let task_id = task_model.id.clone();
                    let url = task_model.url.clone();
                    let db = &app.state::<AppState>().db;

                    // Update status to PROCESSING
                    let _ = download_task::Entity::update(download_task::ActiveModel {
                        id: Set(task_id.clone()),
                        status: Set("PROCESSING".to_string()),
                        ..Default::default()
                    })
                    .exec(db)
                    .await;

                    // Spawn worker
                    tokio::spawn(async move {
                        // Ensure permit is held until task completes
                        let _permit = permit;

                        let worker = DownloadWorker::new(app.clone());
                        let download_dir = app
                            .path()
                            .download_dir()
                            .unwrap_or(PathBuf::from("downloads"));

                        // Ensure dir exists
                        if !download_dir.exists() {
                            let _ = std::fs::create_dir_all(&download_dir);
                        }

                        let db = &app.state::<AppState>().db;

                        match worker
                            .execute_download(task_id.clone(), url, download_dir)
                            .await
                        {
                            Ok(_) => {
                                tracing::info!("Task completed: {}", task_id);
                                let _ = download_task::Entity::update(download_task::ActiveModel {
                                    id: Set(task_id.clone()),
                                    status: Set("COMPLETED".to_string()),
                                    completed_at: Set(Some(Utc::now())),
                                    progress: Set(100.0),
                                    ..Default::default()
                                })
                                .exec(db)
                                .await;

                                let _ = app.emit("download-completed", &task_id);
                            }
                            Err(e) => {
                                tracing::error!("Task failed: {} - {}", task_id, e);
                                let _ = download_task::Entity::update(download_task::ActiveModel {
                                    id: Set(task_id.clone()),
                                    status: Set("FAILED".to_string()),
                                    error_message: Set(Some(e)),
                                    ..Default::default()
                                })
                                .exec(db)
                                .await;

                                let _ = app.emit("download-failed", &task_id);
                            }
                        }
                    });
                }
                None => {
                    // No task found, release permit and wait for notification
                    drop(permit);
                    self.notify.notified().await;
                }
            }
        }
    }

    async fn get_next_task(&self) -> Option<download_task::Model> {
        let db = &self.app_handle.state::<AppState>().db;

        // Simple priority queue: FIFO, maybe add priority column later
        download_task::Entity::find()
            .filter(download_task::Column::Status.eq("QUEUED"))
            .order_by_asc(download_task::Column::CreatedAt)
            .one(db)
            .await
            .unwrap_or(None)
    }
}
