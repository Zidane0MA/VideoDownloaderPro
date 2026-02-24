---
name: yt-dlp-integration
description: "Patterns for integrating yt-dlp as a Tauri Sidecar: spawning processes, parsing JSON metadata, streaming stdout progress, handling errors (403, 429, geo-block), cookie auth flows, and playlist extraction. Use when implementing download commands, metadata fetching, or yt-dlp process management in VideoDownloaderPro."
---

# yt-dlp Integration Patterns for Tauri

Production patterns for running `yt-dlp` as an external binary (Sidecar) from a Tauri v2 Rust backend, including metadata extraction, progress streaming, error classification, and cookie authentication.

## When to Use This Skill

- Spawning `yt-dlp` from Rust as a child process
- Parsing `--dump-json` output into Rust structs
- Reading real-time download progress from stdout
- Handling platform-specific errors (403, 429, geo-block, age-gate)
- Implementing cookie/auth retry flows
- Managing playlists and batch downloads

## Core Architecture

```
Frontend (invoke) → Rust Command → spawn yt-dlp.exe → stdout/stderr → parse → emit events → Frontend
```

The `yt-dlp` binary lives at `app_data/binaries/yt-dlp.exe` (runtime copy, auto-updatable). Rust spawns it via `std::process::Command` (sync) or `tokio::process::Command` (async), reads its output line-by-line, and emits Tauri events to the frontend.

---

## Pattern 1: Spawning yt-dlp as a Child Process

```rust
use std::path::PathBuf;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Path to the runtime yt-dlp binary (inside app_data).
fn ytdlp_path(app_handle: &tauri::AppHandle) -> PathBuf {
    app_handle
        .path()
        .app_data_dir()
        .expect("app_data_dir not found")
        .join("binaries")
        .join("yt-dlp.exe")
}

/// Spawn yt-dlp with arbitrary args, return stdout/stderr line readers.
async fn spawn_ytdlp(
    binary: &PathBuf,
    args: &[&str],
) -> Result<tokio::process::Child, std::io::Error> {
    Command::new(binary)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .creation_flags(0x08000000) // CREATE_NO_WINDOW on Windows
        .spawn()
}
```

> **IMPORTANT**: Always use `.creation_flags(0x08000000)` on Windows to prevent a console window from flashing on every yt-dlp invocation. This constant is `CREATE_NO_WINDOW`.

---

## Pattern 2: Metadata Extraction (`--dump-json`)

```rust
use serde::Deserialize;

/// Minimal subset of yt-dlp's JSON output. Extend as needed.
#[derive(Debug, Deserialize)]
pub struct YtdlpMetadata {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub uploader: Option<String>,
    pub uploader_id: Option<String>,
    pub uploader_url: Option<String>,
    pub channel: Option<String>,
    pub channel_id: Option<String>,
    pub channel_url: Option<String>,
    pub duration: Option<f64>,            // seconds
    pub thumbnail: Option<String>,        // URL
    pub upload_date: Option<String>,      // "YYYYMMDD"
    pub filesize_approx: Option<u64>,
    pub ext: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub formats: Option<Vec<YtdlpFormat>>,
    // Playlist fields (present when --dump-json on playlist item)
    pub playlist_title: Option<String>,
    pub playlist_index: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct YtdlpFormat {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub filesize: Option<u64>,
    pub filesize_approx: Option<u64>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub fps: Option<f64>,
    #[serde(default)]
    pub format_note: Option<String>,
}

/// Fetch metadata for a single URL.
async fn fetch_metadata(
    binary: &PathBuf,
    url: &str,
    cookies_browser: Option<&str>,
) -> Result<YtdlpMetadata, AppError> {
    let mut args = vec!["--dump-json", "--no-download", "--no-warnings"];

    // Optionally attach cookies
    if let Some(browser) = cookies_browser {
        args.extend(&["--cookies-from-browser", browser]);
    }
    args.push(url);

    let child = spawn_ytdlp(binary, &args).await?;
    let output = child.wait_with_output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(classify_ytdlp_error(&stderr));
    }

    let metadata: YtdlpMetadata = serde_json::from_slice(&output.stdout)
        .map_err(|e| AppError::ParseError(format!("Failed to parse yt-dlp JSON: {}", e)))?;

    Ok(metadata)
}
```

### Key notes
- `--dump-json` outputs a single JSON object per video to stdout.
- For playlists, use `--dump-json --flat-playlist` to get one JSON line per item (much faster, no full metadata).
- Always use `--no-warnings` to keep stderr clean for error detection.

---

