use super::parser::Parser;
use crate::auth::cookie_manager::CookieManager;
use crate::entity::download_task;
use crate::metadata::format_processor::DownloadOptions;
use crate::metadata::{fetcher, store};
use crate::sidecar::{get_binary_path, types::SidecarBinary};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::Serialize;
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tauri::Manager;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

/// Minimum interval between progress emissions to avoid flooding the IPC bridge.
const PROGRESS_THROTTLE: Duration = Duration::from_millis(500);

/// Kill the entire process tree rooted at the given child process.
/// On Windows, `child.kill()` only kills the immediate process, leaving
/// subprocesses (e.g. ffmpeg spawned by yt-dlp) running as orphans.
/// This function uses `taskkill /F /T /PID` to kill the full tree.
#[cfg(windows)]
async fn kill_process_tree(child: &mut tokio::process::Child) {
    if let Some(pid) = child.id() {
        tracing::info!("Killing process tree for PID {}", pid);
        let output = tokio::process::Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .output()
            .await;
        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                tracing::info!(
                    "taskkill PID={}: stdout='{}' stderr='{}'",
                    pid,
                    stdout.trim(),
                    stderr.trim()
                );
            }
            Err(e) => {
                tracing::error!("taskkill failed for PID {}: {}", pid, e);
            }
        }
    } else {
        tracing::warn!("Cannot kill process tree: no PID available");
    }
}

