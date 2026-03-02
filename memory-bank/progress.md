# Progress Status

## Roadmap Overview

### Phase 1: Setup & Core Structure (Completed)
- [x] Initialize Tauri v2 Project
- [x] Setup Frontend Dependencies (Tailwind v3, Zustand, Query, i18n)
- [x] Setup Backend Dependencies (Tracing, Tokio, Shell)
- [x] Configure Sidecars (yt-dlp, ffmpeg, permissions)

### Phase 2: Database & Core Models (Completed)
- [x] Schema Migration
- [x] Sea-ORM Entities
- [x] Basic Tests

### Phase 3: Download Engine (Rust) (Partial)
- [x] Sidecar Manager (Update/Copy)
- [x] Metadata Fetcher
- [x] Download Worker
- [x] Queue System
- [x] Cookie / Auth Integration (CookieManager)
    - [x] Frontend UI (ConnectAccountModal)
    - [x] Backend Commands (L1/L2)
    - [x] **Debug**: Fix `yt-dlp` rejection of cookies ("Sign in to confirm your age") only for L3
    - [x] **Debug**: Fix Unicode filepath corruption on Windows by migrating to native filesystem scans
    - [x] **Deno Sidecar**: Embedded JS runtime (replaces QuickJS) for signature extraction
    - [x] **JSON Cookie Support**: Manual Import (L3) with format conversion
    - [x] **Verify L1 (WebView)**: Working (Fixed "Not Connected" & "Channel Closed" bugs).
    - [/] **Verify L2 (Browser)**: Working for Firefox. Chrome/Edge restricted by App-Bound Encryption (UX limitations added).
    - [x] **Verify L3 (Manual)**: Working.
    - [x] **Validation**: Fix False Positive Auth (Empty/Guest Cookies)
    - [x] **Feature**: Display Username & Avatar in Account Card (Extraction for IG/TikTok/X)

### Phase 4: Frontend - Download Manager (Partial - *Note: Metadata Preview and Quality Selection deferred*)
- [x] Active Downloads UI
- [x] URL Input & Preview
- [x] Settings Page (Path, Concurrency, Language, Trash Config)
- [ ] Disk Space Dashboard: Docs state Phase 4 includes a dashboard in Settings. Status: Currently a non-functional stub.
- [x] Metadata Preview: UI now shows thumbnail, title, uploader, and duration when adding a download.
- [x] Quality Selector: Implemented as part of Phase 5.6.

### Phase 5: Frontend - Gallery (Wall) (Partial - *Note: Search & Filter deferred*)
- [x] Virtualized Grid (via @virtuoso.dev/masonry)
- [x] Post Card Component
- [x] Media Viewer
    - [x] Two-pane layout (Player + Metadata Sidebar) with React Portal
    - [x] Custom Video Player: auto-hiding controls, seekbar, volume, fullscreen
    - [x] Hold-to-speedup (2x) with top badge indicator
    - [x] Inter-post navigation (keyboard + hover-reveal edge arrows)
- [ ] Creator Avatars: Docs describe the Gallery cards with creator avatars. Status: Entity has the field, but it is never populated/fetched.
- [ ] Search & Filter: Docs claim the Wall supports filtering by title, creator, platform. Status: Logic is missing.
- [ ] Metadata Dump: Docs state Phase 5 includes storing raw yt-dlp metadata. Status: Missing.
- [ ] Subfolder Organization: Docs specify /downloads/{Platform}/{Creator or Source}/. Status: Code downloads all files into a flat directory.
- [ ] Filename Template: Docs describe a customizable filename template. Status: Missing.
- [ ] Trash Auto Clean: Docs claim the Trash view supports auto-cleaning. Status: Missing.
- [ ] Delete Files on Remove: Docs claim the Trash view supports auto-cleaning. Status: Missing.

### Phase 5.5: Settings Integration & Engine Polish
- [x] Wire `download_path` DB setting to download worker (currently hardcoded to system default)
- [x] Wire `concurrent_downloads` DB setting to Queue Semaphore (now supports live-reload via watch channel)
- [ ] Make `rate_limit` configurable (currently hardcoded `--limit-rate 5M`)
- [ ] Expose `update_yt_dlp()` as IPC command; wire "Check for Updates" button
- [ ] Persist theme preference to DB; wire CSS class/variable swap

### Phase 5.6: Format & Quality Selection (Completed)
- [x] Extend `YtDlpFormat` to capture comprehensive format data (fps, codecs, HDR, audio channels, etc.)
- [x] Extend `AddDownloadModal` to show professional quality presets, advanced format grid, audio track selector, and subtitle toggles.
- [x] Pass `DownloadOptions` (JSON) through UI → `create_download_task`
- [x] Update download worker to parse `DownloadOptions` and apply correct yt-dlp flags (audio-only, subtitles, container override).

### Phase 5.7: Sources & Playlists
- [ ] Remove `--no-playlist` hardcode; add playlist expansion logic
- [ ] Build CRUD IPC commands for `source` entity (channels, playlists, creator profiles)
- [ ] Build frontend "Sources" section (list, add, sync trigger)
- [ ] Link `post.source_id` when downloading from a source

### Phase 5.8: Trash & Lifecycle
- [ ] Implement soft-delete (`deleted_at` instead of physical delete)
- [ ] Add "Trash" view in frontend for soft-deleted items
- [ ] Implement `trash_auto_clean` background job (periodic at startup)
- [ ] Add `delete_files_on_remove` toggle in Settings

- [ ] Phase 5.x: Infrastructure Gaps (Identified in Documentation Audit)
    - [ ] `media.checksum` (SHA-256) for duplicate detection
    - [ ] `posts.raw_json` metadata dump storage
    - [ ] Structured Error Codes (`NET_001`, etc.)
    - [ ] Pre-download Health Checks (disk, net, path)
    - [ ] Proactive 24h Cookie Health Checks
    - [ ] Per-task logging (`task-{uuid}.log`)
    - [ ] Subfolder organization (`/downloads/Platform/Creator/`)
    - [ ] Filename template customization

### Phase 6: Polish & Packaging
- [ ] Dark/Light Mode (CSS variables, persistent theme)
- [ ] Global Error Handling UI
- [ ] Installer Build (MSI, AppImage)

*   **Browser Encryption**: L2 (Browser Import) is limited on Chrome/Edge due to App-Bound Encryption.
*   **Documentation Discrepancies**: Many features in Phase 4/5 (Metadata Preview, Quality Selector, Search) and the Infrastructure (Health checks, Checksums, Hierarchy) are documented but missing from code.
*   **IPC Contract Drift**: The documented API in `07_ipc_api_contract.md` significantly exceeds the implemented commands and events.
*   **Source Entity Unused**: `sources` table exists with full schema but zero logic references it.
