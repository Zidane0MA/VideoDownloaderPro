use super::parser::Parser;
use crate::entity::download_task;
use crate::sidecar::{get_binary_path, types::SidecarBinary};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::Serialize;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

/// Minimum interval between progress emissions to avoid flooding the IPC bridge.
const PROGRESS_THROTTLE: Duration = Duration::from_millis(500);

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
    ) -> Result<(Option<u64>, u64), String> {
        let binary_path =
            get_binary_path(&self.app, SidecarBinary::YtDlp).map_err(|e| e.to_string())?;

        let mut cmd = Command::new(binary_path);

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
            .arg("5M");

        // Apply format selection if specified
        if let Some(ref fmt) = format_selection {
            cmd.arg("-f").arg(fmt);
        }

        cmd.arg(&url);

        // Windows: hide console window
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn yt-dlp: {}", e))?;

        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to open stderr")?;

        // --- Stderr capture task ---
        let stderr_lines = std::sync::Arc::new(Mutex::new(Vec::<String>::new()));
        let stderr_lines_clone = stderr_lines.clone();
        let stderr_handle = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 {
                    break;
                }
                let trimmed = line.trim().to_string();
                if !trimmed.is_empty() {
                    tracing::warn!(target: "yt-dlp:stderr", "{}", trimmed);
                    stderr_lines_clone.lock().await.push(trimmed);
                }
                line.clear();
            }
        });

        // --- Stdout progress reading with cancellation ---
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let parser = Parser::new();
        let mut last_emit = Instant::now();

        // Create explicit variables to track stats across the loop
        let mut final_total_bytes = None;
        let mut final_downloaded_bytes = 0;
        let mut was_killed = false;

        let read_result: Result<(), String> = loop {
            // Check cancellation BEFORE entering select! to guarantee
            // cancel wins even if read_line already returned data.
            if cancel_token.is_cancelled() {
                tracing::info!("Download cancelled for task: {}", task_id);
                let _ = child.kill().await;
                let _ = child.wait().await; // Reap process to avoid zombies on Windows
                was_killed = true;
                break Err("Download cancelled by user".to_string());
            }

            tokio::select! {
                biased; // Prefer cancellation over read_line

                // Cancellation branch
                _ = cancel_token.cancelled() => {
                    tracing::info!("Download cancelled for task: {}", task_id);
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                    was_killed = true;
                    break Err("Download cancelled by user".to_string());
                }
                // Read next line
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => break Ok(()), // EOF
                        Ok(_) => {
                            if let Some(progress) = parser.parse_line(&line) {
                                // Update final stats
                                final_total_bytes = progress.total_bytes;
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
                                        downloaded_bytes: progress.downloaded_bytes.unwrap_or(0),
                                        total_bytes: progress.total_bytes,
                                    };

                                    let _ = self.app.emit("download-progress", &payload);

                                    // Persist progress to DB (throttled)
                                    // Use update_many to avoid implicit SELECT after UPDATE
                                    let _ = download_task::Entity::update_many()
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
                                        )
                                        .col_expr(
                                            download_task::Column::DownloadedBytes,
                                            sea_orm::sea_query::Expr::value(progress.downloaded_bytes),
                                        )
                                        .col_expr(
                                            download_task::Column::TotalBytes,
                                            sea_orm::sea_query::Expr::value(progress.total_bytes),
                                        )
                                        .filter(download_task::Column::Id.eq(task_id.clone()))
                                        .exec(&db)
                                        .await;
                                }
                            }
                            line.clear();
                        }
                        Err(e) => break Err(format!("Failed to read stdout: {}", e)),
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
            return Ok((final_total_bytes, final_downloaded_bytes));
        } else {
            child.wait().await.map_err(|e| e.to_string())?
        };

        if status.success() {
            Ok((final_total_bytes, final_downloaded_bytes))
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
            Err(error_detail)
        }
    }
}
