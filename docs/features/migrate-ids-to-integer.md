# Task: Migrate Entity IDs from UUID strings to Integer (Autoincrement)

## Context

VideoDownloaderPro is a Tauri + Rust desktop app using SeaORM with a SQLite database.
The project is in **early testing phase — the database has no production data** and
can be fully wiped and re-initialized. This means the migration is a clean rewrite,
not an incremental patch.

The decision to move from UUID strings to autoincrement integers is driven by a
companion project (Tagflow) that uses cursor-based pagination over thousands of
video records with multiple filters. Integer PKs make cursor queries trivial and
efficient: `WHERE id > 1450 ORDER BY id ASC LIMIT 50`. UUID v4 strings are random
and cannot be used as meaningful cursors.

---

## Current State

All entity tables use `String` primary keys populated with `Uuid::new_v4().to_string()`.
The relevant files are:

### Entity models (`src-tauri/src/entity/`)
- `creator.rs`        — `pub id: String`
- `post.rs`           — `pub id: String`
- `source.rs`         — `pub id: String`
- `media.rs`          — `pub id: String`
- `download_task.rs`  — `pub id: String`
- `platform.rs`       — `pub id: String` (seeded with values like `"youtube"`, `"tiktok"`)
- `platform_session.rs` — `pub platform_id: String` (FK to platforms, intentional string key)
- `setting.rs`        — `pub key: String` (named key-value store, intentional string key)

### Migration (`src-tauri/src/migration/m20260217_000001_initial_schema.rs`)
Initial schema. Creates all tables. All PKs currently use `string(...)`.

### Additional migrations
- `m20260217_000002_add_download_stats.rs`
- `m20260219_000001_add_username_to_sessions.rs`
- `m20260224_000001_add_avatar_and_error_to_sessions.rs`

### Code that generates IDs (`src-tauri/src/`)
- `commands/sources.rs` — `Uuid::new_v4().to_string()` for source and task IDs
- `queue/manager.rs`    — `Uuid::new_v4().to_string()` for media and task IDs
- `metadata/store.rs`   — uses yt-dlp's `uploader_id`/`channel_id` as `creator.id`,
                          and yt-dlp's `playlist.id` / `video.id` as `source.id` / `post.id`

---

## Scope of Changes Required

### 1. Tables that get integer autoincrement PKs

| Table            | Current PK type | Notes |
|------------------|-----------------|-------|
| `creators`       | `String`        | Needs new `external_id` and `is_self` columns (see below) |
| `posts`          | `String`        | yt-dlp video ID moves to `external_id TEXT` |
| `sources`        | `String`        | yt-dlp playlist ID moves to `external_id TEXT NULL` |
| `media`          | `String`        | Pure internal ID, no external reference |
| `download_tasks` | `String`        | Pure internal ID, no external reference |

### 2. Tables that do NOT change

| Table               | Reason |
|---------------------|--------|
| `platforms`         | Seeded with meaningful string IDs (`"youtube"`, `"tiktok"`) used throughout for platform detection. Keep `String`. |
| `platform_sessions` | PK is `platform_id TEXT` — FK to `platforms.id`. Keep as-is. |
| `settings`          | PK is a named `key TEXT` (e.g. `"download_path"`). Intentional. Keep as-is. |

### 3. Foreign key columns that change type

| Table            | Column       | References     | New type        |
|------------------|--------------|----------------|-----------------|
| `sources`        | `creator_id` | `creators.id`  | `Option<i64>`   |
| `posts`          | `creator_id` | `creators.id`  | `i64`           |
| `posts`          | `source_id`  | `sources.id`   | `Option<i64>`   |
| `media`          | `post_id`    | `posts.id`     | `i64`           |
| `download_tasks` | `post_id`    | `posts.id`     | `Option<i64>`   |

---

## The `external_id` Pattern

Several tables currently use yt-dlp's own IDs as the primary key. These external IDs
must be preserved for dedup and linking, but moved to a dedicated column.

