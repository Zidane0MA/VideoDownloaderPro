# Source & Feed Architecture

This document defines the finalized architectural decisions for how `VideoDownloaderPro`
models multi-feed sources across platforms. It supersedes the exploratory ideas in
`source_intent_architecture.md`.

---

## Core Concepts

### The Two-Dimension Problem

A `source` must answer two independent questions simultaneously:

| Dimension     | Field         | Examples                                        |
|---------------|---------------|-------------------------------------------------|
| What IS it?   | `source_type` | `CHANNEL`, `PLAYLIST`, `SAVED`, `LIKED`         |
| Which feed?   | `feed_type`   | `VIDEOS`, `SHORTS`, `STREAMS`, `REELS`, `POSTS` |

Previously, `source_type` attempted to encode both. This caused the feed-selection
dilemma. Splitting them into two columns resolves it cleanly without new tables.

`source_type` answers **what kind of entity** the source is:

| Value      | Meaning                                            |
|------------|----------------------------------------------------|
| `CHANNEL`  | A public creator profile with sub-feeds             |
| `PLAYLIST` | A standalone playlist (no sub-feeds)                |
| `SAVED`    | Authenticated user's saved/bookmarked collection    |
| `LIKED`    | Authenticated user's liked/favorited collection     |

`feed_type` answers **which content stream** within a channel to sync:

| Value     | Meaning                          |
|-----------|----------------------------------|
| `VIDEOS`  | Standard long-form videos        |
| `SHORTS`  | Short-form vertical content      |
| `STREAMS` | Live streams / VODs              |
| `REELS`   | Instagram Reels                  |
| `POSTS`   | Static image/text posts          |

> [!IMPORTANT]
> `feed_type` is exclusively for sub-feeds of a `CHANNEL`. Sources of type
> `SAVED`, `LIKED`, or `PLAYLIST` always have `feed_type = NULL` because they
> represent a single collection, not a multi-feed profile.

### Platform ↔ Feed Type Compatibility Matrix

Not all platforms support all feed types. This matrix defines which `feed_type`
values are valid per platform, and drives the UI feed selector pills:

| Platform    | Available `feed_type` values             |
|-------------|------------------------------------------|
| YouTube     | `VIDEOS`, `SHORTS`, `STREAMS`            |
| Instagram   | `REELS`, `POSTS`                         |
| TikTok      | `VIDEOS`                                 |
| X (Twitter) | `POSTS`                                  |

This matrix is defined as a Rust constant map and mirrored in the frontend
`platformContexts` config. When adding a new platform, both must be updated.

---

## Database Schema Decision

### Migration: Add `feed_type` column

```sql
ALTER TABLE sources ADD COLUMN feed_type TEXT NULL;
```

`feed_type` is `NULL` for sources that are not sub-feeds of a channel
(e.g. standalone playlists, saved collections without a parent channel).


### Rust Constants

Define feed and source types as constants to avoid magic strings throughout the codebase:

```rust
pub mod source_type {
    pub const CHANNEL:  &str = "CHANNEL";
    pub const PLAYLIST: &str = "PLAYLIST";
    pub const SAVED:    &str = "SAVED";
    pub const LIKED:    &str = "LIKED";
}

pub mod feed_type {
    pub const VIDEOS:  &str = "VIDEOS";
    pub const SHORTS:  &str = "SHORTS";
    pub const STREAMS: &str = "STREAMS";
    pub const REELS:   &str = "REELS";
    pub const POSTS:   &str = "POSTS";
}
```

---

## How Sources Are Stored

### Example: YouTube channel with Videos + Shorts (no Streams)

Two rows in `sources`, where `creator_id` points to the numeric ID of the channel. Streams simply does **not exist** as a row —
it is not disabled, it is absent. This is intentional: absent feeds produce no overhead
in sync queries, no UI clutter, and no ambiguous `is_active` states.

```
id: 11  creator_id: 1  source_type: CHANNEL  feed_type: VIDEOS   is_active: true
id: 12  creator_id: 1  source_type: CHANNEL  feed_type: SHORTS   is_active: true
```

