pub mod auth;
pub mod background;
mod commands;
pub mod db;
pub mod download;
mod entity;
pub mod metadata;
pub mod migration;
pub mod platform;
pub mod queue;
pub mod sidecar;

use sea_orm::{DatabaseConnection, EntityTrait};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
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
    // Initialize tracing with file appender
    // Store logs in ./logs for easy access during dev, or standard location in prod if needed.
    // For this context, "./logs" relative to CWD is best.
    let file_appender = tracing_appender::rolling::daily("logs", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_names(true)
                .with_writer(std::io::stdout),
        )
        .with(
            fmt::layer()
                .with_ansi(false)
                .with_writer(non_blocking)
                .with_target(true),
        )
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    tracing::info!("Video Downloader Pro starting...");
    tracing::info!("Logs are being written to ./logs");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            // Initialize the database synchronously within the async runtime
            let db =
                tauri::async_runtime::block_on(async { db::init_db(app_data_dir.clone()).await })
                    .expect("Failed to initialize database");

            tracing::info!("Database initialized successfully");

            // Start Trash background cleaner
            background::trash_cleaner::start_trash_cleaner(
                app.handle(),
                std::sync::Arc::new(db.clone()),
            );

            // Initialize sidecars (copy from bundle to app_data if needed)
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                if let Err(e) = sidecar::setup_sidecars(&handle).await {
                    tracing::error!("Failed to setup sidecars: {}", e);
                    // We don't panic here to allow the app to start, but sidecars won't work
                }
            });

            app.manage(AppState { db: db.clone() });

            // Initialize CookieManager
            let cookie_manager = std::sync::Arc::new(auth::cookie_manager::CookieManager::new(
                db.clone().into(),
                app_data_dir.clone(),
            ));
            let _ = cookie_manager.init(); // Fire and forget init (create temp dir)
            app.manage(cookie_manager);

            // Read `concurrent_downloads` from the DB; default to 3 if absent or invalid.
            let initial_concurrency: usize = tauri::async_runtime::block_on(async {
                use crate::entity::setting::Entity as Setting;
                Setting::find_by_id("concurrent_downloads")
                    .one(&db)
                    .await
                    .ok()
                    .flatten()
                    .and_then(|s| s.value.parse::<usize>().ok())
                    .map(|n| n.clamp(1, 10))
                    .unwrap_or(3)
            });
            tracing::info!("Queue concurrency on startup: {}", initial_concurrency);

            // Create the watch channel that allows live concurrency updates.
            let (concurrency_tx, concurrency_rx) = tokio::sync::watch::channel(initial_concurrency);

            // Register sender as Tauri state so `update_setting` can push to it.
            use commands::settings::ConcurrencyTx;
            app.manage(ConcurrencyTx(concurrency_tx));

            // Initialize Download Queue with DB-sourced concurrency limit.
            let queue = queue::DownloadQueue::new(app.handle().clone(), concurrency_rx);
            app.manage(queue.clone());

            // Start scheduler in background
            tauri::async_runtime::spawn(async move {
                queue.start_scheduler().await;
            });

            // --- System Tray Setup ---
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::with_id("tray")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                if window.label() == "main" {
                    window.hide().unwrap();
                    api.prevent_close();
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::sidecar::get_sidecar_status,
            commands::sidecar::get_ytdlp_version,
            commands::sidecar::update_ytdlp,
            commands::download::create_download_task,
            commands::download::cancel_download_task,
            commands::download::retry_download_task,
            commands::download::get_queue_status,
            commands::download::pause_download_task,
            commands::download::resume_download_task,
            commands::download::pause_queue,
            commands::download::resume_queue,
            commands::download::fetch_metadata_command,
            commands::download::clear_download_history,
            commands::download::retry_all_failed,
            commands::auth::get_auth_status,
            commands::auth::update_session,
            commands::auth::delete_session,
            commands::auth::import_from_browser,
            commands::auth::open_login_window,
            commands::auth::verify_session_status,
            commands::auth::verify_all_sessions,
            commands::settings::get_settings,
            commands::settings::update_setting,
            commands::settings::select_download_path,
            commands::wall::get_posts,
            commands::wall::delete_post,
            commands::wall::reveal_in_explorer,
            commands::wall::restore_post,
            commands::wall::get_trash_posts,
            commands::wall::empty_trash_command,
            commands::sources::get_sources_command,
            commands::sources::delete_source_command,
            commands::sources::add_source_command,
            commands::sources::update_source_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
