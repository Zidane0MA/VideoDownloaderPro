# Active Context

**Phase 3: Download Engine (Rust)**
Phase 2 is complete. Database layer is set up with SQLite + Sea-ORM. Moving to Phase 3: building the download engine.

## Recent Changes
*   [x] **Phase 2 Complete:** SQLite database with Sea-ORM, 8 tables, migrations, entities, and Tauri managed state.
*   [x] **Migration:** Initial schema with platforms, creators, sources, posts, media, download_tasks, settings, platform_sessions.
*   [x] **Seed Data:** 4 platforms (YouTube, TikTok, Instagram, X) + 14 default settings.
*   [x] **Integration Test:** Verified tables, indexes, and seed data.

## Next Steps
1.  **Sidecar Manager:** Build update/copy logic for yt-dlp and ffmpeg.
2.  **Metadata Fetcher:** Parse yt-dlp JSON output into Post/Creator/Media entities.
3.  **Download Worker:** Execute yt-dlp downloads with progress tracking.
4.  **Queue System:** Background worker pool with concurrency management.
