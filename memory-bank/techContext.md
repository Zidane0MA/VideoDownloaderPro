# Tech Context

## Technology Stack

### Frontend
*   **Framework:** React 18 (Vite)
*   **Language:** TypeScript
*   **Styling:** TailwindCSS v3 (Stable)
*   **State:** Zustand (global UI state), TanStack Query (server state/IPC)
*   **Virtualization:** @tanstack/react-virtual (for the "Wall")
*   **I18n:** react-i18next

### Backend (Rust)
*   **Runtime:** Tauri v2
*   **Database ORM:** Sea-ORM (Async, SQLite)
*   **Serialization:** Serde
*   **Async:** Tokio
*   **Logging:** Tracing

### Infrastructure / Binaries
*   **Database:** SQLite (embedded)
*   **Downloader:** `yt-dlp` (Python-based, compiled executable)
*   **JS Runtime:** `QuickJS` (bundled sidecar for `yt-dlp` signature extraction)
*   **Media Proc:** `ffmpeg`
*   **Encryption:** Windows DPAPI (via Rust `bytehouse` or similar crate)

## Development Setup
*   **Node.js:** v18+
*   **Rust:** Stable (1.75+)
*   **VS Code:** Recommended extensions (Tauri, Rust-Analyzer, Tailwind).

## Database Schema (Key Tables)
*   `platforms`: Supported sites (YouTube, Instagram, etc.).
*   `creators`: Profiles/Channels.
*   `sources`: Tracked playlists/queries.
*   `posts`: Content metadata.
*   `media`: Actual files (1:N with posts).
*   `download_tasks`: Active queue state.
*   `settings`: Key-value user configs.
*   `platform_sessions`: Encrypted auth data.
