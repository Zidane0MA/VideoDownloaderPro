# IPC API Contract (Frontend ↔ Backend)

## Overview
All communication between the React frontend and the Tauri Rust backend is done via `invoke()` (request/response) and Tauri events (server-push). This document defines the complete API surface.

---

## 1. Commands (`invoke()`)

### Download Operations

#### `create_download_task`
Creates a new download task in the queue.

```typescript
// Frontend
const taskId = await invoke<string>('create_download_task', {
  url: string,
  formatSelection?: string,  // yt-dlp format ID, default: 'best'
});
```

| Param | Type | Required | Description |
|---|---|---|---|
| `url` | string | ✅ | URL to download |
| `formatSelection` | string | ❌ | yt-dlp format string (e.g., `bestvideo+bestaudio/best`) |

**Returns:** `string` — UUID of the created task.
**Errors:** `INVALID_URL`, `DUPLICATE_URL` (if post with same URL already exists).

---

#### `get_queue_status`
Fetches all download tasks and global queue status.

```typescript
const status = await invoke<QueueStatusResponse>('get_queue_status');
```

**Returns:**

```typescript
interface QueueStatusResponse {
  isPaused: boolean;
  tasks: DownloadTask[];
}

interface DownloadTask {
  id: string;
  url: string;
  postId: string | null;
  status: 'QUEUED' | 'FETCHING_META' | 'READY' | 'DOWNLOADING' | 'PAUSED' | 'COMPLETED' | 'FAILED' | 'CANCELLED';
  priority: number;
  progress: number;       // 0.0 to 100.0 (Updated from 0.0-1.0 to match backend)
  speed: string | null;   // "2.5 MiB/s"
  eta: string | null;     // "00:05:23"
  errorMessage: string | null;
  retries: number;
  maxRetries: number;
  formatSelection: string | null;
  createdAt: string;      // ISO 8601
  startedAt: string | null;
  completedAt: string | null;
}
```

---

#### `pause_download_task`
Pauses an active download.

```typescript
await invoke('pause_download_task', { taskId: string });
```

**Errors:** `TASK_NOT_FOUND`, `INVALID_STATE`.

---

#### `resume_download_task`
Resumes a paused download.

```typescript
await invoke('resume_download_task', { taskId: string });
```

**Errors:** `TASK_NOT_FOUND`, `INVALID_STATE`.

---

#### `pause_queue`
Globally pauses the download queue.

```typescript
await invoke('pause_queue');
```

---

#### `resume_queue`
Resumes the download queue.

```typescript
await invoke('resume_queue');
```

---

#### `cancel_download_task`
Cancels a download and cleans up partial files.

```typescript
await invoke('cancel_download_task', { taskId: string });
```

**Errors:** `TASK_NOT_FOUND`.

---

#### `retry_download_task`
Retries a failed download.

```typescript
await invoke('retry_download_task', { taskId: string });
```

**Errors:** `TASK_NOT_FOUND`, `INVALID_STATE`.

---

### Post Operations

#### `get_posts`
Fetches posts for the Wall view with pagination and filtering.

```typescript
const result = await invoke<PostsPage>('get_posts', {
  page?: number,          // Default: 1
  limit?: number,         // Default: 50
  creatorId?: string,
  platformId?: string,
  search?: string,        // Search in title/description
  includeDeleted?: boolean, // Default: false (for trash view)
});
```

**Returns:**

```typescript
interface PostsPage {
  posts: Post[];
  total: number;
  page: number;
  totalPages: number;
}

interface Post {
  id: string;
  creatorId: string;
  creatorName: string;
  creatorAvatar: string | null;
  sourceId: string | null;
  title: string | null;
  description: string | null;
  originalUrl: string;
  status: 'PENDING' | 'COMPLETED' | 'FAILED';
  postedAt: string | null;
  downloadedAt: string | null;
  deletedAt: string | null;
  media: Media[];
}

interface Media {
  id: string;
  type: 'VIDEO' | 'IMAGE' | 'AUDIO';
  filePath: string;          // Use convertFileSrc() for display
  thumbnailPath: string | null;
  thumbnailSmPath: string | null;
  orderIndex: number;
  width: number | null;
  height: number | null;
  duration: number | null;    // seconds
  fileSize: number | null;    // bytes
}
```