### Example: Standalone playlist

```
id: 15  creator_id: 1  source_type: PLAYLIST  feed_type: NULL  is_active: true
```

### Example: Authenticated personal feeds (via vdp://)

Personal feeds reference a **real `creators` row** with `is_self = true`.
This row is auto-inserted when a platform session is first activated (login flow),
and uses the session's username/avatar for display. 
The creator's `external_id` can be used to capture their actual platform user ID.

```sql
-- Inserted automatically on first TikTok login:
INSERT INTO creators (platform_id, is_self, name, url)
VALUES ('tiktok', 1, 'Mi cuenta TikTok', 'vdp://tiktok/me');
-- Suppose it gets id = 2
```

```
id: 20  creator_id: 2  source_type: SAVED  feed_type: NULL  url: vdp://tiktok/me/saved
id: 21  creator_id: 2  source_type: LIKED  feed_type: NULL  url: vdp://tiktok/me/liked
```

> [!NOTE]
> The `is_self = true` property is a convention the UI detects to render a "My Account"
> card with the platform session's avatar. The `creators.name` field is a static
> fallback; the UI should prefer `platform_sessions.username` for the display name.


---

## The `vdp://` Protocol

`vdp://` is reserved exclusively for sources that:

1. Require an authenticated session (cookies) to access.
2. Do **not** have a stable public URL independent of the logged-in user.

```
vdp://tiktok/me/saved     → My TikTok saved videos  (requires TikTok session)
vdp://instagram/me/liked  → My Instagram liked posts (requires Instagram session)
```

`vdp://` is **not** used for multi-feed encoding. A channel's public feeds
(`@mkbhd/videos`, `@mkbhd/shorts`) always use their real URLs. This keeps
the protocol narrow, auditable, and easy to maintain.

The `add_source_command` resolver expands `vdp://` into a real URL or a
platform-specific API call before saving to the DB.

---

## Source Creation Flow

### Adding a public channel (multi-feed)

When a user adds a channel URL (e.g. `youtube.com/@mkbhd`), the UI presents a
feed selector powered by Gemini to detect available feeds for the platform. The
user selects which feeds to subscribe to (e.g. Videos ✓, Shorts ✓, Streams ✗).

The backend creates **N rows** in `sources` — one per selected feed — within a
single transaction to ensure atomicity.

### Dedup Strategy

Dedup checks must operate on the **`(creator_id, feed_type)` pair**, not only
on the URL:

```
User adds @mkbhd/videos  → platform_id: youtube, external_id: mkbhd, feed_type: VIDEOS → ✅ Created
User adds @mkbhd/shorts  → platform_id: youtube, external_id: mkbhd, feed_type: SHORTS → ✅ Created (different feed)
User adds @mkbhd/videos  → platform_id: youtube, external_id: mkbhd, feed_type: VIDEOS → ❌ Duplicate rejected
```

For personal feeds, dedup is on `(platform_id, source_type)` since there is only
one "my saved" per platform:

```
User adds vdp://tiktok/me/saved  → platform: tiktok, type: SAVED → ✅ Created
User adds vdp://tiktok/me/saved  → platform: tiktok, type: SAVED → ❌ Duplicate rejected
```

> [!TIP]
> This means the current URL-based dedup in `add_source_command` needs to be
> replaced with a composite key check constraint handled by `ON CONFLICT` in the DB. The unique constraint is
> `(platform_id, external_id)` for channels.

### Adding feeds to an existing channel

If the user already has `@mkbhd/videos` and later wants to add Shorts, the UI
should detect the existing creator and present only the unsubscribed feeds.
This is purely a UI concern — the backend simply creates the new row.

---

## Sync Worker Strategy

### MVP: Independent row iteration

The sync worker queries all active source rows and processes them sequentially.
Each row is an independent unit with its own URL, `last_checked`, and `sync_mode`.
No grouping by `creator_id` occurs at the worker level.

### Long-term: Dual Queue with Domain Throttling

The sync architecture is designed to evolve into a **dual-queue** pattern:

