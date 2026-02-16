# Development Roadmap

## Phase 1: Setup & Core Structure
**Goal:** Project scaffold + dependencies + sidecar binaries.

1.  **Initialize Project:**
    -   Create Tauri v2 App (`npm create tauri-app@latest`).
    -   Select: `TypeScript`, `React`, `Vite`.
2.  **Frontend Dependencies:**
    -   `tailwindcss`, `lucide-react` (icons), `zustand` (state), `@tanstack/react-query`.
    -   `react-i18next`, `i18next`, `i18next-browser-languagedetector`.
    -   `@tanstack/react-virtual` (virtualized lists).
3.  **Backend Dependencies (Cargo):**
    -   `sea-orm` + `sea-orm-migration` (ORM + migrations).
    -   `serde`, `serde_json` (serialization).
    -   `tokio` (async runtime).
    -   `tracing`, `tracing-subscriber`, `tracing-appender` (logging).
    -   `uuid` (ID generation).
    -   `sha2` (checksum for duplicate detection).
4.  **Sidecar Binaries:**
    -   Download `yt-dlp.exe` and `ffmpeg.exe`.
    -   Place in `src-tauri/binaries/` with target-triple naming.
    -   Configure `tauri.conf.json` permissions.
5.  **i18n Setup:**
    -   Create `/src/locales/en.json` and `/src/locales/es.json`.
    -   Configure `i18next` with browser language detection.
6.  **Testing Foundation:**
    -   Set up Rust unit tests (`cargo test`).
    -   Set up basic frontend tests (Vitest).

## Phase 2: Database & Core Models
**Goal:** Schema, migrations, and data layer.

1.  **Schema Migration:**
    -   Create initial migration defining all tables: `platforms`, `creators`, `sources`, `posts`, `media`, `download_tasks`, `settings`.
    -   Initialize SQLite connection on app launch.
    -   Seed `platforms` table (YouTube, TikTok, Instagram, X).
    -   Seed `settings` table with defaults.
2.  **Sea-ORM Entities:**
    -   Generate entity files from migration.
    -   Implement CRUD operations for each table.
3.  **Tests:**
    -   Unit tests for all Sea-ORM queries.
    -   Test cascade/soft-delete behavior.

## Phase 3: The Download Engine (Rust)
**Goal:** yt-dlp command wrapper + download queue.

1.  **Sidecar Manager:**
    -   Runtime copy of `yt-dlp.exe` from bundled to `app_data/binaries/`.
    -   Version check + auto-update via `yt-dlp -U`.
    -   Same for `ffmpeg.exe`.
2.  **Metadata Fetcher:**
    -   Implement Rust function to execute `yt-dlp --dump-json`.
    -   Parse JSON output into typed structs (Creator, Title, Formats, Thumbnails).
3.  **Download Worker:**
    -   Spawn `yt-dlp` process with format + cookie flags.
    -   Parse `[download]` progress lines via regex.
    -   Emit `download-progress` events to frontend.
    -   Handle `--write-thumbnail --convert-thumbnails jpg`.
4.  **Download Queue:**
    -   Worker pool (N configurable workers, default 3).
    -   FIFO scheduling with priority (manual > sync).
    -   State machine: QUEUED → FETCHING_META → READY → DOWNLOADING → COMPLETED/FAILED.
    -   Pause: kill yt-dlp process, set status to PAUSED.
    -   Resume: restart with `-c` (continue partial download).
    -   Cancel: kill process, clean up partial files.
    -   Retry: exponential backoff, respects `max_retries`.
5.  **Thumbnail Processing:**
    -   After download: spawn `ffmpeg` to generate 300px thumbnail for Wall.
6.  **Duplicate Detection:**
    -   Pre-download: check `original_url` in `posts` table.
    -   Post-download: compute SHA-256, check against `media.checksum`.
7.  **Cookie / Auth Integration (CookieManager):**
    -   **CookieManager module:** Central Rust module for all cookie operations.
    -   Read/write `platform_sessions` table (encrypted cookies per platform).
    -   **Session storage:** Encrypt cookies with Windows DPAPI, store in `app_data/auth/{platform}.cookies.enc`.
    -   **Cookie health check:** Periodic verification of stored cookies (test request). Update `last_verified` and `status` in `platform_sessions`.
    -   **Temp cookie file:** Generate temporary `cookies.txt` (Netscape format) for yt-dlp, delete after use.
    -   On restricted content error → check stored cookies → retry with `--cookies`.
    -   **Fallback chain:** Stored cookies → `--cookies-from-browser` → prompt user.
    -   **WebView login support:** Open secondary WebView window, capture cookies via `Webview::cookies_for_url()` after user login.