---

#### `get_post`
Fetches a single post with all media.

```typescript
const post = await invoke<Post>('get_post', { postId: string });
```

---

#### `delete_post`
Soft-deletes a post (moves to trash).

```typescript
await invoke('delete_post', { postId: string });
```

---

#### `restore_post`
Restores a soft-deleted post from trash.

```typescript
await invoke('restore_post', { postId: string });
```

---

#### `permanently_delete_post`
Hard-deletes a post and optionally removes files from disk.

```typescript
await invoke('permanently_delete_post', { postId: string });
```

---

#### `empty_trash`
Permanently deletes all posts in trash.

```typescript
const deletedCount = await invoke<number>('empty_trash');
```

---

### Creator Operations

#### `get_creators`
Fetches all known creators.

```typescript
const creators = await invoke<Creator[]>('get_creators', {
  platformId?: string,
});
```

```typescript
interface Creator {
  id: string;
  platformId: string;
  name: string;
  handle: string | null;
  url: string;
  avatarPath: string | null;
  postCount: number;        // Computed
}
```

---

### Settings Operations

#### `get_settings`
Fetches all settings as a key-value map.

```typescript
const settings = await invoke<Record<string, string>>('get_settings');
```

---

#### `update_setting`
Updates a single setting.

```typescript
await invoke('update_setting', { key: string, value: string });
```

---

#### `get_disk_usage`
Returns disk space information.

```typescript
const usage = await invoke<DiskUsage>('get_disk_usage');
```

```typescript
interface DiskUsage {
  downloadPathTotal: number;    // bytes
  downloadPathUsed: number;     // bytes
  downloadPathAvailable: number; // bytes
  appDataSize: number;          // bytes (database + logs + trash)
  trashSize: number;            // bytes
}
```

---

### Sidecar Operations

#### `get_ytdlp_version`
Returns the current yt-dlp version.

```typescript
const version = await invoke<string>('get_ytdlp_version');
// Returns: "2025.01.15" or similar
```

---

#### `update_ytdlp`
Triggers a yt-dlp self-update.

```typescript
const result = await invoke<UpdateResult>('update_ytdlp');
```

```typescript
interface UpdateResult {
  success: boolean;
  oldVersion: string;
  newVersion: string | null;  // null if already up to date
  message: string;
}
```

---

### Session / Cookie Operations

#### `open_platform_login`
Opens a WebView window for the user to log into a platform. Cookies are captured automatically on close.

```typescript
const result = await invoke<LoginResult>('open_platform_login', {
  platformId: string,  // 'youtube', 'instagram', 'tiktok', 'x'
});
```

```typescript
interface LoginResult {
  success: boolean;
  platformId: string;
  status: 'ACTIVE' | 'EXPIRED' | 'NONE';
  expiresAt: string | null;  // ISO 8601
}
```

**Errors:** `PLATFORM_NOT_FOUND`, `WEBVIEW_ERROR`.

---

#### `get_session_status`
Returns the current session status for one or all platforms.

```typescript
const sessions = await invoke<PlatformSession[]>('get_session_status', {
  platformId?: string,  // Optional, omit for all platforms
});
```

```typescript
interface PlatformSession {
  platformId: string;
  status: 'ACTIVE' | 'EXPIRED' | 'NONE';
  cookieMethod: 'webview' | 'browser' | 'file' | null;
  expiresAt: string | null;
  lastVerified: string | null;
}
```

---

#### `logout_platform`
Clears stored cookies for a platform.

```typescript
await invoke('logout_platform', { platformId: string });
```

**Errors:** `PLATFORM_NOT_FOUND`.

---

#### `import_cookies_file`
Imports a `cookies.txt` file for a specific platform.

