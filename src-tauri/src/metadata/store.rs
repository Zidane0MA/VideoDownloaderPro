use sea_orm::{
    ActiveValue::Set, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, TransactionTrait,
};

use super::models::{YtDlpOutput, YtDlpPlaylist, YtDlpVideo};
use crate::entity::{creator, post, source}; // media removed as unused for now

/// Saves the fetched metadata to the database.
/// Returns the ID of the main post created/updated, or the playlist ID.
pub async fn save_metadata(
    db: &DatabaseConnection,
    metadata: YtDlpOutput,
) -> Result<String, DbErr> {
    match metadata {
        YtDlpOutput::Video(v) => save_video(db, v).await,
        YtDlpOutput::VideoFallback(v) => save_video(db, v).await,
        YtDlpOutput::Playlist(p) => save_playlist(db, p).await,
    }
}

async fn save_video(db: &DatabaseConnection, v: YtDlpVideo) -> Result<String, DbErr> {
    let txn = db.begin().await?;

    // 1. Process Creator
    let creator_id = upsert_creator(&txn, &v).await?;

    // 2. Process Post
    let post_id = upsert_post(&txn, &v, &creator_id, None).await?;

    txn.commit().await?;

    Ok(post_id)
}

async fn save_playlist(db: &DatabaseConnection, p: YtDlpPlaylist) -> Result<String, DbErr> {
    let txn = db.begin().await?;

    // 1. Upsert Source (Playlist)
    let source_id = p.id.clone();

    // Check if creator exists for the playlist uploader
    let creator_id = if let (Some(id), Some(name)) = (&p.uploader_id, &p.uploader) {
        let active_creator = creator::ActiveModel {
            id: Set(id.clone()),
            platform_id: Set("youtube".to_string()),
            name: Set(name.clone()),
            url: Set(p.webpage_url.clone().unwrap_or_default()),
            ..Default::default()
        };

        creator::Entity::insert(active_creator)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(creator::Column::Id)
                    .update_columns([creator::Column::Name, creator::Column::Url])
                    .to_owned(),
            )
            .exec(&txn)
            .await?;
        Some(id.clone())
    } else {
        None
    };

    let active_source = source::ActiveModel {
        id: Set(source_id.clone()),
        platform_id: Set("youtube".to_string()),
        creator_id: Set(creator_id),
        source_type: Set("PLAYLIST".to_string()),
        name: Set(p.title),
        url: Set(p.webpage_url.unwrap_or_default()),
        sync_mode: Set("ALL".to_string()),
        is_active: Set(true),
        ..Default::default()
    };

    source::Entity::insert(active_source)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(source::Column::Id)
                .update_columns([source::Column::Name, source::Column::Url])
                .to_owned(),
        )
        .exec(&txn)
        .await?;

    // 2. Process Entries
    if let Some(entries) = p.entries {
        for entry in entries {
            match entry {
                YtDlpOutput::Video(v) | YtDlpOutput::VideoFallback(v) => {
                    // Link to source
                    let c_id = upsert_creator(&txn, &v).await?;
                    upsert_post(&txn, &v, &c_id, Some(source_id.clone())).await?;
                }
                _ => {}
            }
        }
    }

    txn.commit().await?;

    Ok(source_id)
}

async fn upsert_creator(db: &impl ConnectionTrait, v: &YtDlpVideo) -> Result<String, DbErr> {
    let id = v
        .uploader_id
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let name = v.uploader.clone().unwrap_or_else(|| "Unknown".to_string());
    let url = v.uploader_url.clone().unwrap_or_default();

    let active = creator::ActiveModel {
        id: Set(id.clone()),
        platform_id: Set("youtube".to_string()), // TODO: Infer from URL or yt-dlp extractor
        name: Set(name),
        url: Set(url),
        ..Default::default()
    };

    creator::Entity::insert(active)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(creator::Column::Id)
                .update_columns([creator::Column::Name, creator::Column::Url])
                .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(id)
}

async fn upsert_post(
    db: &impl ConnectionTrait,
    v: &YtDlpVideo,
    creator_id: &str,
    source_id: Option<String>,
) -> Result<String, DbErr> {
    let id = v.id.clone();

    // Serialize full JSON for raw storage
    let raw_json = serde_json::to_string(v).ok();

    let active = post::ActiveModel {
        id: Set(id.clone()),
        creator_id: Set(creator_id.to_string()),
        source_id: Set(source_id),
        title: Set(Some(v.title.clone())),
        description: Set(v.description.clone()),
        original_url: Set(v.webpage_url.clone().unwrap_or_default()),
        status: Set("PENDING".to_string()),
        posted_at: Set(parse_date(&v.upload_date)),
        raw_json: Set(raw_json),
        ..Default::default()
    };

    post::Entity::insert(active)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(post::Column::Id)
                .update_columns([
                    post::Column::Title,
                    post::Column::Description,
                    post::Column::RawJson,
                    post::Column::SourceId, // Update source if it was missing?
                ])
                .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(id)
}

fn parse_date(date_str: &Option<String>) -> Option<chrono::DateTime<chrono::Utc>> {
    if let Some(s) = date_str {
        if let Ok(naive) = chrono::NaiveDate::parse_from_str(s, "%Y%m%d") {
            return Some(chrono::DateTime::from_naive_utc_and_offset(
                naive.and_hms_opt(0, 0, 0).unwrap(),
                chrono::Utc,
            ));
        }
    }
    None
}
