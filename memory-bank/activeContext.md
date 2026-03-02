# Active Context

> [!IMPORTANT]
> When planning new features, ALWAYS consult `/docs` for the authoritative detailed requirements. The Memory Bank is a high-level summary and may not capture every edge case or UI element specified in the roadmap.

**Post-Audit: Revised Roadmap (2026-02-25)**
Deep technical audit revealed significant gaps between the Settings UI and backend integration. Four new intermediate phases (5.5–5.8) have been inserted before Phase 6.

## Recent Changes
- **Technical Audit**: Analyzed all backend and frontend files. Documented all disconnected settings, stubs, and missing features.
- **Documentation Audit**: Deep dive into `/docs` revealed significant "Phantom APIs" and infrastructure gaps (Checksums, Health Checks, Subfolder Org).
- **Media Viewer (Phase 5)**: Complete — custom video player, React Portal overlay, inter-post navigation.
- **Settings Integration (Phase 5.5)**: `download_path` and `concurrent_downloads` connected. Concurrency now supports **live-reload** via `tokio::sync::watch`.
- **Revised Roadmap**: Created Phase 5.x for Infrastructure Gaps and updated follow-up phases.

## Current State
- **Phase 4 & 5**: Marked COMPLETED but audit found missing "Integrated" features (Metadata preview, Quality selector, Search/Filter).
- **Settings Integration**: ✅ `download_path` and `concurrent_downloads` fully integrated. `trash_auto_clean_days` still disconnected.
- **Format & Quality Selection (Phase 5.6)**: ✅ Complete. Professional UI built using `ProcessedMetadata` for video qualities, audio tracks, and subtitles. `DownloadOptions` JSON structure used for format selection in worker.
- **Download Engine**: 🔴 Rate limit hardcoded (`5M`), playlists blocked (`--no-playlist`), Hierarchy/Checksums missing.
- **IPC API Contract**: 🔴 Major drift; documented commands/events missing from backend.
- **Source Entity**: 🔴 DB schema exists but completely unused.
- **Sidecar Updates**: 🟡 Backend `update_yt_dlp()` exists but no IPC command exposed; UI button is a stub.

## Next Steps
1. **Phase 5.5 (Remaining)**: Wire `rate_limit` (configurable) and `update_yt_dlp()` IPC command.
2. **Phase 5.7**: Enable playlist/channel support and Source CRUD.
3. **Phase 5.8**: Implement Trash view and soft-delete lifecycle.
