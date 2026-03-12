# VideoDownloaderPro Implementation Context - Debugging Handover

This document summarizes the changes made to fix the download limit bug and the subsequent performance optimizations/UI virtualization.

## 1. Backend: Download Limit & DB Performance
- **Sources API**: Updated `src-tauri/src/commands/sources.rs` (`AddSourceRequest`) to accept `limit_mode` and `max_items`.
- **yt-dlp Integration**: `src-tauri/src/metadata/fetcher.rs` now passes `--playlist-items 1-{limit}` to the sidecar, preventing excessive fetching at the source.
- **TikTok Fetcher**: `src-tauri/src/metadata/tiktok/mod.rs` respects `max_items` to stop cursor-based fetching early.
- **Database Chunking**: `src-tauri/src/metadata/store.rs` refactored `save_playlist` to commit every 50 videos in separate transactions. This fixed the "database is locked" errors during large playlist imports.
- **Queue Batching**: `src-tauri/src/commands/sources.rs` (`queue_posts`) now uses `insert_many` for `download_task` to avoid N insertions.

## 2. Frontend: Performance & Virtualization
- **State Management**: Added `expandedGroups` Record to `src/store/downloadStore.ts` and `toggleGroup` action.
- **Memoization**: `DownloadItem.tsx` is wrapped in `React.memo` to stop re-renders on active progress updates of other items.
- **Hook Optimization**: `useDownloadManager.ts` uses a memoized `taskKeys` string to prevent re-sorting the 600+ tasks on every Progress event (which fires multiple times a second).
- **Virtualization**: Replaced `GroupedTaskList` in `DownloadsList.tsx` with `GroupedVirtuoso`.

## 3. Current "Broken" State (Invisible List)
Despite having 600+ items (confirmed by the badge counts), the list is rendering as empty or 0-height.

### Potential Root Causes:
1. **Container Height Collapse**: `GroupedVirtuoso` (and Virtuoso in general) renders nothing if its parent has 0px height. Even with `useWindowScroll`, the component needs to be correctly positioned in the DOM flow.
2. **Flattening Logic Bug**: In `DownloadsList.tsx`, `flattenedGroups`, `groupCounts`, and `allItems` are calculated in a `useMemo`. If `expandedGroups` isn't initialized correctly or the logic for `-1` sourceId group (standalone) is failing, the indices might mismatch.
3. **Virtuoso Layout**: `useWindowScroll` works well when the list is the main content of the page. If it's deeply nested in Tailwind flex/overflow containers, Virtuoso might lose the scroll context.

### Component References for next agent:
- `src/components/DownloadsList.tsx`: Main virtualization entry point.
- `src/components/PlaylistGroup.tsx`: Refactored to be a controlled component (props `isExpanded`, `onToggle`).
- `src/hooks/useDownloadManager.ts`: Provides the `tasks` and `toggleGroup` logic.

## 4. Pending Tasks
- Fix the visibility of `GroupedVirtuoso` items.
- Verify `itemContent` index mapping in `GroupedVirtuoso` matches the `allItems` array structure.
