use crate::entity::setting;
use crate::entity::setting::Entity as Setting;
use crate::AppState;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use std::collections::HashMap;
use tauri::State;

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

#[tauri::command]
pub async fn update_setting(
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Check if it exists
    let existing = Setting::find_by_id(&key)
        .one(&state.db)
        .await
        .map_err(|e: sea_orm::DbErr| e.to_string())?;

    if let Some(model) = existing {
        // Update
        let mut active: setting::ActiveModel = model.into();
        active.value = Set(value);
        active.updated_at = Set(chrono::Utc::now());
        active
            .update(&state.db)
            .await
            .map_err(|e: sea_orm::DbErr| e.to_string())?;
    } else {
        let new_setting = setting::ActiveModel {
            key: Set(key.clone()),
            value: Set(value),
            updated_at: Set(chrono::Utc::now()),
        };
        new_setting
            .insert(&state.db)
            .await
            .map_err(|e: sea_orm::DbErr| e.to_string())?;
    }

    if key == "concurrent_downloads" {
        tracing::info!("Concurrent downloads updated. Requires app restart to apply changes to the background worker pool.");
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
