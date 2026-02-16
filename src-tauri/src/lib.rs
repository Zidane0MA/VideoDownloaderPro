mod commands;
pub mod db;
mod entity;
mod migration;
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

            app.manage(AppState { db });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::sidecar::get_sidecar_status,
            commands::sidecar::get_sidecar_version,
            commands::sidecar::update_sidecar,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
