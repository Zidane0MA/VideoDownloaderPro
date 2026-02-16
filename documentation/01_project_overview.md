# Project Overview: Video Downloader Pro

## 1. Goal
To build a high-performance, professional-grade desktop application for downloading and managing multi-media content from platforms like YouTube, TikTok, Instagram, and X.

This project serves two purposes:
1.  **Standalone Product:** A robust competitor to tools like *4K Video Downloader*, offering a unique "Wall of Content" experience.
2.  **Tech Demo / Module:** A proof-of-concept for a larger "Tags + AI + Management" ecosystem (**TagFlowPro**). The downloader module must be architected to be extractable and reusable.

## 2. Core Features

### A. Downloading
-   **Multi-Platform Support:** Powered by `yt-dlp` (covering 1000+ sites).
-   **Advanced Formats:** 4K/8K support, MP4/MKV/WebM containers, MP3/M4A audio extraction.
-   **Link Extraction:** Support for single videos, playlists, channels, and search queries.
-   **Carousel Support:** Properly handles multi-item posts (e.g., Instagram posts with 5 images/videos).
-   **Ephemeral Content:** Instagram Stories, YouTube Shorts, and any other content supported by `yt-dlp`. TikTok lives are not supported (stream-only).

### B. Download Queue & Concurrency
-   **Download Queue:** All downloads are managed through a central queue with state tracking (QUEUED → FETCHING_META → READY → DOWNLOADING → COMPLETED/FAILED).
-   **Configurable Concurrency:** 1-10 simultaneous downloads (default: 3).
-   **Pause / Resume / Cancel:** Per-download control via process management + `yt-dlp -c` for continuation.
-   **Retry:** Automatic retry with exponential backoff on transient errors.
-   **Priority:** Manual downloads take priority over automatic sync tasks.

### C. Source Management (Advanced)
-   **"Sources" Concept:** Instead of just "subscribing", users configure **Sources**.
-   **Sync Modes:**
    -   **Everything:** Download the entire archive.
    -   **New Only:** Monitor and download only future posts.
    -   **Date Range:** Download content between specific dates (e.g., "Posts from 2023").
    -   **Limit:** "Last N items" (e.g., "Latest 10 videos").
-   **Note:** MVP supports direct download + playlist download. Source Sync modes are documented but deferred.

### D. Gallery (Wall Mode)
-   **The Wall:** A Pinterest-style feed of all downloaded content, ordered chronologically.
-   **Post-Based:** Content is grouped by "Post", which may contain multiple "Media" items.
-   **Smart Organization:** Auto-grouping by Creator/Source.
-   **Performance:** Virtualized rendering (`@tanstack/react-virtual`) + lazy thumbnail loading for thousands of posts.
-   **Search & Filter:** By title, creator, platform, date. *(Post-MVP)*

### E. Authentication & Cookies
-   **Layered approach:**
    1.  No auth (public content) — default.
    2.  `yt-dlp --cookies-from-browser <browser>` — age-restricted/private content.
    3.  `yt-dlp --cookies <file>` — fallback when browser DB is locked.
-   **User configures** preferred browser in Settings.
-   **Account login** (username/password per platform) to be designed post-MVP.

### F. Rate Limiting & Anti-Ban
-   Configurable delays between downloads and metadata requests.
-   `yt-dlp` flags: `--sleep-interval`, `--sleep-requests`, `--retry-sleep exp:1:30`.
-   Exponential backoff on 429 errors.

### G. Storage & Disk Management
-   **Space monitoring:** Dashboard showing used vs. available disk space.
-   **Pre-download check:** Verifies sufficient space before starting download.
-   **Alerts:** Warning when disk space falls below configurable threshold (default: 5GB).
-   **Trash / Soft Delete:** Deleted content moves to trash with configurable auto-clean period.
-   **Physical file deletion:** Configurable — can either move to trash or delete permanently.

## 3. User Interface (UI/UX)
-   **Modern Aesthetics:** Clean, dark-mode-first design using React + TailwindCSS.
-   **Responsiveness:** Masonry Grid layout for the Wall.
-   **Native Feel:** Custom title bar, native file system integration.
-   **Download Manager:** Dedicated view for active downloads with progress bars, pause/cancel/retry.
-   **Internationalization:** English (default) + Spanish. Multi-language support via `react-i18next`. No hardcoded strings.

## 4. Distribution
-   **Target:** Windows `.exe` (Installer & Portable).
-   **Method:** Managed via Tauri's build pipeline.
-   **Auto-Update:** Tauri updater plugin for the app itself.