```typescript
const result = await invoke<LoginResult>('import_cookies_file', {
  platformId: string,
  filePath: string,     // Absolute path to cookies.txt
});
```

**Errors:** `PLATFORM_NOT_FOUND`, `INVALID_COOKIE_FILE` (wrong format or empty).

---

#### `set_cookie_method`
Sets the preferred cookie method globally.

```typescript
await invoke('set_cookie_method', {
  method: 'webview' | 'browser' | 'file',
  browser?: string,  // Required if method is 'browser' (e.g., 'chrome', 'firefox')
});
```

---

## 2. Events (Backend → Frontend)

Events are emitted by the Rust backend and listened to by the React frontend via `listen()`.

### `download-progress`
Emitted during active downloads (~every 500ms).

```typescript
interface DownloadProgressPayload {
  taskId: string;
  progress: number;  // 0.0 to 1.0
  speed: string;     // "2.5 MiB/s"
  eta: string;       // "00:05:23"
  downloadedBytes: number;
  totalBytes: number | null;
}
```

---

### `download-status-changed`
Emitted on every state transition.

```typescript
interface DownloadStatusPayload {
  taskId: string;
  oldStatus: string;
  newStatus: string;
  errorMessage: string | null;
  errorCode: string | null;     // Error code from 06_error_handling.md
}
```

---

### `queue-summary`
Emitted when the queue composition changes.

```typescript
interface QueueSummaryPayload {
  activeCount: number;
  queuedCount: number;
  pausedCount: number;
  completedCount: number;
  failedCount: number;
}
```

---

### `disk-space-warning`
Emitted when available disk space falls below threshold.

```typescript
interface DiskSpaceWarningPayload {
  availableGb: number;
  thresholdGb: number;
}
```

---

### `ytdlp-update-available`
Emitted when auto-update check finds a new version.

```typescript
interface YtdlpUpdatePayload {
  currentVersion: string;
  latestVersion: string;
}
```

---

### `session-status-changed`
Emitted when a platform session status changes (login, logout, expiration).

```typescript
interface SessionStatusPayload {
  platformId: string;
  oldStatus: 'ACTIVE' | 'EXPIRED' | 'NONE';
  newStatus: 'ACTIVE' | 'EXPIRED' | 'NONE';
  cookieMethod: 'webview' | 'browser' | 'file' | null;
  expiresAt: string | null;
}
```

---

### `auth-required`
Emitted when a download requires authentication and no valid cookies are available.

```typescript
interface AuthRequiredPayload {
  taskId: string;
  platformId: string;
  url: string;
  errorCode: 'AUTH_001' | 'AUTH_002' | 'AUTH_003';
  message: string;
}
```

---

## 3. Error Response Format

All `invoke()` errors follow this structure:

```typescript
interface TauriError {
  code: string;       // Machine-readable code (e.g., "TASK_NOT_FOUND")
  message: string;    // Human-readable message (localized)
  details?: any;      // Additional context
}
```

Common error codes:

| Code | Description |
|---|---|
| `TASK_NOT_FOUND` | Download task with given ID doesn't exist |
| `POST_NOT_FOUND` | Post with given ID doesn't exist |
| `INVALID_STATE` | Operation not valid for current state |
| `INVALID_URL` | URL is not recognized by yt-dlp |
| `DUPLICATE_URL` | Post with this URL already exists |
| `DISK_FULL` | Not enough disk space |
| `SIDECAR_NOT_FOUND` | yt-dlp or ffmpeg binary not found |
| `DB_ERROR` | Database operation failed |
| `SETTING_NOT_FOUND` | Unknown setting key |
| `PLATFORM_NOT_FOUND` | Unknown platform ID |
| `AUTH_REQUIRED` | Download needs authentication (no valid cookies) |
| `SESSION_EXPIRED` | Stored cookies are expired |
| `WEBVIEW_ERROR` | WebView login window failed to open or capture cookies |
| `INVALID_COOKIE_FILE` | Imported cookies.txt is malformed or empty |