## Pattern 3: Download Progress Streaming

yt-dlp outputs progress lines to stdout in this format:
```
[download]   5.2% of  150.00MiB at  2.50MiB/s ETA 00:57
[download]  10.4% of  150.00MiB at  2.51MiB/s ETA 00:53
[download] 100% of  150.00MiB in 00:59
```

```rust
use regex::Regex;
use lazy_static::lazy_static;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub task_id: String,
    pub progress: f64,       // 0.0 to 1.0
    pub speed: String,       // "2.50MiB/s"
    pub eta: String,         // "00:57" or "Unknown"
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}

lazy_static! {
    /// Regex to parse yt-dlp [download] progress lines.
    static ref PROGRESS_RE: Regex = Regex::new(
        r"\[download\]\s+(\d+\.?\d*)% of\s+~?([\d.]+)(\w+)\s+at\s+([\d.]+\w+/s)\s+ETA\s+([\d:]+)"
    ).unwrap();

    /// Regex to detect download completion line.
    static ref COMPLETE_RE: Regex = Regex::new(
        r"\[download\]\s+100% of\s+([\d.]+)(\w+)"
    ).unwrap();
}

fn parse_progress_line(line: &str, task_id: &str) -> Option<DownloadProgress> {
    if let Some(caps) = PROGRESS_RE.captures(line) {
        let percent: f64 = caps[1].parse().ok()?;
        let size_val: f64 = caps[2].parse().ok()?;
        let size_unit = &caps[3];
        let speed = caps[4].to_string();
        let eta = caps[5].to_string();

        let total_bytes = to_bytes(size_val, size_unit);

        return Some(DownloadProgress {
            task_id: task_id.to_string(),
            progress: percent / 100.0,
            speed,
            eta,
            downloaded_bytes: (total_bytes as f64 * (percent / 100.0)) as u64,
            total_bytes: Some(total_bytes),
        });
    }
    None
}

fn to_bytes(value: f64, unit: &str) -> u64 {
    let multiplier = match unit {
        "KiB" => 1024.0,
        "MiB" => 1024.0 * 1024.0,
        "GiB" => 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };
    (value * multiplier) as u64
}
```

### Streaming loop with throttled event emission

```rust
use tauri::Emitter;
use std::time::{Duration, Instant};

async fn stream_download_progress(
    child: &mut tokio::process::Child,
    task_id: &str,
    app: &tauri::AppHandle,
) {
    let stdout = child.stdout.take().expect("stdout not captured");
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();
    let mut last_emit = Instant::now();

    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(progress) = parse_progress_line(&line, task_id) {
            // Throttle events to ~500ms intervals
            if last_emit.elapsed() >= Duration::from_millis(500) || progress.progress >= 1.0 {
                let _ = app.emit("download-progress", &progress);
                last_emit = Instant::now();
            }
        }
    }
}
```

---

## Pattern 4: Error Classification from stderr

```rust
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum YtdlpError {
    #[error("[NET_001] No internet connection")]
    NoInternet,
    #[error("[NET_003] Connection timeout")]
    Timeout,
    #[error("[PLAT_001] HTTP 403 Forbidden")]
    Forbidden,
    #[error("[PLAT_002] HTTP 429 Rate Limited")]
    RateLimited,
    #[error("[PLAT_003] Content unavailable (deleted/404)")]
    ContentUnavailable,
    #[error("[PLAT_004] Geo-blocked content")]
    GeoBlocked,
    #[error("[PLAT_005] Private content")]
    PrivateContent,
    #[error("[PLAT_006] Live stream in progress")]
    LiveStream,
    #[error("[YT_002] Format not available")]
    FormatUnavailable,
    #[error("[YT_003] yt-dlp outdated for this URL")]
    Outdated,
    #[error("[AUTH_001] Login required")]
    LoginRequired,
    #[error("[AUTH_002] Age-gated content")]
    AgeGated,
    #[error("[AUTH_004] Browser cookie DB locked")]
    CookieDbLocked,
    #[error("[DISK_001] Disk full")]
    DiskFull,
    #[error("[YT_001] Unknown yt-dlp error: {0}")]
    Unknown(String),
}

/// Classify a yt-dlp stderr output into a strongly-typed error.
/// Check patterns in ORDER of specificity (most specific first).
fn classify_ytdlp_error(stderr: &str) -> YtdlpError {
    let s = stderr.to_lowercase();

    // Auth / Access
    if s.contains("sign in to confirm your age") || s.contains("age-gated") {
        return YtdlpError::AgeGated;
    }
    if s.contains("sign in") || s.contains("login required") {
        return YtdlpError::LoginRequired;
    }
    if s.contains("private video") || s.contains("is private") {
        return YtdlpError::PrivateContent;
    }
    if s.contains("cookies could not be loaded") || s.contains("could not open database") {
        return YtdlpError::CookieDbLocked;
    }

    // Platform
    if s.contains("429") || s.contains("too many requests") {
        return YtdlpError::RateLimited;
    }
    if s.contains("403") {
        return YtdlpError::Forbidden;
    }
    if s.contains("not available in your country") || s.contains("geo") {
        return YtdlpError::GeoBlocked;
    }
    if s.contains("is live") {
        return YtdlpError::LiveStream;
    }
    if s.contains("not available") || s.contains("removed") || s.contains("404") {
        return YtdlpError::ContentUnavailable;
    }

    // yt-dlp specific
    if s.contains("format is not available") || s.contains("requested format") {
        return YtdlpError::FormatUnavailable;
    }
    if s.contains("unsupported url") {
        return YtdlpError::Outdated;
    }

    // Disk
    if s.contains("no space left") || s.contains("disk full") {
        return YtdlpError::DiskFull;
    }

    // Network
    if s.contains("timed out") || s.contains("timeout") {
        return YtdlpError::Timeout;
    }
    if s.contains("name resolution") || s.contains("network is unreachable") || s.contains("no internet") {
        return YtdlpError::NoInternet;
    }

    YtdlpError::Unknown(stderr.lines().last().unwrap_or("Unknown error").to_string())
}
```