Add `external_id TEXT NULL` to: `creators`, `posts`, `sources`.

**Dedup logic must change:**
- `creators`: dedup by `UNIQUE(platform_id, external_id)` instead of `id`
- `posts`: dedup by `UNIQUE(external_id)` instead of `id`
- `sources`: dedup by `UNIQUE(external_id)` for yt-dlp sourced entries

The `OnConflict` clauses in `metadata/store.rs` must be updated accordingly.

---

## The `creators` Table: Additional Changes

Beyond the integer PK, `creators` also needs two new columns per the architectural
decision in `docs/features/source_feed_architecture.md`:

1. `external_id TEXT NULL` — the platform's own creator ID (uploader_id, channel_id).
   `NULL` for personal account rows.

2. `is_self BOOLEAN NOT NULL DEFAULT FALSE` — marks rows representing the user's own
   authenticated platform accounts (not public creators). These rows are inserted when
   the user authenticates a platform. The UI renders them as "My Account" cards.
   They have `is_self = TRUE` and `external_id = NULL`.

---

## SeaORM-Specific Notes

- Integer autoincrement PKs in migrations: `integer(Col).auto_increment().primary_key()`
- In entity structs: `pub id: i64` with `#[sea_orm(primary_key, auto_increment = true)]`
- FK columns previously `String` must become `i64` or `Option<i64>` to match new PK types.
- `OnConflict` upsert clauses targeting the old string `id` must target the new composite
  unique key (e.g. `(platform_id, external_id)` for creators).
- All `Set(Uuid::new_v4().to_string())` calls for affected tables must be removed.
  Use `NotSet` for the `id` field on insert — SeaORM handles autoincrement automatically.
- `SourceResponse` and any other serializable structs in `commands/` that expose `id: String`
  to the frontend must update to `id: i64`.

---

## Files to Analyze and Modify

Analyze **every file** under `src-tauri/src/` systematically.
Key files known to require changes:

### Entities — rewrite field types
- `src-tauri/src/entity/creator.rs`
- `src-tauri/src/entity/post.rs`
- `src-tauri/src/entity/source.rs`
- `src-tauri/src/entity/media.rs`
- `src-tauri/src/entity/download_task.rs`

### Migrations — rewrite schema + check subsequent migrations
- `src-tauri/src/migration/m20260217_000001_initial_schema.rs` — full rewrite of affected tables
- `src-tauri/src/migration/m20260217_000002_add_download_stats.rs` — check for type references
- `src-tauri/src/migration/m20260219_000001_add_username_to_sessions.rs` — check for type refs
- `src-tauri/src/migration/m20260224_000001_add_avatar_and_error_to_sessions.rs` — check for type refs

### Business logic — update ID generation and upsert logic
- `src-tauri/src/metadata/store.rs` — upsert_creator, upsert_post, save_playlist
- `src-tauri/src/commands/sources.rs` — dedup check, SourceResponse, add_source_command
- `src-tauri/src/queue/manager.rs` — media row creation uses Uuid::new_v4()

### Scan all `.rs` files for
- `Uuid::new_v4()`
- `use uuid::Uuid`
- `String` type on id/FK fields in entity structs
- Any place an entity ID is passed as `String` to a command or Tauri event payload

### Dependency cleanup
After all changes, check `src-tauri/Cargo.toml`: if `uuid` is no longer used anywhere,
remove the dependency entirely.

---

## Expected Outcome

- Surrogate PKs on `creators`, `posts`, `sources`, `media`, `download_tasks` are `i64` autoincrement.
- External platform IDs (yt-dlp video IDs, channel IDs, playlist IDs) stored in `external_id TEXT`.
- Dedup logic uses `external_id` with UNIQUE constraints on composite keys.
- No `Uuid::new_v4()` calls remain for affected tables.
- `platform`, `platform_session`, and `setting` tables are untouched.
- Codebase compiles with no type mismatches between entity fields and usages.
- Frontend-facing command responses that expose entity IDs use `i64`, not `String`.
