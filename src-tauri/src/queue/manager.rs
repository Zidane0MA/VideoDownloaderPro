use crate::download::DownloadWorker;
use crate::entity::{download_task, media, post};
use crate::AppState;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{watch, Mutex, Notify, Semaphore};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Base delay for exponential backoff on retries.
const RETRY_BASE_DELAY_SECS: u64 = 5;

#[derive(Clone)]
pub struct DownloadQueue {
    app_handle: AppHandle,
    notify: Arc<Notify>,
    semaphore: Arc<Semaphore>,
    /// Parent cancellation token — cancelling this stops the scheduler + all workers.
    shutdown_token: CancellationToken,
    /// Per-task cancellation tokens keyed by task ID.
    task_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
    /// Global pause flag — when true, the scheduler stops picking up new tasks.
    paused: Arc<AtomicBool>,
    /// Receives live concurrency-limit updates from `update_setting`.
    concurrency_rx: watch::Receiver<usize>,
}

impl DownloadQueue {
    /// Create a new queue.
    ///
    /// * `initial_concurrency` – number of slots read from the DB at startup.
    /// * `concurrency_rx` – watch receiver; the scheduler polls this to apply
    ///   live limit changes without restarting.
    pub fn new(app_handle: AppHandle, concurrency_rx: watch::Receiver<usize>) -> Self {
        let initial_concurrency = *concurrency_rx.borrow();
        Self {
            app_handle,
            notify: Arc::new(Notify::new()),
            semaphore: Arc::new(Semaphore::new(initial_concurrency)),
            shutdown_token: CancellationToken::new(),
            task_tokens: Arc::new(Mutex::new(HashMap::new())),
            paused: Arc::new(AtomicBool::new(false)),
            concurrency_rx,
        }
    }

    /// Notify the scheduler that a new task is available.
    pub fn add_task(&self) {
        self.notify.notify_one();
    }

    /// Cancel a specific running task by ID.
    pub async fn cancel_task(&self, task_id: &str) -> bool {
        let tokens = self.task_tokens.lock().await;
        if let Some(token) = tokens.get(task_id) {
            token.cancel();
            true
        } else {
            false
        }
    }

    /// Trigger graceful shutdown of the scheduler and all workers.
    pub fn shutdown(&self) {
        tracing::info!("Shutting down download queue...");
        self.shutdown_token.cancel();
    }

    /// Pause the queue — no new tasks will be picked up.
    pub fn pause_queue(&self) {
        tracing::info!("Queue paused");
        self.paused.store(true, Ordering::SeqCst);
    }

    /// Resume the queue — scheduler resumes picking up tasks.
    pub fn resume_queue(&self) {
        tracing::info!("Queue resumed");
        self.paused.store(false, Ordering::SeqCst);
        self.notify.notify_one();
    }

    /// Check if the queue is globally paused.
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    /// Recover tasks that were left in PROCESSING state (e.g. after a crash).
    async fn recover_stale_tasks(&self) {
        let db = &self.app_handle.state::<AppState>().db;

        let stale_tasks = download_task::Entity::find()
            .filter(download_task::Column::Status.eq("PROCESSING"))
            .all(db)
            .await
            .unwrap_or_default();

        if stale_tasks.is_empty() {
            return;
        }

        tracing::warn!(
            "Recovering {} stale PROCESSING tasks -> QUEUED",
            stale_tasks.len()
        );

        for task in stale_tasks {
            let _ = download_task::Entity::update(download_task::ActiveModel {
                id: Set(task.id.clone()),
                status: Set("QUEUED".to_string()),
                ..Default::default()
            })
            .exec(db)
            .await;
        }
    }

