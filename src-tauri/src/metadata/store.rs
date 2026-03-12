use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr,
    EntityTrait, QueryFilter, TransactionTrait,
};

use super::models::{YtDlpOutput, YtDlpPlaylist, YtDlpVideo};
use crate::entity::{creator, post, source}; // media removed as unused for now

/// Saves the fetched metadata to the database.
/// Returns the ID of the main post created/updated, or the playlist ID.
pub async fn save_metadata(
    db: &DatabaseConnection,
    metadata: YtDlpOutput,
    source_type: Option<&str>,
    feed_type: Option<&str>,
    platform_hint: Option<&str>,
    source_url_hint: Option<&str>,
) -> Result<i64, DbErr> {
    match metadata {
        YtDlpOutput::Video(v) => save_video(db, v).await,
        YtDlpOutput::VideoFallback(v) => save_video(db, v).await,
        YtDlpOutput::Playlist(p) => save_playlist(
            db,
            p,
            source_type.unwrap_or(crate::constants::source_type::PLAYLIST),
            feed_type,
            platform_hint,
            source_url_hint,
        )
        .await,
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

async fn save_playlist(
    db: &DatabaseConnection,
    p: YtDlpPlaylist,
    source_type: &str,
    feed_type: Option<&str>,
    platform_hint: Option<&str>,
    source_url_hint: Option<&str>,
) -> Result<i64, DbErr> {
    let txn = db.begin().await?;

    // 1. Process Creator
    let entry_platform_hint = p.entries.as_ref().and_then(|entries| {
        entries.iter().find_map(|entry| match entry {
            YtDlpOutput::Video(v) | YtDlpOutput::VideoFallback(v) => v
                .webpage_url
                .as_deref()
                .or(v.url.as_deref()),
            _ => None,
        })
    });

    let inferred_platform = platform_hint
        .or_else(|| p.webpage_url.as_deref().and_then(crate::platform::detect_platform))
        .or_else(|| source_url_hint.and_then(crate::platform::detect_platform))
        .or_else(|| entry_platform_hint.and_then(crate::platform::detect_platform))
        .unwrap_or("unknown")
        .to_string();

    if inferred_platform == "unknown" {
        return Err(DbErr::Custom(
            "Unsupported or unknown platform for playlist".to_string(),
        ));
    }

    let creator_id = if let (Some(ext_id), Some(name)) = (&p.uploader_id, &p.uploader) {
        let existing = creator::Entity::find()
            .filter(creator::Column::PlatformId.eq(&inferred_platform))
            .filter(creator::Column::ExternalId.eq(ext_id))
            .one(&txn)
            .await?;

        if let Some(existing_creator) = existing {
            let mut active: creator::ActiveModel = existing_creator.into();
            active.name = Set(name.clone());
            active.url = Set(p.webpage_url.clone().unwrap_or_default());
            let updated = active.update(&txn).await?;
            Some(updated.id)
        } else {
            let active_creator = creator::ActiveModel {
                id: sea_orm::ActiveValue::NotSet,
                platform_id: Set(inferred_platform.clone()),
                external_id: Set(Some(ext_id.clone())),
                is_self: Set(false),
                name: Set(name.clone()),
                url: Set(p.webpage_url.clone().unwrap_or_default()),
                ..Default::default()
            };

            let result = creator::Entity::insert(active_creator).exec(&txn).await?;
            Some(result.last_insert_id)
        }
    } else {
        None
    };

    // 2. Upsert Source (Playlist)
    let external_id = p.id.clone();

    let source_url = p
        .webpage_url
        .clone()
        .or_else(|| source_url_hint.map(|s| s.to_string()))
        .unwrap_or_default();
    let source_name = p.title;

    // Manual upsert: SQLite partial indexes can't be targeted by ON CONFLICT.
    // Check for existing source by (creator_id + feed_type) or by url.
    let existing_source = if feed_type.is_some() && creator_id.is_some() {
        source::Entity::find()
            .filter(source::Column::CreatorId.eq(creator_id))
            .filter(source::Column::FeedType.eq(feed_type))
            .one(&txn)
            .await?
    } else {
        source::Entity::find()
            .filter(source::Column::Url.eq(&source_url))
            .one(&txn)
            .await?
    };

    let source_id = if let Some(existing) = existing_source {
        // Update existing source
        let mut active: source::ActiveModel = existing.into();
        active.name = Set(source_name);
        active.url = Set(source_url);
        let updated = active.update(&txn).await?;
        updated.id
    } else {
        // Insert new source
        let active_source = source::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            platform_id: Set(inferred_platform.clone()),
            creator_id: Set(creator_id),
            external_id: Set(Some(external_id.clone())),
            source_type: Set(source_type.to_string()),
            feed_type: Set(feed_type.map(String::from)),
            name: Set(source_name),
            url: Set(source_url),
            sync_mode: Set("ALL".to_string()),
            is_active: Set(true),
            ..Default::default()
        };
        let result = source::Entity::insert(active_source).exec(&txn).await?;
        result.last_insert_id
    };

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

    let existing = if let Some(ref ext_id) = external_id {
        creator::Entity::find()
            .filter(creator::Column::PlatformId.eq(&platform))
            .filter(creator::Column::ExternalId.eq(ext_id))
            .one(db)
            .await?
    } else {
        creator::Entity::find()
            .filter(creator::Column::PlatformId.eq(&platform))
            .filter(creator::Column::Name.eq(&name))
            .one(db)
            .await?
    };

    if let Some(existing_creator) = existing {
        let mut active: creator::ActiveModel = existing_creator.into();
        active.name = Set(name);
        active.url = Set(url);
        let updated = active.update(db).await?;
        Ok(updated.id)
    } else {
        let active = creator::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            platform_id: Set(platform),
            external_id: Set(external_id),
            is_self: Set(false),
            name: Set(name),
            url: Set(url),
            ..Default::default()
        };

        let result = creator::Entity::insert(active).exec(db).await?;
        Ok(result.last_insert_id)
    }
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

    let original_url = v
        .webpage_url
        .clone()
        .or_else(|| v.url.clone())
        .or_else(|| v.original_url.clone())
        .unwrap_or_default();

    let existing = post::Entity::find()
        .filter(post::Column::ExternalId.eq(&external_id))
        .one(db)
        .await?;

    if let Some(existing_post) = existing {
        let mut active: post::ActiveModel = existing_post.into();
        active.title = Set(Some(v.title.clone()));
        active.description = Set(v.description.clone());
        active.raw_json = Set(raw_json);
        active.source_id = Set(source_id);
        active.original_url = Set(original_url);
        let updated = active.update(db).await?;
        Ok(updated.id)
    } else {
        let active = post::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            creator_id: Set(creator_id),
            source_id: Set(source_id),
            external_id: Set(external_id),
            title: Set(Some(v.title.clone())),
            description: Set(v.description.clone()),
            original_url: Set(original_url),
            status: Set("PENDING".to_string()),
            posted_at: Set(parse_date(&v.upload_date)),
            raw_json: Set(raw_json),
            ..Default::default()
        };

        let result = post::Entity::insert(active).exec(db).await?;
        Ok(result.last_insert_id)
    }
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
