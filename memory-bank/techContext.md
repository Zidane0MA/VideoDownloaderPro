# Tech Context

## Technology Stack

### Frontend
*   **Framework:** React 18 (Vite)
*   **Language:** TypeScript
*   **Styling:** TailwindCSS v3 (Stable)
*   **State:** Zustand (global UI state), TanStack Query (server state/IPC)
*   **Virtualization:** @virtuoso.dev/masonry (for the "Wall")
*   **I18n:** react-i18next

### Backend (Rust)
*   **Runtime:** Tauri v2
*   **Plugins:** `tauri-plugin-dialog` (File picker), `tauri-plugin-shell`, `tauri-plugin-opener`
*   **Database ORM:** Sea-ORM (Async, SQLite)
*   **Serialization:** Serde
*   **Async:** Tokio
*   **Logging:** Tracing

### Infrastructure / Binaries
*   **Database:** SQLite (embedded)
*   **Downloader:** `yt-dlp` (Python-based, compiled executable)
*   **JS Runtime:** `Deno` (bundled sidecar for `yt-dlp` signature extraction)
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
*   `platform_sessions`: Encrypted auth data (now includes `username` and `avatar_url` columns).

## Key Algorithms
*   **Cookie Validation**: Checks for specific auth tokens (e.g. `auth_token`, `sessionid`) before saving sessions to prevent guest-cookie false positives.
*   **Profile Extraction**: Parses specific cookies and calls internal platform APIs to cache the logged-in username and avatar URL.
