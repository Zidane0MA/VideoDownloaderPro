mod commands;
pub mod db;
pub mod download;
mod entity;
pub mod metadata;
mod migration;
pub mod queue;
pub mod sidecar;

use sea_orm::DatabaseConnection;
use tauri::Manager;
use tracing_subscriber::{fmt, EnvFilter};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// The database connection, stored as Tauri managed state.
pub struct AppState {
    pub db: DatabaseConnection,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .with_thread_names(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    tracing::info!("Video Downloader Pro starting...");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            // Initialize the database synchronously within the async runtime
            let db = tauri::async_runtime::block_on(async { db::init_db(app_data_dir).await })
                .expect("Failed to initialize database");

            tracing::info!("Database initialized successfully");

            // Initialize sidecars (copy from bundle to app_data if needed)
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                if let Err(e) = sidecar::setup_sidecars(&handle).await {
                    tracing::error!("Failed to setup sidecars: {}", e);
                    // We don't panic here to allow the app to start, but sidecars won't work
                }
            });

            app.manage(AppState { db });

            // Initialize Download Queue (Max 3 concurrent downloads)
            let queue = queue::DownloadQueue::new(app.handle().clone(), 3);
            app.manage(queue.clone());

            // Start scheduler in background
            tauri::async_runtime::spawn(async move {
                queue.start_scheduler().await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::sidecar::get_sidecar_status,
            commands::sidecar::get_ytdlp_version,
            commands::sidecar::update_ytdlp,
            commands::download::create_download_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
