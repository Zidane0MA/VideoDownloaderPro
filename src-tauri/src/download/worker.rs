use super::parser::Parser;
use crate::auth::cookie_manager::CookieManager;
use crate::entity::download_task;
use crate::metadata::{fetcher, store};
use crate::sidecar::{get_binary_path, types::SidecarBinary};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::Serialize;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tauri::Manager;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken; // Ensure Manager is imported for state access

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
    let _ = child.kill().await;
}

#[derive(Clone, Serialize, Debug)]
pub struct DownloadProgressPayload {
    pub task_id: String,
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

impl DownloadWorker {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub async fn execute_download(
        &self,
        task_id: String,
        url: String,
        output_dir: PathBuf,
        format_selection: Option<String>,
        cancel_token: CancellationToken,
        db: DatabaseConnection,
    ) -> Result<DownloadResult, DownloadError> {
        let binary_path = get_binary_path(&self.app, SidecarBinary::YtDlp).map_err(|e| {
            DownloadError::Failed {
                message: e.to_string(),
                total_bytes: None,
                downloaded_bytes: 0,
                filename: None,
            }
        })?;

        // --- Auth / Cookie Setup ---
        // We do this BEFORE metadata fetch because age-gated videos require cookies even for metadata.
        let cookie_manager = self.app.state::<std::sync::Arc<CookieManager>>();
        let mut temp_cookie_path: Option<PathBuf> = None;
        let platform_id = if url.contains("youtube.com") || url.contains("youtu.be") {
            Some("youtube")
        } else if url.contains("tiktok.com") {
            Some("tiktok")
        } else if url.contains("instagram.com") {
            Some("instagram")
        } else if url.contains("x.com") || url.contains("twitter.com") {
            Some("x")
        } else {
            None
        };

        if let Some(pid) = platform_id {
            match cookie_manager.create_temp_cookie_file(pid).await {
                Ok(path) => {
                    if let Some(p) = path {
                        tracing::info!("Using cookies for platform: {}", pid);
                        temp_cookie_path = Some(p);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to create temp cookie file for {}: {}", pid, e);
                }
            }
        }

        // --- Metadata Fetch Step ---
        // Check if task already has a linked Post. If not, fetch and save metadata first.
        let task = download_task::Entity::find_by_id(&task_id)
            .one(&db)
            .await
            .map_err(|e| {
                // Try cleanup if we fail early
                if let Some(_path) = &temp_cookie_path {
                    // We can't await easily in map_err, but we should try.
                    // Verify if we can just log here.
                    tracing::error!("DB error: {}", e);
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

            // Pass the cookie path to fetcher
            match fetcher::fetch_metadata(&self.app, &url, temp_cookie_path.as_ref()).await {
                Ok(metadata) => {
                    match store::save_metadata(&db, metadata).await {
                        Ok(post_id) => {
                            tracing::info!(
                                "Metadata saved for task {}, linked to post {}",
                                task_id,
                                post_id
                            );
                            // Link post_id to task
                            let _ = download_task::Entity::update(download_task::ActiveModel {
                                id: Set(task_id.clone()),
                                post_id: Set(Some(post_id)),
                                ..Default::default()
                            })
                            .exec(&db)
                            .await
                            .map_err(|e| tracing::error!("Failed to update task post_id: {}", e));
                        }
                        Err(e) => {
                            tracing::error!("Failed to save metadata for task {}: {}", task_id, e);
                            // Verify cleanup
                            if let Some(path) = &temp_cookie_path {
                                let _ = cookie_manager.cleanup_temp_file(path).await;
                            }
                            return Err(DownloadError::Failed {
                                message: format!("Metadata save error: {}", e),
                                total_bytes: None,
                                downloaded_bytes: 0,
                                filename: None,
                            });
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to fetch metadata for task {}: {}", task_id, e);
                    // Verify cleanup
                    if let Some(path) = &temp_cookie_path {
                        let _ = cookie_manager.cleanup_temp_file(path).await;
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

        let mut cmd = Command::new(binary_path);

        let qjs_path =
            get_binary_path(&self.app, SidecarBinary::Qjs).map_err(|e| DownloadError::Failed {
                message: format!("QuickJS not found: {}", e),
                total_bytes: None,
                downloaded_bytes: 0,
                filename: None,
            })?;
        let qjs_arg = format!("quickjs:{}", qjs_path.to_string_lossy());

        // --newline is CRITICAL for line-by-line progress parsing
        // -c enables resume of partial downloads (for pause/resume support)
        cmd.arg("--newline")
            .arg("--no-playlist")
            .arg("-c")
            .arg("-P")
            .arg(&output_dir)
            .arg("--output")
            .arg("%(title)s.%(ext)s")
            // Rate limit for debugging/stability (5MB/s)
            .arg("--limit-rate")
            .arg("5M")
            .arg("--js-runtimes")
            .arg(qjs_arg);

        // Inject cookies if available
        if let Some(ref cookie_path) = temp_cookie_path {
            cmd.arg("--cookies").arg(cookie_path);
        }

        // Apply format selection if specified
        if let Some(ref fmt) = format_selection {
            cmd.arg("-f").arg(fmt);
        }

        cmd.arg(&url);

        // Windows: hide console window
        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| DownloadError::Failed {
            message: format!("Failed to spawn yt-dlp: {}", e),
            total_bytes: None,
            downloaded_bytes: 0,
            filename: None,
        })?;

        let stdout = child.stdout.take().ok_or(DownloadError::Failed {
            message: "Failed to open stdout".to_string(),
            total_bytes: None,
            downloaded_bytes: 0,
            filename: None,
        })?;
        let stderr = child.stderr.take().ok_or(DownloadError::Failed {
            message: "Failed to open stderr".to_string(),
            total_bytes: None,
            downloaded_bytes: 0,
            filename: None,
        })?;

        // --- Stderr capture task ---
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

        // --- Stdout progress reading with cancellation ---
        let mut reader = BufReader::new(stdout);
        let mut buf = Vec::new();
        let parser = Parser::new();
        let mut last_emit = Instant::now();

        // Create explicit variables to track stats across the loop
        let mut final_total_bytes = None;
        let mut final_downloaded_bytes = 0;
        let mut final_filename = None;
        // The merged filename is the ACTUAL output file after yt-dlp merges
        // video+audio streams. It differs from the intermediate Destination filenames.
        let mut merged_filename: Option<String> = None;
        let mut was_killed = false;

        let read_result: Result<(), DownloadError> = loop {
            // Check cancellation BEFORE entering select! to guarantee
            // cancel wins even if read_until already returned data.
            if cancel_token.is_cancelled() {
                tracing::info!("Download cancelled for task: {}", task_id);
                kill_process_tree(&mut child).await;
                let _ = child.wait().await; // Reap process to avoid zombies on Windows
                was_killed = true;
                break Err(DownloadError::Cancelled {
                    total_bytes: final_total_bytes,
                    downloaded_bytes: final_downloaded_bytes,
                    filename: final_filename.clone(),
                });
            }

            tokio::select! {
                biased; // Prefer cancellation over read_until

                // Cancellation branch
                _ = cancel_token.cancelled() => {
                    tracing::info!("Download cancelled for task: {}", task_id);
                    kill_process_tree(&mut child).await;
                    let _ = child.wait().await;
                    was_killed = true;
                    break Err(DownloadError::Cancelled {
                        total_bytes: final_total_bytes,
                        downloaded_bytes: final_downloaded_bytes,
                        filename: final_filename.clone(),
                    });
                }
                // Read next line (using read_until for robust UTF-8 handling)
                result = reader.read_until(b'\n', &mut buf) => {
                    match result {
                        Ok(0) => break Ok(()), // EOF
                        Ok(_) => {
                            let line = String::from_utf8_lossy(&buf);
                            // Import ParseResult
                            use super::parser::ParseResult;

                            match parser.parse_line(&line) {
                                ParseResult::Progress(progress) => {
                                    // Update final stats (accumulate if present)
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
                                            task_id: task_id.clone(),
                                            progress: progress.progress,
                                            speed: progress.speed.clone().unwrap_or_default(),
                                            eta: progress.eta.clone().unwrap_or_default(),
                                            downloaded_bytes: final_downloaded_bytes,
                                            total_bytes: final_total_bytes, // Use accumulated value
                                        };

                                        let _ = self.app.emit("download-progress", &payload);

                                        // Persist progress to DB (throttled)
                                        // Use update_many to avoid implicit SELECT after UPDATE
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

                                        // Only update downloaded_bytes if we have a valid value
                                        // (parser returns None if total_bytes is unknown, but we might track it manually)
                                        if progress.downloaded_bytes.is_some() {
                                            update = update.col_expr(
                                                download_task::Column::DownloadedBytes,
                                                sea_orm::sea_query::Expr::value(progress.downloaded_bytes),
                                            );
                                        }

                                        // Only update total_bytes if we have a new value.
                                        // This prevents overwriting a known size with NULL if yt-dlp sends an update without size.
                                        if let Some(total) = progress.total_bytes {
                                            update = update.col_expr(
                                                download_task::Column::TotalBytes,
                                                sea_orm::sea_query::Expr::value(total),
                                            );
                                        }

                                        let _ = update
                                            .filter(download_task::Column::Id.eq(task_id.clone()))
                                            .exec(&db)
                                            .await;
                                    }
                                }
                                ParseResult::Filename(name) => {
                                    final_filename = Some(name);
                                }
                                ParseResult::MergedFilename(name) => {
                                    tracing::info!("Merged output file: {}", name);
                                    merged_filename = Some(name);
                                }
                                ParseResult::Ignore => {}
                            }
                            buf.clear();
                        }
                        Err(e) => break Err(DownloadError::Failed {
                            message: format!("Failed to read stdout: {}", e),
                            total_bytes: final_total_bytes,
                            downloaded_bytes: final_downloaded_bytes,
                            filename: final_filename.clone(),
                        }),
                    }
                }
            }
        };

        // Wait for stderr task to finish
        let _ = stderr_handle.await;

        // If the read loop errored (cancelled or IO error), propagate it
        read_result?;

        // Wait for process to finish (skip if already killed & waited)
        let status = if was_killed {
            // Process already reaped in the kill path
            return Err(DownloadError::Cancelled {
                total_bytes: final_total_bytes,
                downloaded_bytes: final_downloaded_bytes,
                filename: final_filename.clone(),
            });
        } else {
            child.wait().await.map_err(|e| DownloadError::Failed {
                message: e.to_string(),
                total_bytes: final_total_bytes,
                downloaded_bytes: final_downloaded_bytes,
                filename: final_filename.clone(),
            })?
        };

        let result = if status.success() {
            // Prefer merged filename (final output) over intermediate destination filename.
            let result_filename = merged_filename.or(final_filename);

            // Read ACTUAL file size from disk — yt-dlp progress only reports
            // per-stream sizes, so for multi-stream downloads (video+audio)
            // the parsed total_bytes is just the last stream's size.
            let actual_file_size = result_filename.as_ref().and_then(|fname| {
                let file_path = output_dir.join(fname);
                tracing::info!("Reading file size from: {:?}", file_path);
                match std::fs::metadata(&file_path) {
                    Ok(m) => {
                        let size = m.len();
                        tracing::info!("Actual file size on disk: {} bytes", size);
                        Some(size)
                    }
                    Err(e) => {
                        tracing::warn!("Could not read file metadata: {}", e);
                        None
                    }
                }
            });

            // Use disk size if available, never fall back to parsed progress bytes
            let total = actual_file_size.unwrap_or(0);

            Ok(DownloadResult {
                total_bytes: actual_file_size,
                downloaded_bytes: total,
                filename: result_filename,
            })
        } else {
            // Build error message from stderr
            let stderr_output = stderr_lines.lock().await;
            let error_detail = if stderr_output.is_empty() {
                format!("yt-dlp exited with status: {}", status)
            } else {
                // Take last 5 lines for a concise error
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
                filename: final_filename.clone(),
            })
        };

        // Cleanup temp cookie file
        if let Some(path) = temp_cookie_path {
            let _ = cookie_manager.cleanup_temp_file(&path).await;
        }

        result
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_robust_utf8_conversion() {
        // A sequence of bytes that is NOT valid UTF-8
        let invalid_utf8 = vec![0, 159, 146, 150];

        // This is what we now do in worker.rs
        let lossy_string = String::from_utf8_lossy(&invalid_utf8);

        // It should NOT panic and should provide some representation
        assert!(!lossy_string.is_empty());
    }

    #[test]
    fn test_mixed_output_parsing() {
        let mixed_bytes = b"normal text \xEF\xBF\xBD damaged text\n";
        let line = String::from_utf8_lossy(mixed_bytes);
        assert!(line.contains("normal text"));
        assert!(line.contains("damaged text"));
    }
}
