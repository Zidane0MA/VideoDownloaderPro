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
) -> Result<i64, DbErr> {
    match metadata {
        YtDlpOutput::Video(v) => save_video(db, v).await,
        YtDlpOutput::VideoFallback(v) => save_video(db, v).await,
        YtDlpOutput::Playlist(p) => save_playlist(db, p).await,
    }
}

async fn save_video(db: &DatabaseConnection, v: YtDlpVideo) -> Result<i64, DbErr> {
    let txn = db.begin().await?;

    // 1. Process Creator
    let creator_id = upsert_creator(&txn, &v).await?;

    // 2. Process Post
    let post_id = upsert_post(&txn, &v, creator_id, None).await?;

    txn.commit().await?;

    Ok(post_id)
}

async fn save_playlist(db: &DatabaseConnection, p: YtDlpPlaylist) -> Result<i64, DbErr> {
    let txn = db.begin().await?;

    // 1. Process Creator
    let inferred_platform = p
        .webpage_url
        .as_deref()
        .and_then(crate::platform::detect_platform)
        .unwrap_or("unknown")
        .to_string();

    let creator_id = if let (Some(ext_id), Some(name)) = (&p.uploader_id, &p.uploader) {
        let active_creator = creator::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            platform_id: Set(inferred_platform.clone()),
            external_id: Set(Some(ext_id.clone())),
            is_self: Set(false),
            name: Set(name.clone()),
            url: Set(p.webpage_url.clone().unwrap_or_default()),
            ..Default::default()
        };

        let result = creator::Entity::insert(active_creator)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([creator::Column::PlatformId, creator::Column::ExternalId])
                    .update_columns([creator::Column::Name, creator::Column::Url])
                    .to_owned(),
            )
            .exec(&txn)
            .await?;
        Some(result.last_insert_id)
    } else {
        None
    };

    // 2. Upsert Source (Playlist)
    let external_id = p.id.clone();

    let active_source = source::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        platform_id: Set(inferred_platform.clone()),
        creator_id: Set(creator_id),
        external_id: Set(Some(external_id.clone())),
        source_type: Set("PLAYLIST".to_string()),
        name: Set(p.title),
        url: Set(p.webpage_url.unwrap_or_default()),
        sync_mode: Set("ALL".to_string()),
        is_active: Set(true),
        ..Default::default()
    };

    let result = source::Entity::insert(active_source)
        .on_conflict(
            sea_orm::sea_query::OnConflict::columns([source::Column::PlatformId, source::Column::ExternalId])
                .update_columns([source::Column::Name, source::Column::Url])
                .to_owned(),
        )
        .exec(&txn)
        .await?;
    let source_id = result.last_insert_id;

    // 3. Process Entries
    if let Some(entries) = p.entries {
        for entry in entries {
            match entry {
                YtDlpOutput::Video(v) | YtDlpOutput::VideoFallback(v) => {
                    let c_id = upsert_creator(&txn, &v).await?;
                    upsert_post(&txn, &v, c_id, Some(source_id)).await?;
                }
                _ => {}
            }
        }
    }

    txn.commit().await?;

    Ok(source_id)
}

async fn upsert_creator(db: &impl ConnectionTrait, v: &YtDlpVideo) -> Result<i64, DbErr> {
    let external_id = v
        .uploader_id
        .clone()
        .or_else(|| v.channel_id.clone());
    let name = v
        .uploader
        .clone()
        .or_else(|| v.channel.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let url = v
        .uploader_url
        .clone()
        .or_else(|| v.channel_url.clone())
        .unwrap_or_default();

    let platform = v
        .webpage_url
        .as_deref()
        .or(v.url.as_deref())
        .or(v.uploader_url.as_deref())
        .or(v.channel_url.as_deref())
        .and_then(crate::platform::detect_platform)
        .unwrap_or("unknown")
        .to_string();

    let active = creator::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        platform_id: Set(platform),
        external_id: Set(external_id),
        is_self: Set(false),
        name: Set(name),
        url: Set(url),
        ..Default::default()
    };

    let result = creator::Entity::insert(active)
        .on_conflict(
            sea_orm::sea_query::OnConflict::columns([creator::Column::PlatformId, creator::Column::ExternalId])
                .update_columns([creator::Column::Name, creator::Column::Url])
                .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(result.last_insert_id)
}

async fn upsert_post(
    db: &impl ConnectionTrait,
    v: &YtDlpVideo,
    creator_id: i64,
    source_id: Option<i64>,
) -> Result<i64, DbErr> {
    let external_id = v.id.clone();

    // Serialize full JSON for raw storage
    let raw_json = serde_json::to_string(v).ok();

    let active = post::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        creator_id: Set(creator_id),
        source_id: Set(source_id),
        external_id: Set(external_id.clone()),
        title: Set(Some(v.title.clone())),
        description: Set(v.description.clone()),
        original_url: Set(v
            .webpage_url
            .clone()
            .or_else(|| v.url.clone())
            .or_else(|| v.original_url.clone())
            .unwrap_or_default()),
        status: Set("PENDING".to_string()),
        posted_at: Set(parse_date(&v.upload_date)),
        raw_json: Set(raw_json),
        ..Default::default()
    };

    let result = post::Entity::insert(active)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(post::Column::ExternalId)
                .update_columns([
                    post::Column::Title,
                    post::Column::Description,
                    post::Column::RawJson,
                    post::Column::SourceId,
                    post::Column::OriginalUrl, // Fix stale empty URLs from flat-playlist entries
                ])
                .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(result.last_insert_id)
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
