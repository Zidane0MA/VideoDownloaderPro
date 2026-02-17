use std::path::PathBuf;
use std::process::Stdio;
use tauri::{AppHandle, Manager, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use crate::sidecar::{get_binary_path, types::SidecarBinary};
use super::parser::Parser;
use serde::Serialize;

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

    pub async fn execute_download(&self, task_id: String, url: String, _output_path: PathBuf) -> Result<(), String> {
        let binary_path = get_binary_path(&self.app, SidecarBinary::YtDlp)
            .map_err(|e| e.to_string())?;

        // TODO: Add proper args for output template, format selection, etc.
        // For now, testing basic progress parsing.
        let mut cmd = Command::new(binary_path);
        
        // Ensure we force IPv4 if needed, or other default flags
        // --newline is CRITICAL for line-by-line parsing if using stdout
        cmd.arg("--newline") 
           .arg("--no-playlist")
           .arg(&url);

        // Windows cleanup
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| e.to_string())?;

        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to open stderr")?;

        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let parser = Parser::new();

        // We can also read stderr in a separate task if we want to log errors
        let _stderr_reader = BufReader::new(stderr);

        while reader.read_line(&mut line).await.map_err(|e| e.to_string())? > 0 {
            if let Some(progress) = parser.parse_line(&line) {
                // Emit event
                let payload = DownloadProgressPayload {
                    task_id: task_id.clone(),
                    progress: progress.progress,
                    speed: progress.speed.clone().unwrap_or_default(),
                    eta: progress.eta.clone().unwrap_or_default(),
                    downloaded_bytes: progress.downloaded_bytes.unwrap_or(0),
                    total_bytes: progress.total_bytes,
                };
                
                let _ = self.app.emit("download-progress", &payload);
            }
            line.clear();
        }

        let status = child.wait().await.map_err(|e| e.to_string())?;
        
        if status.success() {
            Ok(())
        } else {
            Err(format!("Download failed with status: {}", status))
        }
    }
}
