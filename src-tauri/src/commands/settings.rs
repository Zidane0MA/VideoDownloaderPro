use crate::entity::setting;
use crate::entity::setting::Entity as Setting;
use crate::AppState;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use std::collections::HashMap;
use tauri::State;
use tokio::sync::watch;

/// Tauri-managed wrapper for the concurrency watch sender.
/// `update_setting` uses this to push live `concurrent_downloads` changes
/// to the scheduler loop without restarting the application.
pub struct ConcurrencyTx(pub watch::Sender<usize>);

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let settings = Setting::find()
        .all(&state.db)
        .await
        .map_err(|e: sea_orm::DbErr| e.to_string())?;

    let mut map = HashMap::new();
    for s in settings {
        map.insert(s.key, s.value);
    }

    Ok(map)
}

/// Upsert a setting by key, and—when the key is `concurrent_downloads`—push
/// the parsed value over the watch channel so the scheduler applies it live.
#[tauri::command]
pub async fn update_setting(
    key: String,
    value: String,
    state: State<'_, AppState>,
    concurrency_tx: State<'_, ConcurrencyTx>,
) -> Result<(), String> {
    // Upsert
    let existing = Setting::find_by_id(&key)
        .one(&state.db)
        .await
        .map_err(|e: sea_orm::DbErr| e.to_string())?;

    if let Some(model) = existing {
        let mut active: setting::ActiveModel = model.into();
        active.value = Set(value.clone());
        active.updated_at = Set(chrono::Utc::now());
        active
            .update(&state.db)
            .await
            .map_err(|e: sea_orm::DbErr| e.to_string())?;
    } else {
        let new_setting = setting::ActiveModel {
            key: Set(key.clone()),
            value: Set(value.clone()),
            updated_at: Set(chrono::Utc::now()),
        };
        new_setting
            .insert(&state.db)
            .await
            .map_err(|e: sea_orm::DbErr| e.to_string())?;
    }

    // Live-reload: propagate concurrency changes to the scheduler immediately.
    if key == "concurrent_downloads" {
        match value.parse::<usize>() {
            Ok(n) => {
                let n = n.clamp(1, 10);
                // Err only if all receivers have been dropped (app shutting down) — safe to ignore.
                let _ = concurrency_tx.0.send(n);
                tracing::info!("Concurrency limit updated live to {}", n);
            }
            Err(_) => {
                tracing::warn!("Invalid concurrent_downloads value '{}' — ignoring", value);
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn select_download_path() -> Result<Option<String>, String> {
    // This command triggers the native file picker manually if needed.
    // NOTE: In tauri v2, using the dialog plugin from JS is usually preferred,
    // but the backend command can be useful if the UI logic requires it.
    // For simplicity, we will let the frontend call `open()` from `@tauri-apps/plugin-dialog`.

    // We'll return Ok(None) to indicate the frontend should handle it,
    // or just leave this as a stub documented in the rust side.
    Ok(None)
}
