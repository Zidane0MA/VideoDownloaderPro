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

### Phase 4: Frontend - Download Manager (Completed)
- [x] Active Downloads UI
- [x] URL Input & Preview
- [x] Settings Page (Path, Concurrency, Language, Trash Config)

### Phase 5: Frontend - Gallery (Wall) (Completed)
- [x] Virtualized Grid (via @virtuoso.dev/masonry)
- [x] Post Card Component
- [x] Media Viewer
    - [x] Two-pane layout (Player + Metadata Sidebar) with React Portal
    - [x] Custom Video Player: auto-hiding controls, seekbar, volume, fullscreen
    - [x] Hold-to-speedup (2x) with top badge indicator
    - [x] Inter-post navigation (keyboard + hover-reveal edge arrows)

### Phase 5.5: Settings Integration & Engine Polish
- [x] Wire `download_path` DB setting to download worker (currently hardcoded to system default)
- [ ] Wire `concurrent_downloads` DB setting to Queue Semaphore (currently hardcoded to `3`)
- [ ] Make `rate_limit` configurable (currently hardcoded `--limit-rate 5M`)
- [ ] Expose `update_yt_dlp()` as IPC command; wire "Check for Updates" button
- [ ] Persist theme preference to DB; wire CSS class/variable swap

### Phase 5.6: Format & Quality Selection
- [ ] Extend `AddDownloadModal` to show available formats/qualities (from metadata fetcher)
- [ ] Add "Best Auto" mode + manual quality picker dropdown
- [ ] Pass `format_selection` through UI → `create_download_task`

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

### Phase 6: Polish & Packaging
- [ ] Dark/Light Mode (CSS variables, persistent theme)
- [ ] Global Error Handling UI
- [ ] Installer Build (MSI, AppImage)

## Known Issues / Blockers
*   **Browser Encryption**: L2 (Browser Import) is limited on Chrome/Edge due to App-Bound Encryption.
*   **Settings Disconnected**: `download_path`, `concurrent_downloads`, `trash_auto_clean_days` are stored in DB but NOT read by the backend (see audit report).
*   **Source Entity Unused**: `sources` table exists with full schema but zero logic references it.
