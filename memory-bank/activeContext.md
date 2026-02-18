# Active Context

> [!IMPORTANT]
> When planning new features, ALWAYS consult `/docs` for the authoritative detailed requirements. The Memory Bank is a high-level summary and may not capture every edge case or UI element specified in the roadmap.

**Phase 3: Cookie / Auth Integration (Completed)**
We have successfully implemented robust authentication (L1/L2/L3) and fixed the `yt-dlp` cookie rejection issue by integrating **QuickJS** as a native sidecar for signature extraction. JSON cookie imports (L3) are also fully supported.

## Recent Changes
- **Deno Migration**: Replaced `quickjs` with `deno` as the JS runtime limit for `yt-dlp` to improve compatibility and performance.
- **Auth Fix (AppState Injection)**: Fixed critical bug in `get_auth_status` where `DatabaseConnection` was not correctly managed, causing "state not managed" errors.
- **JSON Cookie Support**: Added support for importing cookies in JSON format (List, Wrapper, Single) via manual import.
- **Persistent Logging**: Enabled daily log rotation with `tracing_appender`.
- **Backend**: Updated `fetcher.rs` and `worker.rs` to use `deno` runtime.

## Current State
- **Backend**: Auth system is stable. `CookieManager` handles encryption, JSON conversion, and temp file creation. `yt-dlp` uses the embedded Deno binary.
- **Frontend**: Connect Account UI is functional.
- **Next Focus**: Moving to Phase 4/5 (Frontend Polish & Gallery).

## Next Steps
1.  **Phase 4: Frontend Polish**:
    -   Active Downloads UI refinement (progress bars, speed, ETA).
    -   Settings Page (custom download path).
2.  **Phase 5: Gallery**:
    -   Implement Virtualized Grid for downloaded content.
3.  **Cleanup**: Verify Deno binary updates in production builds.
