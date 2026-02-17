# Project Brief: Video Downloader Pro

> [!NOTE]
> Detailed technical specifications, API contracts, and roadmap items are maintained in `/docs`. This file serves as a high-level overview. Always cross-reference `/docs` during implementation.

## Top-Level Goals
1.  **Professional Desktop Application:** Build a high-performance, native-feeling Windows application for downloading video/audio from 1000+ sites (YouTube, TikTok, Instagram, X).
2.  **"Wall of Content" Experience:** Differentiate from competitors (like *4K Video Downloader*) by offering a visually rich, pinterest-style gallery of downloaded content, rather than just a list of files.
3.  **Tech Demo & Reusability:** Serve as a polished proof-of-concept for the "TagFlowPro" ecosystem, with a modular architecture that allows the downloader engine to be reused in future projects.

## Core Core Requirements
*   **Engine:** Powered by `yt-dlp` and `ffmpeg` as managed sidecars.
*   **Performance:** Capable of handling thousands of items in the gallery with virtualized rendering.
*   **Privacy:** Cookie/Auth data is encrypted at rest (Windows DPAPI) and never leaves the local machine.
*   **UX:** "Quiet" operation with clear status indicators, persistent queues, and drag-and-drop simplicity.
*   **Distribution:** Single `.exe` installer and portable version for Windows.

## Key Features
*   **Smart Sources:** Subscribe to Playlists, Channels, or specfic search queries (Post-MVP).
*   **4-Layer Authentication:** From public access to WebView-based login for restricted content.
*   **Robust Queue:** Concurrency control, retries, pause/resume, and priorities.
*   **Media Management:** Auto-grouping by Creator/Source, carousel support for multi-item posts.
