use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── 1. platforms ──────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Platforms::Table)
                    .if_not_exists()
                    .col(string(Platforms::Id).primary_key())
                    .col(string(Platforms::Name).not_null())
                    .col(string(Platforms::BaseUrl).not_null())
                    .col(string_null(Platforms::IconPath))
                    .to_owned(),
            )
            .await?;

        // ── 2. creators ───────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Creators::Table)
                    .if_not_exists()
                    .col(string(Creators::Id).primary_key())
                    .col(string(Creators::PlatformId).not_null())
                    .col(string(Creators::Name).not_null())
                    .col(string_null(Creators::Handle))
                    .col(string(Creators::Url).not_null())
                    .col(string_null(Creators::AvatarPath))
                    .col(
                        timestamp(Creators::CreatedAt)
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Creators::Table, Creators::PlatformId)
                            .to(Platforms::Table, Platforms::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // ── 3. sources ────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Sources::Table)
                    .if_not_exists()
                    .col(string(Sources::Id).primary_key())
                    .col(string(Sources::PlatformId).not_null())
                    .col(string_null(Sources::CreatorId))
                    .col(string(Sources::Type).not_null())
                    .col(string(Sources::Name).not_null())
                    .col(string(Sources::Url).not_null())
                    .col(string(Sources::SyncMode).not_null())
                    .col(timestamp_null(Sources::DateStart))
                    .col(timestamp_null(Sources::DateEnd))
                    .col(integer_null(Sources::MaxItems))
                    .col(timestamp_null(Sources::LastChecked))
                    .col(
                        boolean(Sources::IsActive)
                            .default(Value::Bool(Some(true)))
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Sources::Table, Sources::PlatformId)
                            .to(Platforms::Table, Platforms::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Sources::Table, Sources::CreatorId)
                            .to(Creators::Table, Creators::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // ── 4. posts ──────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Posts::Table)
                    .if_not_exists()
                    .col(string(Posts::Id).primary_key())
                    .col(string(Posts::CreatorId).not_null())
                    .col(string_null(Posts::SourceId))
                    .col(string_null(Posts::Title))
                    .col(string_null(Posts::Description))
                    .col(string(Posts::OriginalUrl).not_null())
                    .col(
                        string(Posts::Status)
                            .default("PENDING")
                            .not_null(),
                    )
                    .col(timestamp_null(Posts::PostedAt))
                    .col(timestamp_null(Posts::DownloadedAt))
                    .col(timestamp_null(Posts::DeletedAt))
                    .col(string_null(Posts::RawJson))
                    .col(
                        timestamp(Posts::CreatedAt)
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Posts::Table, Posts::CreatorId)
                            .to(Creators::Table, Creators::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Posts::Table, Posts::SourceId)
                            .to(Sources::Table, Sources::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // ── 5. media ──────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Media::Table)
                    .if_not_exists()
                    .col(string(Media::Id).primary_key())
                    .col(string(Media::PostId).not_null())
                    .col(string(Media::Type).not_null())
                    .col(string(Media::FilePath).not_null())
                    .col(string_null(Media::ThumbnailPath))
                    .col(string_null(Media::ThumbnailSmPath))
                    .col(
                        integer(Media::OrderIndex)
                            .default(Value::Int(Some(0)))
                            .not_null(),
                    )
                    .col(integer_null(Media::Width))
                    .col(integer_null(Media::Height))
                    .col(integer_null(Media::Duration))
                    .col(integer_null(Media::FileSize))
                    .col(string_null(Media::FormatId))
                    .col(string_null(Media::Checksum))
                    .col(timestamp_null(Media::DeletedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Media::Table, Media::PostId)
                            .to(Posts::Table, Posts::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // ── 6. download_tasks ─────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(DownloadTasks::Table)
                    .if_not_exists()
                    .col(string(DownloadTasks::Id).primary_key())
                    .col(string(DownloadTasks::Url).not_null())
                    .col(string_null(DownloadTasks::PostId))
                    .col(
                        string(DownloadTasks::Status)
                            .default("QUEUED")
                            .not_null(),
                    )
                    .col(
                        integer(DownloadTasks::Priority)
                            .default(Value::Int(Some(0)))
                            .not_null(),
                    )
                    .col(
                        float(DownloadTasks::Progress)
                            .default(Value::Float(Some(0.0)))
                            .not_null(),
                    )
                    .col(string_null(DownloadTasks::Speed))
                    .col(string_null(DownloadTasks::Eta))
                    .col(string_null(DownloadTasks::ErrorMessage))
                    .col(
                        integer(DownloadTasks::Retries)
                            .default(Value::Int(Some(0)))
                            .not_null(),
                    )
                    .col(
                        integer(DownloadTasks::MaxRetries)
                            .default(Value::Int(Some(3)))
                            .not_null(),
                    )
                    .col(string_null(DownloadTasks::FormatSelection))
                    .col(
                        timestamp(DownloadTasks::CreatedAt)
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(timestamp_null(DownloadTasks::StartedAt))
                    .col(timestamp_null(DownloadTasks::CompletedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(DownloadTasks::Table, DownloadTasks::PostId)
                            .to(Posts::Table, Posts::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // ── 7. settings ───────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Settings::Table)
                    .if_not_exists()
                    .col(string(Settings::Key).primary_key())
                    .col(string(Settings::Value).not_null())
                    .col(
                        timestamp(Settings::UpdatedAt)
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // ── 8. platform_sessions ──────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(PlatformSessions::Table)
                    .if_not_exists()
                    .col(string(PlatformSessions::PlatformId).primary_key())
                    .col(
                        string(PlatformSessions::Status)
                            .default("NONE")
                            .not_null(),
                    )
                    .col(ColumnDef::new(PlatformSessions::EncryptedCookies).binary().null())
                    .col(
                        string(PlatformSessions::CookieMethod)
                            .default("webview")
                            .not_null(),
                    )
                    .col(timestamp_null(PlatformSessions::ExpiresAt))
                    .col(timestamp_null(PlatformSessions::LastVerified))
                    .col(
                        timestamp(PlatformSessions::CreatedAt)
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        timestamp(PlatformSessions::UpdatedAt)
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PlatformSessions::Table, PlatformSessions::PlatformId)
                            .to(Platforms::Table, Platforms::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // ── Indexes ───────────────────────────────────────────────
        manager
            .create_index(
                Index::create()
                    .name("idx_posts_timeline")
                    .table(Posts::Table)
                    .col(Posts::PostedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_posts_status")
                    .table(Posts::Table)
                    .col(Posts::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_posts_deleted")
                    .table(Posts::Table)
                    .col(Posts::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_media_post")
                    .table(Media::Table)
                    .col(Media::PostId)
                    .col(Media::OrderIndex)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_media_checksum")
                    .table(Media::Table)
                    .col(Media::Checksum)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_download_tasks_status")
                    .table(DownloadTasks::Table)
                    .col(DownloadTasks::Status)
                    .col(DownloadTasks::Priority)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_download_tasks_created")
                    .table(DownloadTasks::Table)
                    .col(DownloadTasks::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // ── Seed: Platforms ───────────────────────────────────────
        let insert_platforms = Query::insert()
            .into_table(Platforms::Table)
            .columns([
                Platforms::Id,
                Platforms::Name,
                Platforms::BaseUrl,
            ])
            .values_panic(["youtube".into(), "YouTube".into(), "https://www.youtube.com".into()])
            .values_panic(["tiktok".into(), "TikTok".into(), "https://www.tiktok.com".into()])
            .values_panic(["instagram".into(), "Instagram".into(), "https://www.instagram.com".into()])
            .values_panic(["x".into(), "X (Twitter)".into(), "https://x.com".into()])
            .to_owned();

        manager.exec_stmt(insert_platforms).await?;

        // ── Seed: Default Settings ────────────────────────────────
        let defaults: Vec<(&str, &str)> = vec![
            ("download_path", "~/Downloads/VideoDownloaderPro"),
            ("max_concurrent_downloads", "3"),
            ("cookie_method", "webview"),
            ("cookie_browser", "chrome"),
            ("default_video_format", "best"),
            ("default_audio_format", "best"),
            ("trash_auto_clean_days", "30"),
            ("delete_files_on_remove", "false"),
            ("disk_space_warning_gb", "5"),
            ("ytdlp_auto_update", "true"),
            ("ytdlp_update_interval_hours", "24"),
            ("language", "en"),
            ("sleep_interval", "2"),
            ("sleep_requests", "1"),
        ];

        for (key, value) in defaults {
            let stmt = Query::insert()
                .into_table(Settings::Table)
                .columns([Settings::Key, Settings::Value])
                .values_panic([key.into(), value.into()])
                .to_owned();
            manager.exec_stmt(stmt).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop in reverse order to respect foreign keys
        let tables = [
            PlatformSessions::Table.into_table_ref(),
            Settings::Table.into_table_ref(),
            DownloadTasks::Table.into_table_ref(),
            Media::Table.into_table_ref(),
            Posts::Table.into_table_ref(),
            Sources::Table.into_table_ref(),
            Creators::Table.into_table_ref(),
            Platforms::Table.into_table_ref(),
        ];

        for table in tables {
            manager
                .drop_table(Table::drop().table(table).if_exists().to_owned())
                .await?;
        }

        Ok(())
    }
}

// ── Iden enums for type-safe table/column references ──────────────

#[derive(DeriveIden)]
pub enum Platforms {
    Table,
    Id,
    Name,
    BaseUrl,
    IconPath,
}

#[derive(DeriveIden)]
pub enum Creators {
    Table,
    Id,
    PlatformId,
    Name,
    Handle,
    Url,
    AvatarPath,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum Sources {
    Table,
    Id,
    PlatformId,
    CreatorId,
    Type,
    Name,
    Url,
    SyncMode,
    DateStart,
    DateEnd,
    MaxItems,
    LastChecked,
    IsActive,
}

#[derive(DeriveIden)]
pub enum Posts {
    Table,
    Id,
    CreatorId,
    SourceId,
    Title,
    Description,
    OriginalUrl,
    Status,
    PostedAt,
    DownloadedAt,
    DeletedAt,
    RawJson,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum Media {
    Table,
    Id,
    PostId,
    Type,
    FilePath,
    ThumbnailPath,
    ThumbnailSmPath,
    OrderIndex,
    Width,
    Height,
    Duration,
    FileSize,
    FormatId,
    Checksum,
    DeletedAt,
}

#[derive(DeriveIden)]
pub enum DownloadTasks {
    Table,
    Id,
    Url,
    PostId,
    Status,
    Priority,
    Progress,
    Speed,
    Eta,
    ErrorMessage,
    Retries,
    MaxRetries,
    FormatSelection,
    CreatedAt,
    StartedAt,
    CompletedAt,
}

#[derive(DeriveIden)]
pub enum Settings {
    Table,
    Key,
    Value,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum PlatformSessions {
    Table,
    PlatformId,
    Status,
    EncryptedCookies,
    CookieMethod,
    ExpiresAt,
    LastVerified,
    CreatedAt,
    UpdatedAt,
}
