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
*   **Async:** Tokio (including `watch` channels for settings live-reload)
*   **Logging:** Tracing

### Infrastructure / Binaries
*   **Database:** SQLite (embedded, connection enforces `journal_mode=WAL` + 5s `busy_timeout` for concurrent I/O)
*   **Downloader:** `yt-dlp` (Python-based, compiled executable)
*   **JS Runtime:** `Deno` (bundled sidecar for `yt-dlp` signature extraction)
*   **Media Proc:** `ffmpeg`
*   **Encryption:** Windows DPAPI (via Rust `bytehouse` or similar crate)

## Development Setup
*   **Node.js:** v18+
*   **Rust:** Stable (1.75+)
*   **VS Code:** Recommended extensions (Tauri, Rust-Analyzer, Tailwind).

## Database Schema (Key Tables)
*   **Keys:** Uses `INTEGER PRIMARY KEY AUTOINCREMENT` for all domain entities, linking via numerical IDs in foreign constraints, leaving string keys exclusively for platforms and settings entries.
*   `platforms`: Supported sites (YouTube, Instagram, etc.).
*   `creators`: Profiles/Channels (numerical `id`, `external_id`, `is_self`).
*   `sources`: Tracked playlists/queries (numerical `id`, `external_id`, `source_type`, `feed_type`).
*   `posts`: Content metadata.
*   `media`: Actual files (1:N with posts).
*   `download_tasks`: Active queue state.
*   `settings`: Key-value user configs.
*   `platform_sessions`: Encrypted auth data (now includes `username` and `avatar_url` columns).

## Technical Constraints & Safety
*   **SQLite Partial Indexes:** The `sources` table uses partial unique indexes to enforce uniqueness for feeds (where `feed_type` is not null) and URLs (where `feed_type` is null).
*   **Manual Upsert Pattern:** Because SQLite doesn't support targeting partial indexes in `ON CONFLICT`, the `store.rs` implementation performs manual find-or-insert operations within transactions.

## Key Algorithms
*   **Cookie Validation**: Checks for specific auth tokens (e.g. `auth_token`, `sessionid`) before saving sessions to prevent guest-cookie false positives.
*   **Profile Extraction**: Parses specific cookies and calls internal platform APIs to cache the logged-in username and avatar URL.