---

## Pattern 5: Cookie / Auth Retry Flow

```rust
/// Layered auth strategy as defined in project architecture.
/// Layer 1: No auth → Layer 2: --cookies-from-browser → Layer 3: --cookies file
async fn download_with_auth_fallback(
    binary: &PathBuf,
    url: &str,
    settings: &AppSettings,
) -> Result<YtdlpMetadata, YtdlpError> {
    // Layer 1: Try without auth
    match fetch_metadata(binary, url, None).await {
        Ok(meta) => return Ok(meta),
        Err(YtdlpError::AgeGated | YtdlpError::Forbidden | YtdlpError::LoginRequired) => {
            tracing::info!("Auth required, trying cookies-from-browser");
        }
        Err(e) => return Err(e), // Non-auth error, don't retry
    }

    // Layer 2: Try with browser cookies
    if let Some(browser) = &settings.cookie_browser {
        match fetch_metadata(binary, url, Some(browser)).await {
            Ok(meta) => return Ok(meta),
            Err(YtdlpError::CookieDbLocked) => {
                tracing::warn!("Browser cookie DB locked, trying cookies.txt");
            }
            Err(e) => return Err(e),
        }
    }

    // Layer 3: Try with cookies.txt file
    if let Some(cookie_file) = &settings.cookie_file_path {
        let mut args = vec!["--dump-json", "--no-download", "--cookies"];
        let path_str = cookie_file.to_string_lossy();
        args.push(&path_str);
        args.push(url);

        let child = spawn_ytdlp(binary, &args).await?;
        let output = child.wait_with_output().await
            .map_err(|e| YtdlpError::Unknown(e.to_string()))?;

        if output.status.success() {
            let meta: YtdlpMetadata = serde_json::from_slice(&output.stdout)
                .map_err(|e| YtdlpError::Unknown(e.to_string()))?;
            return Ok(meta);
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(classify_ytdlp_error(&stderr));
    }

    Err(YtdlpError::LoginRequired)
}
```

---

## Pattern 6: Full Download Execution with yt-dlp Flags

```rust
/// Build the full yt-dlp argument list for downloading.
fn build_download_args(
    url: &str,
    output_template: &str,
    format_selection: &str,
    settings: &AppSettings,
) -> Vec<String> {
    let mut args = vec![
        // Format
        "-f".to_string(), format_selection.to_string(),
        // Output
        "-o".to_string(), output_template.to_string(),
        // Continue partial downloads
        "-c".to_string(),
        // Thumbnails
        "--write-thumbnail".to_string(),
        "--convert-thumbnails".to_string(), "jpg".to_string(),
        // Metadata (embeds in file)
        "--embed-metadata".to_string(),
        // Rate limiting
        "--sleep-interval".to_string(), settings.sleep_interval.to_string(),
        "--max-sleep-interval".to_string(), settings.max_sleep_interval.to_string(),
        "--sleep-requests".to_string(), settings.sleep_requests.to_string(),
        // Retries (yt-dlp internal retries)
        "--retries".to_string(), settings.retries.to_string(),
        "--fragment-retries".to_string(), settings.fragment_retries.to_string(),
        "--retry-sleep".to_string(), format!("exp:1:{}", settings.retry_sleep_max),
        // Suppress warnings in stderr
        "--no-warnings".to_string(),
        // Progress to stdout (newline mode for line-by-line parsing)
        "--newline".to_string(),
        // The URL
        url.to_string(),
    ];

    // Optional: cookies
    if let Some(browser) = &settings.cookie_browser {
        args.extend(["--cookies-from-browser".to_string(), browser.clone()]);
    }

    args
}
```

