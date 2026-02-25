# Active Context

> [!IMPORTANT]
> When planning new features, ALWAYS consult `/docs` for the authoritative detailed requirements. The Memory Bank is a high-level summary and may not capture every edge case or UI element specified in the roadmap.

**Phase 3: Cookie / Auth Integration (Completed)**
We have successfully implemented robust authentication (L1/L2/L3) and fixed the `yt-dlp` cookie rejection issue by integrating **Deno** as a native sidecar for signature extraction. JSON cookie imports (L3) are also fully supported.

## Recent Changes
- **Path Saving Fix**: Fixed a bug where yt-dlp's stdout encoding in Windows dropped unicode characters (like `⧸`), causing discrepancies between the database path and the actual file path. Enforced `PYTHONIOENCODING=utf-8` on process spawn.
- **Gallery Wall Feature**: Built a high-performance virtualized masonry grid using `@virtuoso.dev/masonry` to handle thousands of items seamlessly.
- **Media Pipeline**: Configured backend `ytdlp` commands (`worker.rs`) and `post_process.rs` to download and automatically resize optimal platform thumbnails.
- **Settings Page**: Added advanced settings UI with `Zustand` and native Rust backend syncing. Implemented download path selector using `tauri-plugin-dialog`, concurrent downloads slider, language toggle, and Trash auto-clean configuration.

## Current State
- **Backend**: Auth system is stable. `CookieManager` handles encryption, JSON conversion, and temp file creation. `yt-dlp` uses the embedded Deno binary.
- **Frontend**: Connect Account UI is functional with improved error handling for browser imports.
- **Authentication**: 
    -   **L1 (WebView)**: Verified & Working.
    -   **L2 (Browser Import)**: Working for Firefox; Restricted for Chrome/Edge (App-Bound Encryption). UX warnings added.
    -   **L3 (Manual)**: Verified & Working.
    -   **Validation**: Implemented (rejects guest cookies).
    -   **Display**: Username and Avatar showing for supported platforms (IG/TikTok/X).
- **Next Focus**: Completing Phase 5 with the Media Viewer component, then advancing to Phase 6 (Polish & Packaging).

## Next Steps
1.  **Phase 5: Media Viewer**:
    -   Implement the fullscreen media viewer when clicking a PostCard.
2.  **Phase 6: Polish**:
    -   Dark/Light Mode configurations.
    -   Global Error Handling UI.
2.  **Cleanup**: Verify Deno binary updates in production builds.