8.  **Tests:**
    -   Unit tests for yt-dlp output parsers (progress, metadata).
    -   Unit tests for queue state machine transitions.
    -   Integration test: URL → metadata → download → DB entries.

## Phase 4: Frontend — Download Manager UI
**Goal:** Active downloads view with full control.

1.  **Download Bar / Panel:**
    -   List of active/queued/completed downloads.
    -   Per-item: progress bar, speed, ETA, status badge.
    -   Actions: Pause, Resume, Cancel, Retry, Remove.
2.  **URL Input:**
    -   Paste URL → show metadata preview (title, thumbnail, creator, formats).
    -   Format/quality selector dropdown.
    -   "Download" button.
3.  **IPC Integration:**
    -   Subscribe to `download-progress` events (TanStack Query + Tauri events).
    -   Optimistic updates on Pause/Cancel actions.
4.  **Settings Page:**
    -   Download path selector (native file picker).
    -   Concurrent downloads slider (1-10).
    -   **Accounts section:**
        -   Per-platform session status indicators (ACTIVE ● / EXPIRED ○ / NONE ○).
        -   "Connect" / "Disconnect" buttons per platform.
        -   Cookie method selector: WebView (recommended) / Browser / File import.
        -   Browser selector dropdown (Chrome/Firefox/Edge) for fallback method.
    -   yt-dlp update button + version display.
    -   Language selector (EN/ES).
    -   Disk space dashboard.
    -   Trash auto-clean configuration.

## Phase 5: Frontend — Gallery (Wall Mode)
**Goal:** The "Wall of Content" — core differentiator.

1.  **Wall Layout:**
    -   Masonry grid with virtualized rendering (`@tanstack/react-virtual`).
    -   Lazy thumbnail loading with `IntersectionObserver`.
    -   Paginated backend queries (`LIMIT/OFFSET` with cursor).
2.  **Post Card Component:**
    -   Thumbnail (reduced 300px version for performance).
    -   Creator avatar + name.
    -   Title, platform badge, date.
    -   Media count indicator for carousels.
3.  **Post Detail View:**
    -   Full-size media viewer (video player / image gallery).
    -   Media carousel navigation.
    -   Metadata: description, original URL, file sizes, download date.
    -   Actions: Open file, open in browser, delete.
4.  **Trash View:**
    -   List of soft-deleted posts.
    -   Restore / Permanently Delete actions.
    -   "Empty Trash" bulk action.
5.  **Source Configuration (Documented, post-MVP):**
    -   UI Modal to add a Source (Channel/Playlist URL).
    -   Select Sync Mode: `Everything`, `New Only`, `Date Range`, `Last N`.

## Phase 6: Polish & Packaging
**Goal:** Production-ready build.

1.  **UI Refinement:**
    -   Dark mode (primary), light mode toggle.
    -   Smooth transitions, skeleton loaders.
    -   Custom title bar.
    -   Responsive layout adjustments.
2.  **Error Handling:**
    -   Graceful failure UX for all error categories (see `06_error_handling.md`).
    -   Toast notifications for non-blocking errors.
    -   Modal alerts for blocking errors (disk full, auth required).
    -   **Auth-required modal:** Clear message explaining why login is needed, with "Login in browser" primary button opening WebView, and advanced options (browser extraction, file import) in collapsible section.
    -   **WebView login window:** Secondary window rendering platform login page, with footer explaining cookie storage and privacy.
3.  **Disk Space Management:**
    -   Pre-download space check.
    -   Low-space warning banner.
    -   Storage dashboard in Settings.
4.  **Logging:**
    -   Ensure all operations are logged via `tracing`.
    -   Daily consolidated log file (`app.log`).
    -   Log viewer in Settings (optional, post-MVP).
5.  **Build & Distribution:**
    -   Update `tauri.conf.json` identifier + permissions.
    -   Configure Tauri updater plugin (auto-update for the app).
    -   Run `npm run tauri build`.
    -   Test `.exe` installer on clean Windows environment.
    -   Test portable `.exe` variant.

## Phase 7: Source Sync (Post-MVP)
**Goal:** Automated monitoring and downloading of new content.

1.  **Sync Scheduler:**
    -   Background task that checks active sources on configurable interval.
    -   Uses `yt-dlp` playlist extraction to get remote IDs.
    -   **Deduplication:** Compares remote IDs vs DB. Queues only new items.
2.  **Sync Modes Implementation:**
    -   `ALL`: Download entire channel/playlist archive.
    -   `FROM_NOW`: Track `last_checked`, only download newer items.
    -   `DATE_RANGE`: Filter by `posted_at` range.
    -   `LATEST_N`: Download only the N most recent items.
3.  **Sync UI:**
    -   Source list with status indicators (syncing, idle, error).
    -   Manual "Sync Now" button per source.
    -   Sync history/log.