#[cfg(not(windows))]
async fn kill_process_tree(child: &mut tokio::process::Child) {
    if let Err(e) = child.kill().await {
        tracing::error!("Failed to kill process tree: {}", e);
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct DownloadProgressPayload {
    pub task_id: i64,
    pub progress: f64,
    pub speed: String,
    pub eta: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}

pub struct DownloadWorker {
    app: AppHandle,
}

#[derive(Debug)]
pub struct DownloadResult {
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub filename: Option<String>,
}

#[derive(Debug)]
pub enum DownloadError {
    Cancelled {
        total_bytes: Option<u64>,
        downloaded_bytes: u64,
        filename: Option<String>,
    },
    Failed {
        message: String,
        total_bytes: Option<u64>,
        downloaded_bytes: u64,
        filename: Option<String>,
    },
}

/// Media file extensions used to identify downloaded content (vs thumbnails, .part files, etc.).
const MEDIA_EXTENSIONS: &[&str] = &[
    "mp4", "webm", "mkv", "avi", "mov", "flv", "mp3", "m4a", "wav", "aac", "ogg", "opus",
];

impl DownloadWorker {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    /// Find the newly downloaded media file by comparing the directory state
    /// before and after the download. Uses `tokio::fs::read_dir` to avoid blocking.
    async fn find_new_media_file(
        dir: &PathBuf,
        pre_download_files: &HashSet<OsString>,
    ) -> Option<String> {
        let mut entries = tokio::fs::read_dir(dir).await.ok()?;
        let mut new_media = Vec::new();

        while let Ok(Some(e)) = entries.next_entry().await {
            if pre_download_files.contains(&e.file_name()) {
                continue;
            }
            let path = e.path();
            let ext = path
                .extension()
                .and_then(|x| x.to_str())
                .unwrap_or("")
                .to_lowercase();
            if MEDIA_EXTENSIONS.contains(&ext.as_str()) {
                new_media.push(e);
            }
        }

        let mut max_len = 0;
        let mut max_file = None;

        for e in new_media {
            if let Ok(metadata) = tokio::fs::metadata(e.path()).await {
                if metadata.len() > max_len {
                    max_len = metadata.len();
                    max_file = Some(e.file_name().to_string_lossy().to_string());
                }
            }
        }

        max_file
    }

    async fn get_pre_download_files(dir: &PathBuf) -> HashSet<OsString> {
        let mut pre_download_files = HashSet::new();
        if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
            while let Ok(Some(e)) = entries.next_entry().await {
                pre_download_files.insert(e.file_name());
            }
        }
        pre_download_files
    }

    async fn prepare_auth_and_metadata(
        &self,
        task_id: i64,
        url: &str,
        db: &DatabaseConnection,
    ) -> Result<Option<PathBuf>, DownloadError> {
        let cookie_manager = self.app.state::<std::sync::Arc<CookieManager>>();
        let mut temp_cookie_path: Option<PathBuf> = None;
        let platform_id = crate::platform::detect_platform(url);

        if let Some(pid) = platform_id {
            match cookie_manager.create_temp_cookie_file(pid).await {
                Ok(Some(path)) => {
                    tracing::info!("Using cookies for platform: {}", pid);
                    temp_cookie_path = Some(path);
                }
                Ok(None) => {}
                Err(e) => tracing::warn!("Failed to create temp cookie file for {}: {}", pid, e),
            }
        }

        let task = download_task::Entity::find_by_id(task_id)
            .one(db)
            .await
            .map_err(|e| {
                if let Some(path) = &temp_cookie_path {
                    tracing::error!(
                        "DB error while having temp cookie {}: {}",
                        path.display(),
                        e
                    );
                }
                DownloadError::Failed {
                    message: format!("DB error: {}", e),
                    total_bytes: None,
                    downloaded_bytes: 0,
                    filename: None,
                }
            })?
            .ok_or_else(|| DownloadError::Failed {
                message: "Task not found".to_string(),
                total_bytes: None,
                downloaded_bytes: 0,
                filename: None,
            })?;

        if task.post_id.is_none() {
            tracing::info!("Task {} has no metadata (post_id), fetching...", task_id);

            match fetcher::fetch_metadata(&self.app, url, temp_cookie_path.as_ref(), None).await {
                Ok(metadata) => match store::save_metadata(
                    db,
                    metadata,
                    None,
                    None,
                    platform_id,
                    Some(url),
                )
                .await
                {
                    Ok(post_id) => {
                        tracing::info!(
                            "Metadata saved for task {}, linked to post {}",
                            task_id,
                            post_id
                        );
                        if let Err(e) = download_task::Entity::update(download_task::ActiveModel {
                            id: Set(task_id),
                            post_id: Set(Some(post_id)),
                            ..Default::default()
                        })
                        .exec(db)
                        .await
                        {
                            tracing::error!("Failed to update task post_id: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to save metadata for task {}: {}", task_id, e);
                        if let Some(path) = &temp_cookie_path {
                            if let Err(e) = cookie_manager.cleanup_temp_file(path).await {
                                tracing::warn!("Failed to cleanup temp cookie file: {}", e);
                            }
                        }
                        return Err(DownloadError::Failed {
                            message: format!("Metadata save error: {}", e),
                            total_bytes: None,
                            downloaded_bytes: 0,
                            filename: None,
                        });
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to fetch metadata for task {}: {}", task_id, e);
                    if let Some(path) = &temp_cookie_path {
                        if let Err(e) = cookie_manager.cleanup_temp_file(path).await {
                            tracing::warn!("Failed to cleanup temp cookie file: {}", e);
                        }
                    }
                    return Err(DownloadError::Failed {
                        message: format!("Metadata fetch error: {}", e),
                        total_bytes: None,
                        downloaded_bytes: 0,
                        filename: None,
                    });
                }
            }
        }

        Ok(temp_cookie_path)
    }

    fn build_yt_dlp_command(
        &self,
        url: &str,
        output_dir: &PathBuf,
        format_selection: Option<&String>,
        rate_limit: Option<&String>,
        temp_cookie_path: Option<&PathBuf>,
    ) -> Result<Command, DownloadError> {
        let binary_path = get_binary_path(&self.app, SidecarBinary::YtDlp).map_err(|e| {
            DownloadError::Failed {
                message: e.to_string(),
                total_bytes: None,
                downloaded_bytes: 0,
                filename: None,
            }
        })?;

        let mut cmd = Command::new(binary_path);
        cmd.env("PYTHONIOENCODING", "utf-8");
        cmd.env("PYTHONUTF8", "1");

        let deno_path =
            get_binary_path(&self.app, SidecarBinary::Deno).map_err(|e| DownloadError::Failed {
                message: format!("Deno not found: {}", e),
                total_bytes: None,
                downloaded_bytes: 0,
                filename: None,
            })?;
        let deno_arg = format!("deno:{}", deno_path.to_string_lossy());

        cmd.arg("--newline")
            .arg("--no-playlist")
            .arg("-c")
            .arg("-P")
            .arg(output_dir)
            .arg("--output")
            .arg("%(title)s.%(ext)s");

        if let Some(limit) = rate_limit {
            if !limit.trim().is_empty() {
                cmd.arg("--limit-rate").arg(limit.trim());
            }
        }

        cmd.arg("--js-runtimes").arg(deno_arg);

        if let Some(cookie_path) = temp_cookie_path {
            cmd.arg("--cookies").arg(cookie_path);
        }

        if let Some(fmt_str) = format_selection {
            if let Ok(opts) = serde_json::from_str::<DownloadOptions>(fmt_str) {
                if opts.audio_only {
                    cmd.arg("-f").arg("bestaudio");
                    cmd.arg("--extract-audio");
                    if let Some(ref audio_fmt) = opts.audio_extract_format {
                        cmd.arg("--audio-format").arg(audio_fmt);
                    }
                } else {
                    let video_part = opts.format_id.as_deref().unwrap_or("bestvideo");
                    let audio_part = opts.audio_format_id.as_deref().unwrap_or("bestaudio");
                    let format_string = format!("{}+{}/best", video_part, audio_part);
                    cmd.arg("-f").arg(&format_string);
                }

                if !opts.subtitle_langs.is_empty() {
                    cmd.arg("--write-subs");
                    cmd.arg("--sub-langs").arg(opts.subtitle_langs.join(","));
                    cmd.arg("--sub-format").arg("vtt");

                    if opts.embed_subs {
                        cmd.arg("--embed-subs");
                    }
                }

                if let Some(ref container) = opts.container {
                    cmd.arg("--merge-output-format").arg(container);
                }
            } else {
                cmd.arg("-f").arg(fmt_str);
            }
        }

        cmd.arg(url);

        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        Ok(cmd)
    }

    async fn handle_progress_updates(
        &self,
        task_id: i64,
        mut child: tokio::process::Child,
        cancel_token: CancellationToken,
        db: DatabaseConnection,
    ) -> Result<(tokio::process::Child, Option<u64>, u64, bool), DownloadError> {
        let stdout = child.stdout.take().ok_or_else(|| DownloadError::Failed {
            message: "Failed to open stdout".to_string(),
            total_bytes: None,
            downloaded_bytes: 0,
            filename: None,
        })?;

        let mut reader = BufReader::new(stdout);
        let mut buf = Vec::new();
        let parser = Parser::new();
        let mut last_emit = Instant::now();

        let mut final_total_bytes = None;
        let mut final_downloaded_bytes = 0;
        let mut was_killed = false;

        let read_result: Result<(), DownloadError> = loop {
            if cancel_token.is_cancelled() {
                tracing::info!("Download cancelled for task: {}", task_id);
                kill_process_tree(&mut child).await;
                if let Err(e) = child.wait().await {
                    tracing::warn!("Failed to wait for killed child: {}", e);
                }
                was_killed = true;
                break Err(DownloadError::Cancelled {
                    total_bytes: final_total_bytes,
                    downloaded_bytes: final_downloaded_bytes,
                    filename: None,
                });
            }

            tokio::select! {
                biased;
                _ = cancel_token.cancelled() => {
                    tracing::info!("Download cancelled for task: {}", task_id);
                    kill_process_tree(&mut child).await;
                    if let Err(e) = child.wait().await {
                        tracing::warn!("Failed to wait for killed child: {}", e);
                    }
                    was_killed = true;
                    break Err(DownloadError::Cancelled {
                        total_bytes: final_total_bytes,
                        downloaded_bytes: final_downloaded_bytes,
                        filename: None,
                    });
                }
                result = reader.read_until(b'\n', &mut buf) => {
                    match result {
                        Ok(0) => break Ok(()),
                        Ok(_) => {
                            let line = String::from_utf8_lossy(&buf);
                            use super::parser::ParseResult;

                            match parser.parse_line(&line) {
                                ParseResult::Progress(progress) => {
                                    if let Some(bytes) = progress.total_bytes {
                                        final_total_bytes = Some(bytes);
                                    }
                                    if let Some(bytes) = progress.downloaded_bytes {
                                        final_downloaded_bytes = bytes;
                                    }

                                    let now = Instant::now();
                                    if now.duration_since(last_emit) >= PROGRESS_THROTTLE
                                        || progress.progress >= 100.0
                                    {
                                        last_emit = now;

                                        let payload = DownloadProgressPayload {
                                            task_id,
                                            progress: progress.progress,
                                            speed: progress.speed.clone().unwrap_or_default(),
                                            eta: progress.eta.clone().unwrap_or_default(),
                                            downloaded_bytes: final_downloaded_bytes,
                                            total_bytes: final_total_bytes,
                                        };

                                        if let Err(e) = self.app.emit("download-progress", &payload) {
                                            tracing::error!("Failed to emit download progress: {}", e);
                                        }

                                        let mut update = download_task::Entity::update_many()
                                            .col_expr(
                                                download_task::Column::Progress,
                                                sea_orm::sea_query::Expr::value(progress.progress as f32),
                                            )
                                            .col_expr(
                                                download_task::Column::Speed,
                                                sea_orm::sea_query::Expr::value(progress.speed.clone()),
                                            )
                                            .col_expr(
                                                download_task::Column::Eta,
                                                sea_orm::sea_query::Expr::value(progress.eta.clone()),
                                            );

                                        if progress.downloaded_bytes.is_some() {
                                            update = update.col_expr(
                                                download_task::Column::DownloadedBytes,
                                                sea_orm::sea_query::Expr::value(progress.downloaded_bytes),
                                            );
                                        }

                                        if let Some(total) = progress.total_bytes {
                                            update = update.col_expr(
                                                download_task::Column::TotalBytes,
                                                sea_orm::sea_query::Expr::value(total),
                                            );
                                        }

                                        if let Err(e) = update
                                            .filter(download_task::Column::Id.eq(task_id))
                                            .exec(&db)
                                            .await
                                        {
                                            tracing::error!("Failed to update DB progress: {}", e);
                                        }
                                    }
                                }
                                ParseResult::Ignore => {}
                            }
                            buf.clear();
                        }
                        Err(e) => break Err(DownloadError::Failed {
                            message: format!("Failed to read stdout: {}", e),
                            total_bytes: final_total_bytes,
                            downloaded_bytes: final_downloaded_bytes,
                            filename: None,
                        }),
                    }
                }
            }
        };

        read_result?;
        Ok((child, final_total_bytes, final_downloaded_bytes, was_killed))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn execute_download(
        &self,
        task_id: i64,
        url: String,
        output_dir: PathBuf,
        format_selection: Option<String>,
        rate_limit: Option<String>,
        cancel_token: CancellationToken,
        db: DatabaseConnection,
    ) -> Result<DownloadResult, DownloadError> {
        let temp_cookie_path = self.prepare_auth_and_metadata(task_id, &url, &db).await?;

        let mut cmd = self.build_yt_dlp_command(
            &url,
            &output_dir,
            format_selection.as_ref(),
            rate_limit.as_ref(),
            temp_cookie_path.as_ref(),
        )?;

        let pre_download_files = Self::get_pre_download_files(&output_dir).await;

        let mut child = cmd.spawn().map_err(|e| DownloadError::Failed {
            message: format!("Failed to spawn yt-dlp: {}", e),
            total_bytes: None,
            downloaded_bytes: 0,
            filename: None,
        })?;

        let stderr = child.stderr.take().ok_or_else(|| DownloadError::Failed {
            message: "Failed to open stderr".to_string(),
            total_bytes: None,
            downloaded_bytes: 0,
            filename: None,
        })?;

        let stderr_lines = std::sync::Arc::new(Mutex::new(Vec::<String>::new()));
        let stderr_lines_clone = stderr_lines.clone();
        let stderr_handle = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut buf = Vec::new();
            while let Ok(n) = reader.read_until(b'\n', &mut buf).await {
                if n == 0 {
                    break;
                }
                let line = String::from_utf8_lossy(&buf);
                let trimmed = line.trim().to_string();
                if !trimmed.is_empty() {
                    tracing::warn!(target: "yt-dlp:stderr", "{}", trimmed);
                    stderr_lines_clone.lock().await.push(trimmed);
                }
                buf.clear();
            }
        });

        let progress_result = self
            .handle_progress_updates(task_id, child, cancel_token, db)
            .await;

        if let Err(e) = stderr_handle.await {
            tracing::warn!("Stderr task failed: {}", e);
        }

        let (mut final_child, final_total_bytes, final_downloaded_bytes, was_killed) =
            progress_result?;

        if was_killed {
            return Err(DownloadError::Cancelled {
                total_bytes: final_total_bytes,
                downloaded_bytes: final_downloaded_bytes,
                filename: None,
            });
        }

        let status = final_child
            .wait()
            .await
            .map_err(|e| DownloadError::Failed {
                message: e.to_string(),
                total_bytes: final_total_bytes,
                downloaded_bytes: final_downloaded_bytes,
                filename: None,
            })?;

        let result = if status.success() {
            let result_filename = Self::find_new_media_file(&output_dir, &pre_download_files).await;

            if result_filename.is_none() {
                tracing::warn!("Could not identify downloaded file via filesystem scan");
            } else {
                tracing::info!("Downloaded file (fs scan): {:?}", result_filename);
            }

            let mut actual_file_size = None;
            if let Some(ref fname) = result_filename {
                let file_path = output_dir.join(fname);
                match tokio::fs::metadata(&file_path).await {
                    Ok(m) => {
                        let size = m.len();
                        tracing::info!("Actual file size on disk: {} bytes", size);
                        actual_file_size = Some(size);
                    }
                    Err(e) => {
                        tracing::warn!("Could not read file metadata: {}", e);
                    }
                }
            }

            let total = actual_file_size.unwrap_or(0);

            Ok(DownloadResult {
                total_bytes: actual_file_size,
                downloaded_bytes: total,
                filename: result_filename,
            })
        } else {
            let stderr_output = stderr_lines.lock().await;
            let error_detail = if stderr_output.is_empty() {
                format!("yt-dlp exited with status: {}", status)
            } else {
                let tail: Vec<&str> = stderr_output
                    .iter()
                    .rev()
                    .take(5)
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
                tail.join("\n")
            };
            Err(DownloadError::Failed {
                message: error_detail,
                total_bytes: final_total_bytes,
                downloaded_bytes: final_downloaded_bytes,
                filename: None,
            })
        };

        if let Some(path) = temp_cookie_path {
            let cookie_manager = self.app.state::<std::sync::Arc<CookieManager>>();
            if let Err(e) = cookie_manager.cleanup_temp_file(&path).await {
                tracing::warn!("Failed to cleanup temp cookie file: {}", e);
            }
        }

        result
    }
}
