use sea_orm::{
    ColumnTrait, EntityTrait, LoaderTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;
use tauri_plugin_opener::OpenerExt;
use trash::delete as move_to_trash;

use crate::{
    entity::{creator, media, post},
    AppState,
};

#[derive(Serialize, Deserialize)]
pub struct PostsPage {
    pub posts: Vec<PostResponse>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
    pub total_pages: u64,
}

#[derive(Serialize, Deserialize)]
pub struct PostResponse {
    pub id: String,
    pub creator_id: String,
    pub source_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub original_url: String,
    pub status: String,
    pub posted_at: Option<String>,
    pub downloaded_at: Option<String>,
    pub created_at: String,

    // Joined creator data
    pub creator_name: Option<String>,
    pub creator_handle: Option<String>,
    pub creator_avatar: Option<String>,

    pub media: Vec<MediaResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct MediaResponse {
    pub id: String,
    pub media_type: String,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub thumbnail_sm_path: Option<String>,
    pub order_index: i32,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<i32>,
    pub file_size: Option<i32>,
}

#[tauri::command]
pub async fn get_posts(
    state: State<'_, AppState>,
    page: u64,
    limit: u64,
) -> Result<PostsPage, String> {
    let p = page.max(1);
    let offset = (p - 1) * limit;

    let posts = post::Entity::find()
        .filter(post::Column::Status.eq("COMPLETED"))
        .order_by_desc(post::Column::DownloadedAt)
        .offset(offset)
        .limit(limit)
        .all(&state.db)
        .await
        .map_err(|e| format!("Database error fetching posts: {}", e))?;

    let creators: Vec<Option<creator::Model>> = posts
        .load_one(creator::Entity, &state.db)
        .await
        .map_err(|e| format!("Database error loading creators: {}", e))?;

    let media_lists: Vec<Vec<media::Model>> = posts
        .load_many(media::Entity, &state.db)
        .await
        .map_err(|e| format!("Database error loading media: {}", e))?;

    let total = post::Entity::find()
        .filter(post::Column::Status.eq("COMPLETED"))
        .count(&state.db)
        .await
        .map_err(|e| format!("Database error counting posts: {}", e))?;

    let total_pages = (total as f64 / limit as f64).ceil() as u64;

    let mut response_posts = Vec::with_capacity(posts.len());

    for (i, post) in posts.into_iter().enumerate() {
        let creator_opt = creators.get(i).cloned().flatten();
        let medias = media_lists.get(i).cloned().unwrap_or_default();

        let mut media_responses: Vec<MediaResponse> = medias
            .into_iter()
            .map(|m| MediaResponse {
                id: m.id,
                media_type: m.media_type,
                file_path: m.file_path,
                thumbnail_path: m.thumbnail_path,
                thumbnail_sm_path: m.thumbnail_sm_path,
                order_index: m.order_index,
                width: m.width,
                height: m.height,
                duration: m.duration,
                file_size: m.file_size,
            })
            .collect();

        // Ensure media is ordered by order_index
        media_responses.sort_by_key(|m| m.order_index);

        response_posts.push(PostResponse {
            id: post.id,
            creator_id: post.creator_id,
            source_id: post.source_id,
            title: post.title,
            description: post.description,
            original_url: post.original_url,
            status: post.status,
            posted_at: post.posted_at.map(|t| t.to_rfc3339()),
            downloaded_at: post.downloaded_at.map(|t| t.to_rfc3339()),
            created_at: post.created_at.to_rfc3339(),

            creator_name: creator_opt.as_ref().map(|c| c.name.clone()),
            creator_handle: creator_opt.as_ref().and_then(|c| c.handle.clone()),
            creator_avatar: creator_opt.as_ref().and_then(|c| c.avatar_path.clone()),

            media: media_responses,
        });
    }

    Ok(PostsPage {
        posts: response_posts,
        total,
        page: p,
        limit,
        total_pages,
    })
}

#[tauri::command]
pub async fn reveal_in_explorer(
    app_handle: tauri::AppHandle,
    file_path: String,
) -> Result<(), String> {
    app_handle
        .opener()
        .reveal_item_in_dir(file_path.clone())
        .map_err(|e| format!("Failed to reveal {} in explorer: {}", file_path, e))
}

#[tauri::command]
pub async fn delete_post(state: State<'_, AppState>, post_id: String) -> Result<(), String> {
    // 1. Fetch media for this post to get file paths using sea-orm
    let medias = media::Entity::find()
        .filter(media::Column::PostId.eq(&post_id))
        .all(&state.db)
        .await
        .map_err(|e| format!("Database error finding media: {}", e))?;

    // 2. Iterate and send files to trash, ignoring missing ones
    for m in medias {
        if Path::new(&m.file_path).exists() {
            let _ = move_to_trash(&m.file_path);
        }
        if let Some(thumb) = &m.thumbnail_path {
            if Path::new(thumb).exists() {
                let _ = move_to_trash(thumb);
            }
        }
        if let Some(thumb_sm) = &m.thumbnail_sm_path {
            if Path::new(thumb_sm).exists() {
                let _ = move_to_trash(thumb_sm);
            }
        }
    }

    // 3. Hard delete records from DB
    media::Entity::delete_many()
        .filter(media::Column::PostId.eq(&post_id))
        .exec(&state.db)
        .await
        .map_err(|e| format!("Failed to delete media from db: {}", e))?;

    post::Entity::delete_by_id(&post_id)
        .exec(&state.db)
        .await
        .map_err(|e| format!("Failed to delete post from db: {}", e))?;

    Ok(())
}
