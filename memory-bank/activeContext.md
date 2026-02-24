# Active Context

> [!IMPORTANT]
> When planning new features, ALWAYS consult `/docs` for the authoritative detailed requirements. The Memory Bank is a high-level summary and may not capture every edge case or UI element specified in the roadmap.

**Phase 3: Cookie / Auth Integration (Completed)**
We have successfully implemented robust authentication (L1/L2/L3) and fixed the `yt-dlp` cookie rejection issue by integrating **Deno** as a native sidecar for signature extraction. JSON cookie imports (L3) are also fully supported.

## Recent Changes
- **Settings Page**: Added advanced settings UI with `Zustand` and native Rust backend syncing. Implemented download path selector using `tauri-plugin-dialog`, concurrent downloads slider, language toggle, and Trash auto-clean configuration.
- **Username & Avatar Extraction**: Added `username` and `avatar_url` columns to `platform_sessions` and implemented `UsernameFetcher` logic for TikTok and X (Twitter), alongside Instagram.
- **Deno Migration**: Replaced `quickjs` with `deno` as the JS runtime for `yt-dlp` to improve compatibility and performance.
- **Session Validation**: Implemented mandatory cookie checks (`auth_token`, `sessionid`) to prevent false-positive logins.

## Current State
- **Backend**: Auth system is stable. `CookieManager` handles encryption, JSON conversion, and temp file creation. `yt-dlp` uses the embedded Deno binary.
- **Frontend**: Connect Account UI is functional with improved error handling for browser imports.
- **Authentication**: 
    -   **L1 (WebView)**: Verified & Working.
    -   **L2 (Browser Import)**: Working for Firefox; Restricted for Chrome/Edge (App-Bound Encryption). UX warnings added.
    -   **L3 (Manual)**: Verified & Working.
    -   **Validation**: Implemented (rejects guest cookies).
    -   **Display**: Username and Avatar showing for supported platforms (IG/TikTok/X).
- **Next Focus**: Moving entirely to Phase 5 (Gallery `Wall`).

## Next Steps
1.  **Phase 5: Gallery**:
    -   Implement Virtualized Grid for downloaded content using `@tanstack/react-virtual`.
    -   Build Post Card component (thumbnail, creator, stats).
    -   Implement Media Viewer.
2.  **Cleanup**: Verify Deno binary updates in production builds.
