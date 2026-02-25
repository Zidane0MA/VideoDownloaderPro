# Active Context

> [!IMPORTANT]
> When planning new features, ALWAYS consult `/docs` for the authoritative detailed requirements. The Memory Bank is a high-level summary and may not capture every edge case or UI element specified in the roadmap.

**Post-Audit: Revised Roadmap (2026-02-25)**
Deep technical audit revealed significant gaps between the Settings UI and backend integration. Four new intermediate phases (5.5–5.8) have been inserted before Phase 6.

## Recent Changes
- **Technical Audit**: Analyzed all backend and frontend files. Documented all disconnected settings, stubs, and missing features.
- **Media Viewer (Phase 5)**: Complete — custom video player, React Portal overlay, inter-post navigation.
- **Revised Roadmap**: Created Phases 5.5 (Settings Wiring), 5.6 (Format/Quality), 5.7 (Sources/Playlists), 5.8 (Trash/Lifecycle).

## Current State
- **Phase 5 (Wall + Media Viewer)**: ✅ Complete.
- **Settings Integration**: 🔴 `download_path`, `concurrent_downloads`, `trash_auto_clean_days` saved to DB but NOT read by backend.
- **Download Engine**: 🔴 Rate limit hardcoded (`5M`), playlists blocked (`--no-playlist`), format selection plumbed but no UI.
- **Source Entity**: 🔴 DB schema exists but completely unused.
- **Sidecar Updates**: 🟡 Backend `update_yt_dlp()` exists but no IPC command exposed; UI button is a stub.

## Next Steps
1. **Phase 5.5**: Wire all disconnected settings to backend. Start with `download_path` → worker and `concurrent_downloads` → Semaphore.
2. **Phase 5.6**: Add format/quality selection UI in the AddDownloadModal.
3. **Phase 5.7**: Enable playlist/channel support and Source CRUD.
