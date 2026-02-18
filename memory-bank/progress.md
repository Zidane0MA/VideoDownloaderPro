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
- [/] Cookie / Auth Integration (CookieManager)
    - [x] Frontend UI (ConnectAccountModal)
    - [x] Backend Commands (L1/L2)
    - [x] **Debug**: Fix `yt-dlp` rejection of cookies ("Sign in to confirm your age") only for L3
    - [x] **Deno Sidecar**: Embedded JS runtime (replaces QuickJS) for signature extraction
    - [x] **JSON Cookie Support**: Manual Import (L3) with format conversion
    - [x] **Verify L1 (WebView)**: Test full download flow
    - [ ] **Verify L2 (Browser)**: Test full download flow
    - [ ] **Verify L3 (Manual)**: Test full download flow

### Phase 4: Frontend - Download Manager (Partial)
- [x] Active Downloads UI
- [x] URL Input & Preview
- [ ] Settings Page (Advanced options missing: Path, etc.)

### Phase 5: Frontend - Gallery (Wall)
- [ ] Virtualized Grid
- [ ] Post Card Component
- [ ] Media Viewer

### Phase 6: Polish & Packaging
- [ ] Dark/Light Mode
- [ ] Error Handling UI
- [ ] Installer Build

## Known Issues / Blockers
*   **Auth Failure**: `yt-dlp` fails to use extracted cookies for age-gated content.
