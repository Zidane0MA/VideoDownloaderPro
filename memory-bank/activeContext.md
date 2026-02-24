# Active Context

> [!IMPORTANT]
> When planning new features, ALWAYS consult `/docs` for the authoritative detailed requirements. The Memory Bank is a high-level summary and may not capture every edge case or UI element specified in the roadmap.

**Phase 3: Cookie / Auth Integration (Completed)**
We have successfully implemented robust authentication (L1/L2/L3) and fixed the `yt-dlp` cookie rejection issue by integrating **Deno** as a native sidecar for signature extraction. JSON cookie imports (L3) are also fully supported.

## Recent Changes
- **Username & Avatar Extraction**: Added `username` and `avatar_url` columns to `platform_sessions` and implemented `UsernameFetcher` logic for TikTok and X (Twitter), alongside Instagram.
- **Deno Migration**: Replaced `quickjs` with `deno` as the JS runtime for `yt-dlp` to improve compatibility and performance.
- **Auth Fixes**: Fixed `AppState` injection bug, resolved "Not Connected" / "Channel Closed" WebView errors, and added front-end UI warnings for App-Bound Encryption (L2 Chrome/Edge).
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
- **Next Focus**: Moving to Phase 4 (Frontend Polish) & Phase 5 (Gallery).

## Next Steps
1.  **Phase 4: Frontend Polish**:
    -   Active Downloads UI refinement (progress bars, speed, ETA).
    -   Settings Page (custom download path).
2.  **Phase 5: Gallery**:
    -   Implement Virtualized Grid for downloaded content.
3.  **Cleanup**: Verify Deno binary updates in production builds.
