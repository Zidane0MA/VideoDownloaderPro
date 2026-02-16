# Active Context

**Phase 2: Database & Core Models**
We have successfully initialized the project (Phase 1). Now we are moving to Phase 2: setting up the SQLite database and Sea-ORM entities.

## Recent Changes
*   [x] **Phase 1 Complete:** Initialized Tauri v2 + React + TypeScript project.
*   [x] **Dependencies:** Installed TailwindCSS v3, Zustand, TanStack Query, i18n, Lucide (frontend) and Tokio, Tracing, Shell (backend).
*   [x] **Sidecars:** Downloaded `yt-dlp` and `ffmpeg`, configured `tauri.conf.json` permissions and created download script.
*   [x] **Structure:** Created source folder structure for frontend features and backend commands.

## Next Steps
1.  **Database Setup:** Add `sea-orm` and `sea-orm-migration` dependencies.
2.  **Schema Definition:** Create the initial SQL migration for `posts`, `creators`, `settings`, etc.
3.  **Entity Generation:** Generate Rust entities from the schema.
4.  **Database Connection:** Initialize the SQLite connection in `main.rs`/`lib.rs`.
