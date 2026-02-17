# Active Context

**Phase 3: Download Engine (Rust)**
Phase 2 is complete. Database layer is set up with SQLite + Sea-ORM. Moving to Phase 3: building the download engine.

## Recent Changes
*   [x] **Phase 2 Complete:** SQLite database with Sea-ORM, 8 tables, migrations, entities, and Tauri managed state.
*   [x] **Sidecar Manager (Phase 3.1):** `sidecar/` module with `get_version()`, `update_yt_dlp()`, `check_all()`.
*   [x] **Queue System (Phase 3.4):** Implemented `DownloadQueue` with semaphore concurrency (limit 3) and `tokio::sync::Notify`.

## Next Steps
1.  **Frontend Integration (Phase 4):** Connect React UI to `create_download_task` and listen for progress events.
2.  **Active Downloads UI:** Visualize the queue and progress.