> **CRITICAL**: Always pass `--newline` when streaming progress. Without it, yt-dlp uses `\r` (carriage return) to overwrite the same line, which makes line-by-line parsing impossible.

---

## Pattern 7: Playlist Extraction

```rust
/// Extract all items from a playlist without downloading.
async fn extract_playlist_items(
    binary: &PathBuf,
    playlist_url: &str,
) -> Result<Vec<PlaylistItem>, YtdlpError> {
    let args = ["--dump-json", "--flat-playlist", "--no-warnings", playlist_url];
    let child = spawn_ytdlp(binary, &args).await?;
    let output = child.wait_with_output().await
        .map_err(|e| YtdlpError::Unknown(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(classify_ytdlp_error(&stderr));
    }

    // --flat-playlist outputs one JSON object per line
    let stdout = String::from_utf8_lossy(&output.stdout);
    let items: Vec<PlaylistItem> = stdout
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    Ok(items)
}

#[derive(Debug, Deserialize)]
pub struct PlaylistItem {
    pub id: String,
    pub title: Option<String>,
    pub url: String,             // Direct video URL
    pub duration: Option<f64>,
    pub uploader: Option<String>,
}
```

---

## Pattern 8: Killing yt-dlp Processes (Pause / Cancel)

```rust
use sysinfo::{System, Signal};

/// Kill a yt-dlp child process and all its descendants.
/// On Windows, yt-dlp may spawn sub-processes (e.g., ffmpeg for merging).
fn kill_process_tree(pid: u32) -> Result<(), std::io::Error> {
    // On Windows, use taskkill /T (tree kill)
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("taskkill")
            .args(&["/PID", &pid.to_string(), "/T", "/F"])
            .creation_flags(0x08000000)
            .output()?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On Unix, kill the process group
        unsafe { libc::kill(-(pid as i32), libc::SIGTERM); }
    }

    Ok(())
}
```

> On Windows, always use `taskkill /T /F` for tree-kill. A simple `child.kill()` may leave orphaned ffmpeg processes.

---

## Auto-Update Pattern

```rust
/// Check and update yt-dlp using its built-in self-update.
async fn update_ytdlp(binary: &PathBuf) -> Result<UpdateResult, YtdlpError> {
    let before_version = get_ytdlp_version(binary).await?;

    let child = spawn_ytdlp(binary, &["-U"]).await?;
    let output = child.wait_with_output().await
        .map_err(|e| YtdlpError::Unknown(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("is up to date") {
        return Ok(UpdateResult {
            success: true,
            old_version: before_version.clone(),
            new_version: None,
            message: "Already up to date".to_string(),
        });
    }

    let after_version = get_ytdlp_version(binary).await?;

    Ok(UpdateResult {
        success: true,
        old_version: before_version,
        new_version: Some(after_version),
        message: stdout.to_string(),
    })
}

async fn get_ytdlp_version(binary: &PathBuf) -> Result<String, YtdlpError> {
    let child = spawn_ytdlp(binary, &["--version"]).await?;
    let output = child.wait_with_output().await
        .map_err(|e| YtdlpError::Unknown(e.to_string()))?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
```

---

## Best Practices

### Do's
- **Always use `--newline`** when parsing progress output
- **Always use `--no-warnings`** to keep stderr clean for error classification
- **Always use `CREATE_NO_WINDOW`** on Windows to prevent console flashing
- **Tree-kill processes** — yt-dlp may spawn ffmpeg sub-processes
- **Throttle event emission** — 500ms minimum between progress events
- **Use `--dump-json` before downloading** — validate the URL and get metadata first

### Don'ts
- **Don't use `&str` in async Tauri commands** — use `String` (owned types)
- **Don't block the main thread** — always spawn yt-dlp in async tasks
- **Don't ignore stderr** — it contains critical error classification data
- **Don't hardcode binary paths** — always resolve from `app_data_dir()`
- **Don't skip the auth fallback chain** — follow Layer 1 → 2 → 3 order
