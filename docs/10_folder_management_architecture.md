# Folder Structure and Media Management Architecture

## Overview
This document outlines the proposed architecture for introducing folder-based organization for downloaded media in VideoDownloaderPro. The goal is to provide a robust, secure, and user-friendly system that avoids the common pitfalls of absolute path storage, path traversal vulnerabilities, and desynchronization between the database and the OS file system.

## 1. Database Schema Design (The Foundation)
Currently, downloaded files are tracked using absolute paths (e.g., `C:\Users\Name\Downloads\VDP\video.mp4`) in the database. This approach is prone to breackage when the base directory is changed or files are moved.

### Proposed Solution: Relative Paths & Domain Entities
We will introduce a new abstract entity for folders and compute final paths dynamically at runtime.

#### New Entity: `folders`
*   `id` (UUID): Primary key.
*   `name` (String): Display name of the folder (e.g., "Music", "Funny Videos").
*   `relative_path` (String): The path relative to the root download directory (e.g., `music`, `funny_videos`). Used for OS creation.
*   `parent_id` (UUID, nullable): Self-referencing foreign key to support nested folder trees in the future.
*   `created_at`, `updated_at` (Timestamps).

#### Modifications to `media` and `download_task` Entities
*   **Add Column:** `folder_id` (UUID, nullable) Foreign key to `folders.id`.
*   **Modify Column:** `file_path` will be renamed or repurposed to `filename`. It will **strictly** store the basename of the file (e.g., `rickroll.mp4`), never the absolute path.

### The Single Source of Truth (SSOT) Approach
The absolute path is never persisted. Instead, the backend exposes a utility function, e.g., `resolve_media_path(media_id)`, which calculates the path on demand:
`[Base_Dir_From_Settings] / [Folder.relative_path] / [Media.filename]`

*Benefit:* Changing the global download directory in settings applies instantly to all past downloads. Renaming a folder only requires a single UPDATE statement on the `folders` table.

## 2. Security: Preventing Path Traversal
When accepting folder names or dynamically generating them from playlists/users, we risk Path Traversal attacks (e.g., a malicious name like `../../Windows/System32/`).

### Mitigation Strategy in Rust
1.  **Sanitization Middleware:**
    *   Strip or replace invalid OS characters: `<`, `>`, `:`, `"`, `/`, `\`, `|`, `?`, `*`.
    *   Reject sequences like `..` or `.`.
2.  **Logical Chroot (Canonicalization Guard):**
    Before any `std::fs::create_dir` or `std::fs::File::create` operation, the intended path must be validated.
    *   Compute the intended absolute path.
    *   Use `std::fs::canonicalize` to resolve any symbolic links or relative jumps.
    *   Assert that the resolved path `starts_with(&base_download_dir)`. If false, abort securely.

## 3. OS and Database Synchronization
Users will inevitably modify the folder structure directly via the Windows Explorer, renaming folders, moving videos, or deleting files. The application's UI (Wall) must handle this gracefully without displaying "Broken Image" icons or crashing.

### Proposed Architecture
*   **The Proactive Approach (Recommended for V2):**
    Integrate an OS file watcher (e.g., using `notify` crate in Rust or `tauri-plugin-fs-watch`).
    *   Listen to OS events in the background.
    *   On `FileMoved(old_path, new_path)`: Query the DB for the `old_path`, resolve the new `folder_id` and update the DB transparently.
    *   On `FileDeleted(path)`: Prompt the user in UI or quietly mark the media as 'Orphaned'.
*   **The Reactive Approach (MVP):**
    When the Wall tab is opened, or when a video is clicked, explicitly check `std::path::Path::exists()`.
    *   If missing, show a structured "File Not Found" ghost object in the UI, allowing the user to "Relocate" the file manually via a Tauri file picker.

## 4. User Experience & Features
*   **Creation:** The `AddDownloadModal` will include a Folder Dropdown combining existing folders and a "+ Create New Folder" option.
*   **Smart Rules:** Users can optionally define trigger rules: "If platform is TikTok, download to /TikTok".
*   **Virtual Tags vs Physical Folders:** The UI will clearly distinguish between managing files physically on disk versus simply assigning them visual "Tags" in the database. For many users, a robust Tagging system with a flat OS folder structure provides better organization without the risk of OS desynchronization.

## Summary Objective
Transition from absolute string-based file tracking to a decoupled Relational Database design backed by dynamic path resolution, ensuring system immunity against OS-level file moves and settings changes.