| Queue             | Responsibility                         | Priority |
|-------------------|----------------------------------------|----------|
| **Metadata Queue**| Profile updates, feed pagination, link extraction | High (UI-bound) |
| **Binary Queue**  | yt-dlp downloads, file I/O              | Low (bandwidth-bound) |

A **Global QoS Controller** sits above both queues and manages:
- **Rate Limiters** — Token buckets per domain (e.g. max 1 req/s to TikTok API)
- **Circuit Breaker** — Backs off on 429/403 errors per domain
- **Session Locks** — Mutex per platform to prevent cookie collision when multiple
  sources from the same platform are syncing concurrently

> [!NOTE]
> The dual-queue architecture is a future concern. The MVP sync worker operates
> as a simple sequential loop. This section documents the target architecture
> so implementation decisions today don't block the migration path.

---

## UI: Vista Sources

The Sources view groups rows **by `creator_id`**. This is a UI concern, not a
schema concern — the DB stores individual feed rows; the frontend assembles the card.

### Channel card anatomy

```
┌──────────────────────────────────────────────┐
│ 🎬  MKBHD                          YouTube   │
│     [📹 Videos] [📱 Shorts]                  │
│     Last sync: 2h ago · 1,240 posts          │
└──────────────────────────────────────────────┘
```

- Each **pill** maps to one `source` row with its own `is_active`, `last_checked`,
  and `sync_mode`. Clicking a pill toggles `is_active` for that feed only.

### Personal feed card anatomy

Sources with `creator_id` matching the `__self_` prefix render as personal
account cards. The display name and avatar are resolved from `platform_sessions`,
not from the `creators` row:

```
┌──────────────────────────────────────────────┐
│ 👤  @juan_real (Mi Cuenta)           TikTok   │
│     [💾 Saved] [❤️ Liked]                     │
│     Last sync: 5h ago · 320 posts            │
└──────────────────────────────────────────────┘
```

If the user changes their TikTok username, the card updates automatically on
next session refresh — no migration needed because `is_self = true` explicitly
ties the record to session credentials, bypassing username changes.

### Standalone cards

Sources with `feed_type = NULL` and `is_self = false` (playlists) render as
standalone cards with no pills.


---

## Known Limitation: Singular `post.source_id` and Cross-Feed Duplicates

### The Problem

`post.source_id` is a single nullable foreign key. A post can only be attributed
to one source at a time. The same video can legitimately be discovered through
multiple sources:

- A creator's video appears in their `VIDEOS` feed **and** in your personal `LIKED` feed.
- A video is found via a `PLAYLIST` source **and** later via a `CHANNEL/VIDEOS` source.

Currently, `upsert_post` overwrites `source_id` on conflict, meaning the last
sync to encounter the post "wins" attribution. This is lossy.

### Intended Resolution Strategy

**Deduplication rule (sync worker):** If a post already has `status = COMPLETED`,
the sync worker **must not** create a new `download_task` for it, regardless of
which source triggered the sync. The file is already on disk — no action needed.
This is the only behavior required for MVP correctness.

**Attribution rule (conservative):** The `source_id` of a `COMPLETED` post is
**not overwritten** during upsert. The first source that downloaded the post
retains attribution. This preserves history and avoids confusing the UI.

**Long-term: `post_sources` join table**

True multi-attribution requires replacing the singular `post.source_id` with a
`post_sources` many-to-many join table:

```sql
CREATE TABLE post_sources (
    post_id   TEXT NOT NULL REFERENCES posts(id),
    source_id TEXT NOT NULL REFERENCES sources(id),
    discovered_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (post_id, source_id)
);
```

This enables:
- Querying "which sources contributed this post".
- A **Duplicates management view** where users can re-attribute or review
  posts discovered through multiple feeds.
- Accurate per-source post counts in the UI.

> [!NOTE]
> `post_sources` is deferred until there is a concrete UI requirement for
> cross-feed attribution. The MVP sync worker uses the conservative attribution
> rule above. When `post_sources` is introduced, `post.source_id` becomes
> a derived "primary source" field and the migration path is straightforward.
