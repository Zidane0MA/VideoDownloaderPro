# Active Context

**Phase 3: Download Engine (Rust)**
Phase 2 is complete. Database layer is set up with SQLite + Sea-ORM. Moving to Phase 3: building the download engine.

## Recent Changes
*   [x] **Phase 2 Complete:** SQLite database with Sea-ORM, 8 tables, migrations, entities, and Tauri managed state.
*   [x] **Sidecar Manager (Phase 3.1):** `sidecar/` module with `get_version()`, `update_yt_dlp()`, `check_all()`. Three IPC commands: `get_sidecar_status`, `get_sidecar_version`, `update_sidecar`. 4 unit tests for version parsing.

## Next Steps
1.  **Download Worker (Phase 3.3):** Execute yt-dlp downloads with progress tracking.
2.  **Queue System:** Background worker pool with concurrency management.
