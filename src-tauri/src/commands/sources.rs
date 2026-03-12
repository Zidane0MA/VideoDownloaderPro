use chrono::Utc;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Set,
    Value,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Manager, State};

use crate::{
    auth::cookie_manager::CookieManager,
    entity::{creator, download_task, platform_session, post, source},
    metadata::{fetcher, models::YtDlpOutput, store},
    queue::DownloadQueue,
    AppState,
};

#[derive(Serialize)]
pub struct SourceResponse {
    pub id: i64,
    pub platform_id: String,
    pub creator_id: Option<i64>,
    pub name: String,
    pub url: String,
    pub source_type: String,
    pub feed_type: Option<String>,
    pub sync_mode: String,
    pub is_active: bool,
    pub last_checked: Option<String>,
    pub post_count: i64,
    pub is_self: bool,
    pub avatar_url: Option<String>,
}

#[derive(Serialize)]
pub struct AddSourceResponse {
    pub source_id: i64,
    pub items_queued: usize,
}

#[derive(Deserialize)]
pub struct AddSourceRequest {
    pub url: String,
    pub feed_types: Option<Vec<String>>,
    pub selected_ids: Option<Vec<String>>,
    pub limit_mode: Option<String>,
    pub max_items: Option<u32>,
}

#[derive(Deserialize)]
pub struct UpdateSourceRequest {
    pub source_id: i64,
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

#[tauri::command]
pub async fn get_sources_command(
    state: State<'_, AppState>,
) -> Result<Vec<SourceResponse>, String> {
    let sources = source::Entity::find()
        .all(&state.db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let mut count_map: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    let posts_source_ids = post::Entity::find()
        .select_only()
        .column(post::Column::SourceId)
        .into_tuple::<Option<i64>>()
        .all(&state.db)
        .await
        .unwrap_or_default();

    for id in posts_source_ids.into_iter().flatten() {
        *count_map.entry(id).or_insert(0) += 1;
    }

    // Fetch creators and sessions to enrich the UI
    let mut creator_ids: Vec<i64> = sources.iter().filter_map(|s| s.creator_id).collect();
    creator_ids.sort();
    creator_ids.dedup();

    let creators = creator::Entity::find()
        .filter(creator::Column::Id.is_in(creator_ids))
        .all(&state.db)
        .await
        .unwrap_or_default();

    let mut creator_map = std::collections::HashMap::new();
    for c in creators {
        creator_map.insert(c.id, c);
    }

    let sessions = platform_session::Entity::find()
        .all(&state.db)
        .await
        .unwrap_or_default();

    let mut session_map = std::collections::HashMap::new();
    for s in sessions {
        session_map.insert(s.platform_id.clone(), s);
    }

    let mut response = Vec::with_capacity(sources.len());

    for s in sources {
        let count = count_map.get(&s.id).copied().unwrap_or(0);
        
        let mut is_self = false;
        let mut avatar_url = None;
        let mut display_name = s.name.clone();

        if let Some(c_id) = s.creator_id {
            if let Some(creator) = creator_map.get(&c_id) {
                is_self = creator.is_self;
                
                if is_self {
                    // For self accounts, prefer session data
                    if let Some(session) = session_map.get(&creator.platform_id) {
                        display_name = session.username.clone().unwrap_or(display_name);
                        avatar_url = session.avatar_url.clone();
                    }
                } else {
                    // For public channels, use creator's avatar
                    avatar_url = creator.avatar_path.clone();
                }
            }
        }

        response.push(SourceResponse {
            id: s.id,
            platform_id: s.platform_id,
            creator_id: s.creator_id,
            name: display_name,
            url: s.url,
            source_type: s.source_type,
            feed_type: s.feed_type,
            sync_mode: s.sync_mode,
            is_active: s.is_active,
            last_checked: s.last_checked.map(|t| t.to_rfc3339()),
            post_count: count,
            is_self,
            avatar_url,
        });
    }

    Ok(response)
}

#[tauri::command]
pub async fn delete_source_command(
    state: State<'_, AppState>,
    source_id: i64,
) -> Result<(), String> {
    // Detach posts from this source (set source_id = NULL) to avoid FK violation
    post::Entity::update_many()
        .col_expr(post::Column::SourceId, Expr::value(Value::BigInt(None)))
        .filter(post::Column::SourceId.eq(source_id))
        .exec(&state.db)
        .await
        .map_err(|e| format!("Failed to detach posts: {}", e))?;

    source::Entity::delete_by_id(source_id)
        .exec(&state.db)
        .await
        .map_err(|e| format!("Failed to delete source: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn update_source_command(
    state: State<'_, AppState>,
    request: UpdateSourceRequest,
) -> Result<(), String> {
    // Build the update model with only the fields that were provided
    let mut active = source::ActiveModel {
        id: Set(request.source_id),
        ..Default::default()
    };

    if let Some(name) = request.name {
        active.name = Set(name);
    }

    if let Some(is_active) = request.is_active {
        active.is_active = Set(is_active);
    }

    source::Entity::update(active)
        .exec(&state.db)
        .await
        .map_err(|e| format!("Failed to update source: {}", e))?;

    Ok(())
}

const QUEUE_PRIORITY: i32 = 5;
const QUEUE_MAX_RETRIES: i32 = 3;

async fn handle_tiktok_source(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
    url: &str,
    section: crate::metadata::tiktok::TikTokSection,
    source_type: Option<&str>,
    feed_type: Option<&str>,
) -> Result<i64, String> {
    let username = crate::metadata::tiktok::helpers::extract_tiktok_username(url)
        .ok_or("Could not parse TikTok username from URL")?;

    let cookie_manager = app.state::<Arc<CookieManager>>();
    let cookies = cookie_manager
        .get_session("tiktok")
        .await
        .map_err(|e| e.to_string())?
        .ok_or("TikTok session not found. Please log in first.")?;

    let output = crate::metadata::tiktok::TikTokFetcher::new()
        .fetch_section(&cookies, &username, section, None)
        .await
        .map_err(|e| e.to_string())?;

    let platform_hint = crate::platform::detect_platform(url);
    let saved_id = store::save_metadata(
        &state.db,
        output,
        source_type,
        feed_type,
        platform_hint,
        Some(url),
    )
        .await
        .map_err(|e| format!("DB Error: {}", e))?;
    Ok(saved_id)
}

async fn handle_ytdlp_source(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
    url: &str,
    source_type: Option<&str>,
    feed_type: Option<&str>,
    max_items: Option<u32>,
) -> Result<i64, String> {
    let cookie_manager = app.state::<Arc<CookieManager>>();
    let mut temp_cookie_path = None;
    let platform_id = crate::platform::detect_platform(url);

    if let Some(pid) = platform_id {
        if let Ok(Some(path)) = cookie_manager.create_temp_cookie_file(pid).await {
            temp_cookie_path = Some(path);
        }
    }

    let raw_output = fetcher::fetch_metadata(app, url, temp_cookie_path.as_ref(), max_items)
        .await
        .map_err(|e| e.to_string());

    if let Some(path) = temp_cookie_path {
        if let Err(e) = cookie_manager.cleanup_temp_file(&path).await {
            tracing::warn!("Failed to cleanup cookies: {}", e);
        }
    }

    let output = raw_output?;

    if !matches!(output, YtDlpOutput::Playlist(_)) {
        return Err(
            "This URL points to a single video, not a playlist or channel. Use the download button instead."
                .to_string(),
        );
    }

    let saved_id = store::save_metadata(
        &state.db,
        output,
        source_type,
        feed_type,
        platform_id,
        Some(url),
    )
        .await
        .map_err(|e| format!("DB Error: {}", e))?;
    Ok(saved_id)
}

async fn queue_posts(
    state: &State<'_, AppState>,
    queue: &State<'_, DownloadQueue>,
    source_id: i64,
    selected_ids: Option<Vec<String>>,
) -> Result<usize, String> {
    let mut items_queued = 0;
    let selection_filter: Option<std::collections::HashSet<String>> =
        selected_ids.map(|ids| ids.into_iter().collect());

    let child_posts = post::Entity::find()
        .filter(post::Column::SourceId.eq(source_id))
        .all(&state.db)
        .await
        .map_err(|e| format!("Database error fetching posts: {}", e))?;

    for p in child_posts {
        if p.original_url.is_empty() {
            tracing::warn!("Skipping post {} — no download URL available", p.id);
            continue;
        }

        if let Some(ref filter) = selection_filter {
            if !filter.contains(&p.external_id) {
                continue;
            }
        }

        if p.status == "PENDING" {
            let new_task = download_task::ActiveModel {
                id: sea_orm::ActiveValue::NotSet,
                url: Set(p.original_url),
                post_id: Set(Some(p.id)),
                status: Set("QUEUED".to_string()),
                priority: Set(QUEUE_PRIORITY),
                progress: Set(0.0),
                retries: Set(0),
                max_retries: Set(QUEUE_MAX_RETRIES),
                format_selection: Set(None),
                created_at: Set(Utc::now()),
                ..Default::default()
            };

            if new_task.insert(&state.db).await.is_ok() {
                items_queued += 1;
                queue.add_task();
            }
        }
    }

    Ok(items_queued)
}

#[tauri::command]
pub async fn add_source_command(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    queue: State<'_, DownloadQueue>,
    request: AddSourceRequest,
) -> Result<Vec<AddSourceResponse>, String> {
    let mut actual_url = request.url;
    let mut responses = Vec::new();

    let feed_types = request.feed_types.unwrap_or_default();
    let max_items = if request.limit_mode.as_deref() == Some("custom") { request.max_items } else { None };

    // If no specific feed_types provided, act as a single source (original behavior)
    if feed_types.is_empty() {
        let mut source_type_arg = None;

        // --- Intercept vdp:// pseudo-URLs ---
        if actual_url.starts_with("vdp://tiktok/me/") {
            use sea_orm::EntityTrait;
            let session = crate::entity::platform_session::Entity::find_by_id("tiktok")
                .one(&state.db)
                .await
                .map_err(|e| e.to_string())?
                .ok_or("TikTok session not found. Please log in first.")?;

            let username = session.username.ok_or(
                "Could not resolve your TikTok username from the active session. Please log in again.",
            )?;

            let section = actual_url.strip_prefix("vdp://tiktok/me/").unwrap();
            
            if section == "saved" {
                source_type_arg = Some(crate::constants::source_type::SAVED);
            } else if section == "liked" {
                source_type_arg = Some(crate::constants::source_type::LIKED);
            }

            // Convert pseudo-URL into a real TikTok profile URL for processing and DB storage
            actual_url = format!("https://www.tiktok.com/@{}/{}", username, section);
        }
        // ------------------------------------

        let tiktok_section = crate::metadata::tiktok::helpers::detect_tiktok_section(&actual_url);
        let saved_id_res = if let Some(section) = tiktok_section {
            handle_tiktok_source(&app, &state, &actual_url, section, source_type_arg, None).await
        } else {
            handle_ytdlp_source(&app, &state, &actual_url, source_type_arg, None, max_items).await
        };

        if let Ok(saved_id) = saved_id_res {
            let items_queued = queue_posts(&state, &queue, saved_id, request.selected_ids).await?;
            responses.push(AddSourceResponse { source_id: saved_id, items_queued });
            return Ok(responses);
        } else {
            return Err(saved_id_res.unwrap_err());
        }
    }

    // MULTI-FEED SUPPORT
    for feed_type in feed_types {
        let mut feed_url = actual_url.clone();
        let mut source_type_arg = Some(crate::constants::source_type::CHANNEL);
        let mut feed_type_arg = Some(feed_type.as_str());

        // --- Intercept vdp:// pseudo-URLs for multi-feed ---
        if feed_url.starts_with("vdp://tiktok/me/") {
            use sea_orm::EntityTrait;
            let session = match crate::entity::platform_session::Entity::find_by_id("tiktok")
                .one(&state.db)
                .await
            {
                Ok(Some(s)) => s,
                _ => {
                    tracing::warn!("TikTok session not found for multi-feed");
                    continue;
                }
            };
            if let Some(username) = session.username {
                let section = feed_type.to_lowercase();
                if section == "saved" {
                    source_type_arg = Some(crate::constants::source_type::SAVED);
                    feed_type_arg = None;
                } else if section == "liked" {
                    source_type_arg = Some(crate::constants::source_type::LIKED);
                    feed_type_arg = None;
                } else if section == "videos" {
                    feed_type_arg = Some(crate::constants::feed_type::VIDEOS);
                }
                // When we do vdp://tiktok/me/ it means "current user". If feed_type is "videos", it corresponds to public profile.
                let url_suffix = if section == "videos" || section == "default" { "" } else { &section };
                feed_url = format!("https://www.tiktok.com/@{}/{}", username, url_suffix);
            } else {
                continue;
            }
        } else {
            // Basic URL mutator based on feed type (since frontend won't mutate for multi-select)
            let platform_id = crate::platform::detect_platform(&feed_url).unwrap_or("");
            
            if platform_id == "youtube" {
                if !feed_url.ends_with('/') {
                    feed_url.push('/');
                }
                match feed_type.as_str() {
                    crate::constants::feed_type::VIDEOS => {}, // base URL works
                    crate::constants::feed_type::SHORTS => feed_url.push_str("shorts"),
                    crate::constants::feed_type::STREAMS => feed_url.push_str("streams"),
                    _ => feed_url.push_str(&feed_type.to_lowercase()),
                }
            } else if platform_id == "tiktok" {
                let section = feed_type.to_lowercase();
                if section == "saved" {
                    source_type_arg = Some(crate::constants::source_type::SAVED);
                    feed_type_arg = None;
                    if !feed_url.ends_with('/') { feed_url.push('/'); }
                    feed_url.push_str("saved");
                } else if section == "liked" {
                    source_type_arg = Some(crate::constants::source_type::LIKED);
                    feed_type_arg = None;
                    if !feed_url.ends_with('/') { feed_url.push('/'); }
                    feed_url.push_str("liked");
                }
            }
        }

        let tiktok_section = crate::metadata::tiktok::helpers::detect_tiktok_section(&feed_url);
        let saved_id_res = if let Some(section) = tiktok_section {
            handle_tiktok_source(&app, &state, &feed_url, section, source_type_arg, feed_type_arg).await
        } else {
            handle_ytdlp_source(&app, &state, &feed_url, source_type_arg, feed_type_arg, max_items).await
        };

        match saved_id_res {
            Ok(saved_id) => {
                let items_queued = queue_posts(&state, &queue, saved_id, request.selected_ids.clone()).await.unwrap_or(0);
                responses.push(AddSourceResponse {
                    source_id: saved_id,
                    items_queued,
                });
            }
            Err(e) => {
                tracing::warn!("Failed to add feed {}: {}", feed_type, e);
                // Continue to the next feed type rather than failing the whole request
            }
        }
    }

    if responses.is_empty() {
        return Err("Failed to add any of the requested feeds.".to_string());
    }

    Ok(responses)
}
