use crate::auth::cookie_manager::CookieManager;
use crate::entity::platform_session;
use sea_orm::{DatabaseConnection, EntityTrait};
use std::sync::Arc;
use tauri::{Emitter, Manager, State, Window};

#[tauri::command]
pub async fn get_auth_status(
    db: State<'_, Arc<DatabaseConnection>>,
) -> Result<Vec<platform_session::Model>, String> {
    platform_session::Entity::find()
        .all(db.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_session(
    window: Window,
    cookie_manager: State<'_, Arc<CookieManager>>,
    platform_id: String,
    cookies_str: String,
    method: String,
) -> Result<(), String> {
    cookie_manager
        .set_session(platform_id.clone(), cookies_str, method)
        .await
        .map_err(|e| e.to_string())?;

    // Emit event to update UI
    window
        .emit("session-status-changed", &platform_id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_session(
    window: Window,
    cookie_manager: State<'_, Arc<CookieManager>>,
    platform_id: String,
) -> Result<(), String> {
    cookie_manager
        .delete_session(&platform_id)
        .await
        .map_err(|e| e.to_string())?;

    // Emit event to update UI
    window
        .emit("session-status-changed", &platform_id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn import_from_browser(
    platform_id: String,
    browser: String,
    window: Window,
    app_handle: tauri::AppHandle,
    cookie_manager: State<'_, Arc<CookieManager>>,
) -> Result<(), String> {
    let url = match platform_id.as_str() {
        "youtube" => "https://www.youtube.com",
        "tiktok" => "https://www.tiktok.com",
        "instagram" => "https://www.instagram.com",
        "x" => "https://x.com",
        _ => return Err("Unsupported platform".into()),
    };

    let ytdlp_path =
        crate::sidecar::get_binary_path(&app_handle, crate::sidecar::types::SidecarBinary::YtDlp)
            .map_err(|e| e.to_string())?;

    let temp_cookie_path = std::env::temp_dir().join(format!(
        "cookies_{}_{}.txt",
        platform_id,
        uuid::Uuid::new_v4()
    ));

    // Determine arguments for yt-dlp based on whether it's a standard browser or our WebView
    // Syntax for yt-dlp: --cookies-from-browser BROWSER[:PROFILE]
    let (browser_arg, temp_profile_dir) = if browser == "webview" {
        // Points to our own WebView2 data directory (on Windows: AppData/Local/.../EBWebView)
        let app_data = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| e.to_string())?;
        let webview_dir = app_data.join("EBWebView");

        // Find the Cookies file. structure might be Default/Network/Cookies or Default/Cookies
        let possible_cookie_paths = [
            webview_dir.join("Default").join("Network").join("Cookies"),
            webview_dir.join("Default").join("Cookies"),
        ];

        let cookie_path = possible_cookie_paths
            .iter()
            .find(|p| p.exists())
            .ok_or_else(|| {
                format!(
                    "WebView cookie file not found in {:?}. Have you logged in?",
                    webview_dir
                )
            })?;

        // Create a temporary profile directory to avoid locking issues
        let temp_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        let temp_network_dir = temp_dir.join("Default").join("Network");
        tokio::fs::create_dir_all(&temp_network_dir)
            .await
            .map_err(|e| e.to_string())?;

        let temp_cookie_path = temp_network_dir.join("Cookies");

        tracing::info!(
            "Shadow copying cookies from {:?} to {:?}",
            cookie_path,
            temp_cookie_path
        );

        // Copy the file. Note: This might still fail if locked EXCLUSIVELY, but usually works with Shared Read.
        tokio::fs::copy(cookie_path, &temp_cookie_path)
            .await
            .map_err(|e| format!("Failed to copy locked cookie file: {}", e))?;

        // Attempt to copy -wal and -shm files if they exist (SQLite Write-Ahead Log)
        // This is CRITICAL because modern WebView2/Chromium keeps latest session data in the WAL file
        // until a checkpoint occurs. Without this, we might read an old login state.
        let file_name = cookie_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let parent = cookie_path.parent().unwrap(); // We know it has a parent

        // Construct potential source paths
        let wal_source = parent.join(format!("{}-wal", file_name));
        let shm_source = parent.join(format!("{}-shm", file_name));

        // Construct destination paths (must match the source naming convention relative to dst file)
        let dst_parent = temp_cookie_path.parent().unwrap();
        let dst_file_name = temp_cookie_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let wal_dest = dst_parent.join(format!("{}-wal", dst_file_name));
        let shm_dest = dst_parent.join(format!("{}-shm", dst_file_name));

        if wal_source.exists() {
            if let Err(e) = tokio::fs::copy(&wal_source, &wal_dest).await {
                tracing::warn!(
                    "Failed to copy WAL file (might be locked or unnecessary): {}",
                    e
                );
            } else {
                tracing::info!("Successfully copied WAL file");
            }
        }

        if shm_source.exists() {
            if let Err(e) = tokio::fs::copy(&shm_source, &shm_dest).await {
                tracing::warn!(
                    "Failed to copy SHM file (might be locked or unnecessary): {}",
                    e
                );
            } else {
                tracing::info!("Successfully copied SHM file");
            }
        }

        // Return "chromium:<temp_dir>" and the temp dir path for cleanup
        (
            format!("chromium:{}", temp_dir.to_string_lossy()),
            Some(temp_dir),
        )
    } else {
        (browser.clone(), None)
    };

    // Command: yt-dlp --cookies-from-browser <ARG> --cookies <temp_path> --skip-download <url>
    tracing::info!("Executing yt-dlp import. Browser arg: {}", browser_arg);
    tracing::info!("Temp cookie path: {:?}", temp_cookie_path);

    let output = tokio::process::Command::new(ytdlp_path)
        .arg("--cookies-from-browser")
        .arg(&browser_arg)
        .arg("--cookies")
        .arg(&temp_cookie_path)
        .arg("--skip-download")
        .arg("--verbose") // Add verbose for debugging
        .arg(url)
        .output()
        .await;

    // Cleanup shadow profile if created
    if let Some(path) = temp_profile_dir {
        let _ = tokio::fs::remove_dir_all(path).await;
    }

    let output = output.map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Clean up temp file
        if temp_cookie_path.exists() {
            let _ = tokio::fs::remove_file(&temp_cookie_path).await;
        }

        // Detailed error for common case (browser open)
        if stderr.contains("permission denied")
            || stderr.contains("Device or resource busy")
            || stderr.contains("open")
        {
            return Err("Please close the browser/webview logic window and try again.".into());
        }

        tracing::error!("yt-dlp import failed. Stderr: {}", stderr);
        return Err(format!("Failed to import cookies: {}", stderr));
    }

    if !temp_cookie_path.exists() {
        return Err("yt-dlp did not create a cookie file. Maybe no cookies found?".into());
    }

    let cookies_content = tokio::fs::read_to_string(&temp_cookie_path)
        .await
        .map_err(|e| format!("Failed to read temp cookie file: {}", e))?;

    // Cleanup immediately
    let _ = tokio::fs::remove_file(&temp_cookie_path).await;

    if cookies_content.trim().is_empty() {
        return Err("Imported cookie file was empty.".into());
    }

    cookie_manager
        .set_session(
            platform_id.clone(),
            cookies_content,
            format!("browser_import:{}", browser),
        )
        .await
        .map_err(|e| e.to_string())?;

    window
        .emit("session-status-changed", &platform_id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn open_login_window(
    app_handle: tauri::AppHandle,
    platform_id: String,
) -> Result<(), String> {
    let url = match platform_id.as_str() {
        "youtube" => "https://accounts.google.com/ServiceLogin?service=youtube",
        "tiktok" => "https://www.tiktok.com/login",
        "instagram" => "https://www.instagram.com/accounts/login/",
        "x" => "https://x.com/i/flow/login",
        _ => return Err("Unsupported platform".into()),
    };

    let label = format!("auth_{}", platform_id);

    if let Some(w) = app_handle.get_webview_window(&label) {
        let _ = w.set_focus();
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(
        &app_handle,
        &label,
        tauri::WebviewUrl::External(url.parse().unwrap()),
    )
    .title(format!("Login to {}", platform_id))
    .inner_size(480.0, 720.0)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}
