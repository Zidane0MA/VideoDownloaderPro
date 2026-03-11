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
- **Source Feed Architecture (Phase 5.7)**: Complete — refactored backend to support multi-feed sources (Videos, Shorts, etc.) per creator. Implemented grouped Channel Cards and multi-select feed UI.
- **vdp:// Protocol**: Backend now intercepts and resolves `vdp://tiktok/me/` for authenticated personal feeds (Saved, Liked).

## Current State
- **Phase 1-5**: 🟢 **COMPLETED**. The application now has a fully working Wall Gallery, Media Viewer, Source CRUD, Settings UI, and Trash soft-delete lifecycle.
- **Settings Integration**: ✅ `download_path`, `concurrent_downloads`, `rate_limit`, and `update_yt_dlp` are all fully wired to the backend via IPC commands.
- **Format & Quality Selection**: ✅ Professional UI built using `ProcessedMetadata`. `DownloadOptions` JSON structure used for format selection in worker.
- **Download Engine**: ✅ Configurable rate limits and format resolutions fully work. Playlists and complex extractions via internal APIs (TikTok likes) are scaling up.
- **Source Architecture**: ✅ Foundational architecture complete. Grouped cards by `creator_id` with individual feed pills. Multi-feed creation logic handles partial unique indexing correctly via manual upsert.

## Next Steps
1. **Phase 6 (Polish & Packaging)**: Refine error handling UX, disk space checks, finalize Tauri build settings, and configure updater workflows.
2. **Phase 7 (Source Sync MVP)**: Implement the background worker scheduler to periodically poll active `sources` and download new posts into the timeline automatically.
