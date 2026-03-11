use crate::download::{DownloadError, DownloadResult, DownloadWorker};
use crate::entity::{download_task, media, post};
use crate::AppState;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{watch, Mutex, Notify, OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;

/// Default order index for downloaded media
const DEFAULT_MEDIA_ORDER_INDEX: i32 = 0;

/// Completion progress percentage
const PROGRESS_COMPLETED: f32 = 100.0;

/// Media Types
const MEDIA_TYPE_VIDEO: &str = "VIDEO";
const MEDIA_TYPE_AUDIO: &str = "AUDIO";
const MEDIA_TYPE_IMAGE: &str = "IMAGE";

/// Base delay for exponential backoff on retries.
const RETRY_BASE_DELAY_SECS: u64 = 5;
/// Multiplier for exponential backoff on retries.
const RETRY_BACKOFF_MULTIPLIER: u64 = 2;

#[derive(Clone)]
pub struct DownloadQueue {
    app_handle: AppHandle,
    notify: Arc<Notify>,
    semaphore: Arc<Semaphore>,
    /// Parent cancellation token — cancelling this stops the scheduler + all workers.
    shutdown_token: CancellationToken,
    /// Per-task cancellation tokens keyed by task ID.
    task_tokens: Arc<Mutex<HashMap<i64, CancellationToken>>>,
    /// Global pause flag — when true, the scheduler stops picking up new tasks.
    paused: Arc<AtomicBool>,
    /// Receives live concurrency-limit updates from `update_setting`.
    concurrency_rx: watch::Receiver<usize>,
}

struct ErrorDetails {
    is_cancelled: bool,
    message: String,
    total: Option<u64>,
    downloaded: u64,
    filename: Option<String>,
}

impl ErrorDetails {
    fn from_error(err: &DownloadError) -> Self {
        match err {
            DownloadError::Cancelled {
                total_bytes,
                downloaded_bytes,
                filename,
            } => Self {
                is_cancelled: true,
                message: "Download cancelled".to_string(),
                total: *total_bytes,
                downloaded: *downloaded_bytes,
                filename: filename.clone(),
            },
            DownloadError::Failed {
                message,
                total_bytes,
                downloaded_bytes,
                filename,
            } => Self {
                is_cancelled: false,
                message: message.clone(),
                total: *total_bytes,
                downloaded: *downloaded_bytes,
                filename: filename.clone(),
            },
        }
    }
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
    pub async fn cancel_task(&self, task_id: i64) -> bool {
        let tokens = self.task_tokens.lock().await;
        if let Some(token) = tokens.get(&task_id) {
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
            if let Err(e) = download_task::Entity::update(download_task::ActiveModel {
                id: Set(task.id),
                status: Set("QUEUED".to_string()),
                ..Default::default()
            })
            .exec(db)
            .await
            {
                tracing::error!("Failed to recover stale task {}: {}", task.id, e);
            }
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
            if concurrency_rx.has_changed().unwrap_or(false) {
                let new_cap = *concurrency_rx.borrow_and_update();
                if new_cap > current_cap {
                    let extra = new_cap - current_cap;
                    self.semaphore.add_permits(extra);
                    tracing::info!(
                        "Concurrency raised {} → {} (+{} permits)",
                        current_cap,
                        new_cap,
                        extra
                    );
                } else if new_cap < current_cap {
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

            // Check for next queued task
            let task_model = match self.get_next_task().await {
                Some(task) => task,
                None => {
                    tokio::select! {
                        _ = self.notify.notified() => continue,
                        _ = self.shutdown_token.cancelled() => break,
                    }
                }
            };

            // Process the task
            if !self.process_next_task(task_model).await {
                break;
            }
        }
    }

    /// Process the next acquired task: acquire permit, lock DB, and spawn worker
    async fn process_next_task(&self, task_model: download_task::Model) -> bool {
        // Acquire permit
        let permit = tokio::select! {
            result = self.semaphore.clone().acquire_owned() => {
                match result {
                    Ok(p) => p,
                    Err(_) => {
                        tracing::error!("Semaphore closed, stopping scheduler");
                        return false;
                    }
                }
            }
            _ = self.shutdown_token.cancelled() => return false,
        };

        tracing::info!("Starting task: {}", task_model.id);
        let app = self.app_handle.clone();
        let task_id = task_model.id;
        let db = app.state::<AppState>().db.clone();

        // Setup cancellation token for this task
        let task_token = self.shutdown_token.child_token();
        self.task_tokens
            .lock()
            .await
            .insert(task_id, task_token.clone());

        // Optimistic locking
        let update_result = download_task::Entity::update_many()
            .col_expr(
                download_task::Column::Status,
                sea_orm::sea_query::Expr::value("PROCESSING"),
            )
            .col_expr(
                download_task::Column::StartedAt,
                sea_orm::sea_query::Expr::value(Utc::now()),
            )
            .filter(download_task::Column::Id.eq(task_id))
            .filter(download_task::Column::Status.eq("QUEUED"))
            .exec(&db)
            .await;

        match update_result {
            Ok(res) if res.rows_affected == 0 => {
                tracing::info!(
                    "Task {} status changed (paused/cancelled) before execution, skipping",
                    task_id
                );
                return true;
            }
            Err(e) => {
                tracing::error!("Failed to update task status: {}", e);
                return true;
            }
            _ => {} // Success
        }

        // Spawn worker
        let queue = self.clone();
        tokio::spawn(async move {
            queue
                .process_standalone_task(app, task_model, permit, task_token)
                .await;
        });

        true
    }

    async fn get_next_task(&self) -> Option<download_task::Model> {
        let db = &self.app_handle.state::<AppState>().db;

        download_task::Entity::find()
            .filter(download_task::Column::Status.eq("QUEUED"))
            .order_by_desc(download_task::Column::Priority)
            .order_by_asc(download_task::Column::CreatedAt)
            .one(db)
            .await
            .unwrap_or(None)
    }

    /// Process a standalone task. This executes the entire lifecycle of a single download.
    async fn process_standalone_task(
        &self,
        app: AppHandle,
        task: download_task::Model,
        _permit: OwnedSemaphorePermit, // Holds the semaphore permit until task is dropped
        task_token: CancellationToken,
    ) {
        let task_id = task.id;
        let db = app.state::<AppState>().db.clone();

        let (download_dir, rate_limit) = Self::resolve_download_settings(&app, &db).await;

        if let Ok(false) = tokio::fs::try_exists(&download_dir).await {
            if let Err(e) = tokio::fs::create_dir_all(&download_dir).await {
                tracing::error!(
                    "Failed to create download directory {}: {}",
                    download_dir.display(),
                    e
                );
            }
        }

        let worker = DownloadWorker::new(app.clone());

        match worker
            .execute_download(
                task_id,
                task.url.clone(),
                download_dir.clone(),
                task.format_selection.clone(),
                rate_limit,
                task_token,
                db.clone(),
            )
            .await
        {
            Ok(res) => {
                Self::handle_download_success(&app, &db, task_id, &res).await;

                // Re-fetch task to get the updated post_id from metadata resolution
                if let Ok(Some(updated_task)) = download_task::Entity::find_by_id(task_id).one(&db).await {
                    Self::create_media_and_thumbnails(&app, &db, &updated_task, &download_dir, &res).await;
                } else {
                    tracing::error!("Failed to fetch updated task {} for media creation", task_id);
                }
            }
            Err(err) => {
                Self::handle_download_error(
                    &app,
                    &db,
                    task_id,
                    &download_dir,
                    task.retries,
                    task.max_retries,
                    err,
                    self.notify.clone(),
                )
                .await;
            }
        }

        self.task_tokens.lock().await.remove(&task_id);
    }

    async fn resolve_download_settings(
        app: &AppHandle,
        db: &DatabaseConnection,
    ) -> (PathBuf, Option<String>) {
        use crate::entity::setting::Entity as Setting;

        let download_dir = {
            let custom_path = Setting::find_by_id("download_path")
                .one(db)
                .await
                .ok()
                .flatten()
                .map(|s| s.value)
                .filter(|v| !v.is_empty());

            match custom_path {
                Some(p) => {
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

        let rate_limit = {
            Setting::find_by_id("rate_limit")
                .one(db)
                .await
                .unwrap_or_default()
                .map(|s| s.value)
                .filter(|v| !v.is_empty())
        };

        (download_dir, rate_limit)
    }

    async fn handle_download_success(
        app: &AppHandle,
        db: &DatabaseConnection,
        task_id: i64,
        res: &DownloadResult,
    ) {
        tracing::info!("Task completed: {}", task_id);

        if let Err(e) = download_task::Entity::update(download_task::ActiveModel {
            id: Set(task_id),
            status: Set("COMPLETED".to_string()),
            completed_at: Set(Some(Utc::now())),
            progress: Set(PROGRESS_COMPLETED),
            downloaded_bytes: Set(Some(res.downloaded_bytes as i64)),
            total_bytes: Set(res.total_bytes.map(|b| b as i64)),
            speed: Set(None),
            eta: Set(None),
            error_message: Set(None),
            ..Default::default()
        })
        .exec(db)
        .await
        {
            tracing::error!("Failed to mark task {} as completed: {}", task_id, e);
        }

        if let Err(e) = app.emit("download-completed", task_id) {
            tracing::warn!(
                "Failed to emit download-completed event for task {}: {}",
                task_id,
                e
            );
        }
    }

    async fn create_media_and_thumbnails(
        app: &AppHandle,
        db: &DatabaseConnection,
        task: &download_task::Model,
        download_dir: &std::path::Path,
        res: &DownloadResult,
    ) {
        if let Some(ref post_id) = task.post_id {
            if let Err(e) = post::Entity::update(post::ActiveModel {
                id: Set(*post_id),
                status: Set("COMPLETED".to_string()),
                downloaded_at: Set(Some(Utc::now())),
                ..Default::default()
            })
            .exec(db)
            .await
            {
                tracing::error!("Failed to mark post {} as completed: {}", post_id, e);
            }

            if let Some(ref fname) = res.filename {
                let file_path = download_dir.join(fname);
                let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

                let media_type = match ext.to_lowercase().as_str() {
                    "mp4" | "webm" | "mkv" | "avi" | "mov" | "flv" => MEDIA_TYPE_VIDEO,
                    "mp3" | "m4a" | "wav" | "aac" | "ogg" | "opus" => MEDIA_TYPE_AUDIO,
                    "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" => MEDIA_TYPE_IMAGE,
                    _ => MEDIA_TYPE_VIDEO,
                };

                let media_model = media::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    post_id: Set(*post_id),
                    media_type: Set(media_type.to_string()),
                    file_path: Set(file_path.to_string_lossy().to_string()),
                    order_index: Set(DEFAULT_MEDIA_ORDER_INDEX),
                    file_size: Set(res.total_bytes.map(|b| b as i32)),
                    ..Default::default()
                };

                match media_model.insert(db).await {
                    Err(e) => tracing::error!("Failed to create media row for post {}: {}", post_id, e),
                    Ok(inserted) => {
                        tracing::info!(
                            "Media row created for post {} -> {}",
                            post_id,
                            file_path.display()
                        );

                        if let Ok(ffmpeg) = crate::sidecar::get_binary_path(
                            app,
                            crate::sidecar::types::SidecarBinary::Ffmpeg,
                        ) {
                            let thumbs = crate::download::post_process::process_thumbnails(
                                &ffmpeg, &file_path, media_type,
                            )
                            .await;

                            if let Err(e) = media::Entity::update(media::ActiveModel {
                                id: Set(inserted.id),
                                thumbnail_path: Set(thumbs.thumbnail_path),
                                ..Default::default()
                            })
                            .exec(db)
                            .await
                            {
                                tracing::error!(
                                    "Failed to update media thumbnail for post {}: {}",
                                    post_id,
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_download_error(
        app: &AppHandle,
        db: &DatabaseConnection,
        task_id: i64,
        download_dir: &std::path::Path,
        current_retries: i32,
        max_retries: i32,
        err: DownloadError,
        notify: Arc<Notify>,
    ) {
        let details = ErrorDetails::from_error(&err);
        let total_i64: Option<i64> = details.total.map(|v| v as i64);

        if let Err(e) = download_task::Entity::update_many()
            .col_expr(
                download_task::Column::DownloadedBytes,
                sea_orm::sea_query::Expr::value(details.downloaded as i64),
            )
            .col_expr(
                download_task::Column::TotalBytes,
                sea_orm::sea_query::Expr::value(total_i64),
            )
            .filter(download_task::Column::Id.eq(task_id))
            .exec(db)
            .await
        {
            tracing::error!("Failed to update stats for task {}: {}", task_id, e);
        }

        if details.is_cancelled {
            Self::handle_task_cancellation(
                app,
                db,
                task_id,
                download_dir,
                details.message,
                details.filename,
            )
            .await;
        } else {
            Self::handle_task_retry(
                app,
                db,
                task_id,
                current_retries,
                max_retries,
                details.message,
                notify,
            )
            .await;
        }
    }

    async fn handle_task_cancellation(
        app: &AppHandle,
        db: &DatabaseConnection,
        task_id: i64,
        download_dir: &std::path::Path,
        message: String,
        filename: Option<String>,
    ) {
        let current_task = download_task::Entity::find_by_id(task_id)
            .one(db)
            .await
            .unwrap_or(None);

        let current_status = current_task.as_ref().map(|t| t.status.as_str());

        match current_status {
            Some("PAUSED") => {
                Self::handle_task_paused(app, db, task_id).await;
            }
            Some("PROCESSING") | Some("CANCELLED") => {
                Self::handle_task_cancelled(app, db, task_id, download_dir, message, filename)
                    .await;
            }
            _ => {
                tracing::info!(
                    "Task {} cancel/pause handler skipped — status already '{}'",
                    task_id,
                    current_status.unwrap_or("unknown")
                );
            }
        }
    }

    async fn handle_task_paused(app: &AppHandle, db: &DatabaseConnection, task_id: i64) {
        tracing::info!("Task paused: {}", task_id);
        if let Err(e) = download_task::Entity::update_many()
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
            .filter(download_task::Column::Id.eq(task_id))
            .filter(download_task::Column::Status.eq("PAUSED"))
            .exec(db)
            .await
        {
            tracing::error!("Failed to set task {} to PAUSED: {}", task_id, e);
        }
        if let Err(e) = app.emit("download-paused", task_id) {
            tracing::warn!(
                "Failed to emit download-paused event for task {}: {}",
                task_id,
                e
            );
        }
    }

    async fn handle_task_cancelled(
        app: &AppHandle,
        db: &DatabaseConnection,
        task_id: i64,
        download_dir: &std::path::Path,
        message: String,
        filename: Option<String>,
    ) {
        tracing::info!("Task cancelled: {}", task_id);

        if let Some(fname) = filename {
            let file_path = download_dir.join(&fname);
            let part_path = download_dir.join(format!("{}.part", fname));

            tracing::info!("Cleaning up files for cancelled task: {:?}", file_path);

            if matches!(tokio::fs::try_exists(&part_path).await, Ok(true)) {
                if let Err(e) = tokio::fs::remove_file(&part_path).await {
                    tracing::warn!("Failed to delete part file: {}", e);
                }
            }
            if matches!(tokio::fs::try_exists(&file_path).await, Ok(true)) {
                if let Err(e) = tokio::fs::remove_file(&file_path).await {
                    tracing::warn!("Failed to delete file: {}", e);
                }
            }
        }

        if let Err(e) = download_task::Entity::update_many()
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
            .filter(download_task::Column::Id.eq(task_id))
            .filter(download_task::Column::Status.is_in(["PROCESSING", "CANCELLED"]))
            .exec(db)
            .await
        {
            tracing::error!("Failed to set task {} to CANCELLED: {}", task_id, e);
        }
        if let Err(e) = app.emit("download-cancelled", task_id) {
            tracing::warn!(
                "Failed to emit download-cancelled event for task {}: {}",
                task_id,
                e
            );
        }
    }

    async fn handle_task_retry(
        app: &AppHandle,
        db: &DatabaseConnection,
        task_id: i64,
        current_retries: i32,
        max_retries: i32,
        message: String,
        notify: Arc<Notify>,
    ) {
        let new_retries = current_retries + 1;
        if new_retries < max_retries {
            Self::requeue_task(db, task_id, new_retries, max_retries, &message, notify).await;
        } else {
            Self::fail_task(app, db, task_id, new_retries, max_retries, &message).await;
        }
    }

    async fn requeue_task(
        db: &DatabaseConnection,
        task_id: i64,
        new_retries: i32,
        max_retries: i32,
        message: &str,
        notify: Arc<Notify>,
    ) {
        tracing::warn!(
            "Task {} failed (attempt {}/{}), requeueing: {}",
            task_id,
            new_retries,
            max_retries,
            message
        );

        if let Err(e) = download_task::Entity::update(download_task::ActiveModel {
            id: Set(task_id),
            status: Set("QUEUED".to_string()),
            retries: Set(new_retries),
            error_message: Set(Some(message.to_string())),
            ..Default::default()
        })
        .exec(db)
        .await
        {
            tracing::error!("Failed to requeue task {}: {}", task_id, e);
        }

        let delay = RETRY_BASE_DELAY_SECS * RETRY_BACKOFF_MULTIPLIER.pow(new_retries as u32);
        tokio::time::sleep(std::time::Duration::from_secs(delay)).await;

        notify.notify_one();
    }

    async fn fail_task(
        app: &AppHandle,
        db: &DatabaseConnection,
        task_id: i64,
        new_retries: i32,
        max_retries: i32,
        message: &str,
    ) {
        tracing::error!(
            "Task {} permanently failed after {} retries: {}",
            task_id,
            max_retries,
            message
        );

        if let Err(e) = download_task::Entity::update(download_task::ActiveModel {
            id: Set(task_id),
            status: Set("FAILED".to_string()),
            retries: Set(new_retries),
            error_message: Set(Some(message.to_string())),
            ..Default::default()
        })
        .exec(db)
        .await
        {
            tracing::error!("Failed to mark task {} as FAILED: {}", task_id, e);
        }

        if let Err(e) = app.emit("download-failed", task_id) {
            tracing::warn!(
                "Failed to emit download-failed event for task {}: {}",
                task_id,
                e
            );
        }
    }
}