    pub async fn start_scheduler(&self) {
        tracing::info!("Starting download queue scheduler...");

        // Recover any stale tasks from previous session
        self.recover_stale_tasks().await;

        // Track the concurrency cap as seen by this loop so we can diff.
        let mut current_cap = *self.concurrency_rx.borrow();
        let mut concurrency_rx = self.concurrency_rx.clone();

        loop {
            // Check for shutdown
            if self.shutdown_token.is_cancelled() {
                tracing::info!("Scheduler shutting down");
                break;
            }

            // Live-reload: apply any pending concurrency change.
            // `has_changed` is true once after each new send on the watch channel.
            if concurrency_rx.has_changed().unwrap_or(false) {
                let new_cap = *concurrency_rx.borrow_and_update();
                if new_cap > current_cap {
                    // Growing: add the extra permits so waiting tasks can start.
                    let extra = new_cap - current_cap;
                    self.semaphore.add_permits(extra);
                    tracing::info!(
                        "Concurrency raised {} → {} (+{} permits)",
                        current_cap,
                        new_cap,
                        extra
                    );
                } else if new_cap < current_cap {
                    // Shrinking: do nothing — the semaphore won't issue new permits
                    // once available count reaches the new cap.  In-flight tasks drain naturally.
                    tracing::info!(
                        "Concurrency lowered {} → {} (draining naturally)",
                        current_cap,
                        new_cap
                    );
                }
                current_cap = new_cap;
            }

            // Check for global pause — wait until resumed
            if self.paused.load(Ordering::SeqCst) {
                tokio::select! {
                    _ = self.notify.notified() => continue,
                    _ = self.shutdown_token.cancelled() => break,
                }
            }

            // Check for next queued task BEFORE acquiring a permit
            // This avoids blocking semaphore permits when queue is empty
            let task_model = match self.get_next_task().await {
                Some(task) => task,
                None => {
                    // No task: wait for notification or shutdown
                    tokio::select! {
                        _ = self.notify.notified() => continue,
                        _ = self.shutdown_token.cancelled() => break,
                    }
                }
            };

            // Now acquire a permit (waits if all slots are busy)
            let permit = tokio::select! {
                result = self.semaphore.clone().acquire_owned() => {
                    match result {
                        Ok(p) => p,
                        Err(_) => {
                            tracing::error!("Semaphore closed, stopping scheduler");
                            break;
                        }
                    }
                }
                _ = self.shutdown_token.cancelled() => break,
            };

            tracing::info!("Starting task: {}", task_model.id);
            let app = self.app_handle.clone();
            let task_id = task_model.id.clone();
            let url = task_model.url.clone();
            let format_selection = task_model.format_selection.clone();
            let max_retries = task_model.max_retries;
            let current_retries = task_model.retries;
            let db = app.state::<AppState>().db.clone();

            // Create a child cancellation token for this specific task
            let task_token = self.shutdown_token.child_token();
            self.task_tokens
                .lock()
                .await
                .insert(task_id.clone(), task_token.clone());

            let task_tokens = self.task_tokens.clone();
            let notify = self.notify.clone();

            // Optimistic locking: Try to flip status to PROCESSING only if it's still QUEUED.
            // This prevents race condition where user Pauses/Cancels task while it was waiting for semaphore.
            let update_result = download_task::Entity::update_many()
                .col_expr(
                    download_task::Column::Status,
                    sea_orm::sea_query::Expr::value("PROCESSING"),
                )
                .col_expr(
                    download_task::Column::StartedAt,
                    sea_orm::sea_query::Expr::value(Utc::now()),
                )
                .filter(download_task::Column::Id.eq(task_id.clone()))
                .filter(download_task::Column::Status.eq("QUEUED"))
                .exec(&db)
                .await;

            match update_result {
                Ok(res) if res.rows_affected == 0 => {
                    tracing::info!(
                        "Task {} status changed (paused/cancelled) before execution, skipping",
                        task_id
                    );
                    continue;
                }
                Err(e) => {
                    tracing::error!("Failed to update task status: {}", e);
                    continue;
                }
                _ => {} // Success
            }

            // Spawn worker
            tokio::spawn(async move {
                // Ensure permit is held until task completes
                let _permit = permit;

                let worker = DownloadWorker::new(app.clone());

                // Read download_path from DB settings; fall back to OS default
                let download_dir = {
                    use crate::entity::setting::Entity as Setting;
                    let db_ref = &app.state::<AppState>().db;
                    let custom_path = Setting::find_by_id("download_path")
                        .one(db_ref)
                        .await
                        .ok()
                        .flatten()
                        .map(|s| s.value)
                        .filter(|v| !v.is_empty());

                    match custom_path {
                        Some(p) => {
                            // Expand ~ to actual home directory
                            if p.starts_with("~/") || p.starts_with("~\\") {
                                if let Some(home) = dirs::home_dir() {
                                    home.join(&p[2..])
                                } else {
                                    PathBuf::from(p)
                                }
                            } else {
                                PathBuf::from(p)
                            }
                        }
                        None => app
                            .path()
                            .download_dir()
                            .unwrap_or(PathBuf::from("downloads")),
                    }
                };

                // Read rate_limit from DB
                let rate_limit = {
                    use crate::entity::setting::Entity as Setting;
                    let db_ref = &app.state::<AppState>().db;
                    Setting::find_by_id("rate_limit")
                        .one(db_ref)
                        .await
                        .unwrap_or_default()
                        .map(|s| s.value)
                        .filter(|v| !v.is_empty())
                };

                // Ensure dir exists
                if !download_dir.exists() {
                    let _ = std::fs::create_dir_all(&download_dir);
                }

                match worker
                    .execute_download(
                        task_id.clone(),
                        url,
                        download_dir.clone(),
                        format_selection,
                        rate_limit,
                        task_token,
                        db.clone(),
                    )
                    .await
                {
                    Ok(res) => {
                        tracing::info!("Task completed: {}", task_id);
                        let _ = download_task::Entity::update(download_task::ActiveModel {
                            id: Set(task_id.clone()),
                            status: Set("COMPLETED".to_string()),
                            completed_at: Set(Some(Utc::now())),
                            progress: Set(100.0),
                            downloaded_bytes: Set(Some(res.downloaded_bytes as i64)),
                            total_bytes: Set(res.total_bytes.map(|b| b as i64)),
                            speed: Set(None),
                            eta: Set(None),
                            error_message: Set(None),
                            ..Default::default()
                        })
                        .exec(&db)
                        .await;

                        // --- Update linked Post status to COMPLETED & create Media row ---
                        // Re-read the task to get the post_id
                        if let Ok(Some(updated_task)) =
                            download_task::Entity::find_by_id(&task_id).one(&db).await
                        {
                            if let Some(ref post_id) = updated_task.post_id {
                                // 1. Mark post as COMPLETED
                                let _ = post::Entity::update(post::ActiveModel {
                                    id: Set(post_id.clone()),
                                    status: Set("COMPLETED".to_string()),
                                    downloaded_at: Set(Some(Utc::now())),
                                    ..Default::default()
                                })
                                .exec(&db)
                                .await;

                                // 2. Create a media row for the downloaded file
                                if let Some(ref fname) = res.filename {
                                    let file_path = download_dir.join(fname);
                                    let ext = file_path
                                        .extension()
                                        .and_then(|e| e.to_str())
                                        .unwrap_or("");
                                    let media_type = match ext.to_lowercase().as_str() {
                                        "mp4" | "webm" | "mkv" | "avi" | "mov" | "flv" => "VIDEO",
                                        "mp3" | "m4a" | "wav" | "aac" | "ogg" | "opus" => "AUDIO",
                                        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" => "IMAGE",
                                        _ => "VIDEO",
                                    };

                                    let media_id = Uuid::new_v4().to_string();

                                    let media_model = media::ActiveModel {
                                        id: Set(media_id.clone()),
                                        post_id: Set(post_id.clone()),
                                        media_type: Set(media_type.to_string()),
                                        file_path: Set(file_path.to_string_lossy().to_string()),
                                        order_index: Set(0),
                                        file_size: Set(res.total_bytes.map(|b| b as i32)),
                                        ..Default::default()
                                    };

                                    if let Err(e) = media_model.insert(&db).await {
                                        tracing::error!(
                                            "Failed to create media row for post {}: {}",
                                            post_id,
                                            e
                                        );
                                    } else {
                                        tracing::info!(
                                            "Media row created for post {} -> {}",
                                            post_id,
                                            file_path.display()
                                        );

                                        // 3. Process thumbnails (best-effort):
                                        //    - Extract frame from video and scale to 300px
                                        if let Ok(ffmpeg) = crate::sidecar::get_binary_path(
                                            &app,
                                            crate::sidecar::types::SidecarBinary::Ffmpeg,
                                        ) {
                                            let thumbs =
                                                crate::download::post_process::process_thumbnails(
                                                    &ffmpeg, &file_path, media_type,
                                                )
                                                .await;

                                            // Update media row with thumbnail path
                                            let _ = media::Entity::update(media::ActiveModel {
                                                id: Set(media_id.clone()),
                                                thumbnail_path: Set(thumbs.thumbnail_path),
                                                ..Default::default()
                                            })
                                            .exec(&db)
                                            .await;
                                        }
                                    }
                                }
                            }
                        }

                        let _ = app.emit("download-completed", &task_id);
                    }
                    Err(err) => {
                        // Extract details from the typed error
                        use crate::download::DownloadError;
                        let (is_cancelled, message, total, downloaded, filename) = match &err {
                            DownloadError::Cancelled {
                                total_bytes,
                                downloaded_bytes,
                                filename,
                            } => (
                                true,
                                "Download cancelled".to_string(),
                                *total_bytes,
                                *downloaded_bytes,
                                filename.clone(),
                            ),
                            DownloadError::Failed {
                                message,
                                total_bytes,
                                downloaded_bytes,
                                filename,
                            } => (
                                false,
                                message.clone(),
                                *total_bytes,
                                *downloaded_bytes,
                                filename.clone(),
                            ),
                        };

                        // Update stats in DB regardless of outcome (preserve partial progress)
                        // This fixes "History size lost"
                        let total_i64: Option<i64> = match total {
                            Some(v) => Some(v as i64),
                            None => None,
                        };
                        let _ = download_task::Entity::update_many()
                            .col_expr(
                                download_task::Column::DownloadedBytes,
                                sea_orm::sea_query::Expr::value(downloaded as i64),
                            )
                            .col_expr(
                                download_task::Column::TotalBytes,
                                sea_orm::sea_query::Expr::value(total_i64),
                            )
                            .filter(download_task::Column::Id.eq(task_id.clone()))
                            .exec(&db)
                            .await;

                        if is_cancelled {
                            // Determine if this was a pause or cancel by reading current DB state.
                            let current_task = download_task::Entity::find_by_id(&task_id)
                                .one(&db)
                                .await
                                .unwrap_or(None);

                            let current_status = current_task.as_ref().map(|t| t.status.as_str());

                            match current_status {
                                Some("PAUSED") => {
                                    // Task was paused — update only if still PAUSED (CAS)
                                    tracing::info!("Task paused: {}", task_id);
                                    let _ = download_task::Entity::update_many()
                                        .col_expr(
                                            download_task::Column::Speed,
                                            sea_orm::sea_query::Expr::value(Option::<String>::None),
                                        )
                                        .col_expr(
                                            download_task::Column::Eta,
                                            sea_orm::sea_query::Expr::value(Option::<String>::None),
                                        )
                                        .col_expr(
                                            download_task::Column::ErrorMessage,
                                            sea_orm::sea_query::Expr::value(Option::<String>::None),
                                        )
                                        .filter(download_task::Column::Id.eq(task_id.clone()))
                                        .filter(download_task::Column::Status.eq("PAUSED"))
                                        .exec(&db)
                                        .await;
                                    let _ = app.emit("download-paused", &task_id);
                                }
                                Some("PROCESSING") | Some("CANCELLED") => {
                                    // Task is still in our expected state — safe to mark as CANCELLED
                                    tracing::info!("Task cancelled: {}", task_id);

                                    // --- CLEANUP LOGIC ---
                                    if let Some(fname) = filename {
                                        let file_path = download_dir.join(&fname);
                                        let part_path =
                                            download_dir.join(format!("{}.part", fname));

                                        tracing::info!(
                                            "Cleaning up files for cancelled task: {:?}",
                                            file_path
                                        );

                                        // Try deleting .part file first (most likely exist for incomplete download)
                                        if part_path.exists() {
                                            let _ = std::fs::remove_file(&part_path).map_err(|e| {
                                                tracing::warn!("Failed to delete part file: {}", e)
                                            });
                                        }
                                        // Try deleting main file (if it was somehow finalized or different format)
                                        if file_path.exists() {
                                            let _ = std::fs::remove_file(&file_path).map_err(|e| {
                                                tracing::warn!("Failed to delete file: {}", e)
                                            });
                                        }
                                    }

                                    let _ = download_task::Entity::update_many()
                                        .col_expr(
                                            download_task::Column::Status,
                                            sea_orm::sea_query::Expr::value("CANCELLED"),
                                        )
                                        .col_expr(
                                            download_task::Column::ErrorMessage,
                                            sea_orm::sea_query::Expr::value(Some(message)),
                                        )
                                        .col_expr(
                                            download_task::Column::Speed,
                                            sea_orm::sea_query::Expr::value(Option::<String>::None),
                                        )
                                        .col_expr(
                                            download_task::Column::Eta,
                                            sea_orm::sea_query::Expr::value(Option::<String>::None),
                                        )
                                        .filter(download_task::Column::Id.eq(task_id.clone()))
                                        // CAS: only update if status hasn't been changed by resume/retry
                                        .filter(
                                            download_task::Column::Status
                                                .is_in(["PROCESSING", "CANCELLED"]),
                                        )
                                        .exec(&db)
                                        .await;
                                    let _ = app.emit("download-cancelled", &task_id);
                                }
                                _ => {
                                    tracing::info!(
                                        "Task {} cancel/pause handler skipped — status already '{}'",
                                        task_id,
                                        current_status.unwrap_or("unknown")
                                    );
                                }
                            }
                        } else {
                            // Retry logic: check if we can retry
                            let new_retries = current_retries + 1;
                            if new_retries < max_retries {
                                tracing::warn!(
                                    "Task {} failed (attempt {}/{}), requeueing: {}",
                                    task_id,
                                    new_retries,
                                    max_retries,
                                    message
                                );

                                let _ = download_task::Entity::update(download_task::ActiveModel {
                                    id: Set(task_id.clone()),
                                    status: Set("QUEUED".to_string()),
                                    retries: Set(new_retries),
                                    error_message: Set(Some(message)),
                                    ..Default::default()
                                })
                                .exec(&db)
                                .await;

                                // Exponential backoff
                                let delay = RETRY_BASE_DELAY_SECS * 2u64.pow(new_retries as u32);
                                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;

                                notify.notify_one();
                            } else {
                                tracing::error!(
                                    "Task {} permanently failed after {} retries: {}",
                                    task_id,
                                    max_retries,
                                    message
                                );

                                let _ = download_task::Entity::update(download_task::ActiveModel {
                                    id: Set(task_id.clone()),
                                    status: Set("FAILED".to_string()),
                                    retries: Set(new_retries),
                                    error_message: Set(Some(message)),
                                    ..Default::default()
                                })
                                .exec(&db)
                                .await;

                                let _ = app.emit("download-failed", &task_id);
                            }
                        }
                    }
                }

                // Cleanup task token
                task_tokens.lock().await.remove(&task_id);
            });
        }
    }

    async fn get_next_task(&self) -> Option<download_task::Model> {
        let db = &self.app_handle.state::<AppState>().db;

        // Priority queue: higher priority first, then FIFO by creation time
        download_task::Entity::find()
            .filter(download_task::Column::Status.eq("QUEUED"))
            .order_by_desc(download_task::Column::Priority)
            .order_by_asc(download_task::Column::CreatedAt)
            .one(db)
            .await
            .unwrap_or(None)
    }
}
