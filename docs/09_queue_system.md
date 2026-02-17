# Queue System

The Download Queue System manages concurrent video downloads to ensure system stability and predictable resource usage. It is built on `tokio` for async execution, `Sea-ORM` for state persistence, and `tokio-util` for cancellation.

## Architecture

```mermaid
graph TD
    UI[Frontend] -->|IPC: create_download_task| CMD[Command Handler]
    UI -->|IPC: cancel / retry / status| CMD
    CMD -->|Insert / Update| DB[(SQLite)]
    CMD -->|Notify| Queue[DownloadQueue]
    
    subgraph Background Scheduler
        Queue -->|Recover stale| DB
        Queue -->|Acquire Permit| Sem[Semaphore — Limit: 3]
        Queue -->|Poll by priority| DB
        Queue -->|Spawn + child token| Worker[DownloadWorker]
    end
    
    Worker -->|Update Progress| DB
    Worker -->|Emit Events| UI
    Worker -->|Release Permit| Sem
    Worker -->|On failure| Retry{retries < max?}
    Retry -->|Yes: backoff + requeue| DB
    Retry -->|No: mark FAILED| DB
```

## Key Components

### 1. DownloadQueue (`src-tauri/src/queue/manager.rs`)
- **Semaphore**: Limits concurrent downloads (default: 3).
- **Notify**: Async notification mechanism to wake up the scheduler when new tasks are added.
- **CancellationToken**: Parent token enables graceful shutdown; child tokens enable per-task cancellation.
- **Task Token Registry**: `HashMap<String, CancellationToken>` tracks active tasks for targeted cancellation.
- **Stale Recovery**: On startup, resets any `PROCESSING` tasks back to `QUEUED`.
- **Scheduler Loop**: Checks DB before acquiring a permit; waits for notification when queue is empty.

### 2. DownloadWorker (`src-tauri/src/download/worker.rs`)
- **Cancellation**: Uses `tokio::select!` to race stdout reading against `CancellationToken`.
- **Stderr capture**: Separate tokio task collects stderr lines for error reporting.
- **Format selection**: Passes `-f <format>` when `format_selection` is provided.
- **Output template**: Uses `--output "%(title)s.%(ext)s"`.
- **Progress throttling**: Emits events at most every 500ms to avoid IPC flooding.
- **DB writes**: Persists `progress`, `speed`, `eta` using `update_many` to avoid redundant `SELECT` queries.
- **Robust Termination**: On Windows, uses `taskkill /F /T /PID` to ensure the entire process tree (including `ffmpeg`) is killed.
- **Merge Handling**: Parses `[Merger]` output to identify the final filename and reads the actual file size from disk to ensure accuracy.

### 3. Task States (`download_task` table)
- `QUEUED`: Waiting for a slot.
- `PROCESSING`: Currently downloading.
- `COMPLETED`: Finished successfully.
- `FAILED`: Permanently failed (retries exhausted).
- `CANCELLED`: Cancelled by user.
- `PAUSED`: Paused by user (individual or via global pause).

### 4. Retry Logic
- On failure, checks `retries < max_retries` (default: 3).
- Increments `retries`, resets to `QUEUED`, applies exponential backoff (5s × 2^n).
- After all retries exhausted, marks `FAILED` with error message from stderr.

### 5. Pause / Resume
- **Global pause**: `AtomicBool` flag checked by the scheduler — stops picking up new tasks.
- **Per-task pause**: ON WINDOWS: Uses `taskkill /F /T` to terminate the process tree. ON LINUX/MAC: Standard `kill()`. State is saved as `PAUSED`.
- **Resume**: PAUSED tasks are set back to `QUEUED`. yt-dlp `-c` flag enables native partial download resumption.

### 6. IPC Commands
| Command | Description |
|---------|-------------|
| `create_download_task` | Creates a DB entry with `QUEUED` status and notifies the scheduler. |
| `cancel_download_task` | Cancels a running task (kills process tree) or marks a queued task as `CANCELLED`. |
| `retry_download_task` | Resets a `FAILED` or `CANCELLED` task to `QUEUED` with retries cleared. |
| `get_queue_status` | Returns all tasks + global `is_paused` flag. |
| `pause_download_task` | Pauses a single QUEUED or PROCESSING task. |
| `resume_download_task` | Resumes a PAUSED task back to QUEUED. |
| `pause_queue` | Globally pauses the scheduler. |
| `resume_queue` | Resumes the scheduler. |

## User Controls & IPC Mapping

The system exposes high-level controls via IPC that map directly to queue operations.

### Pause / Resume
- **Global Pause**: Freezes the scheduler. No new tasks are started. Running tasks continue until they finish or are individually paused.
- **Task Pause**:
  - **Running (`PROCESSING`)**: The process tree is terminated (Windows: `taskkill /F /T`). The state is saved as `PAUSED`.
  - **Queued (`QUEUED`)**: State simply flips to `PAUSED`.
- **Resume**:
  - **Task**: State flips to `QUEUED`. When picked up, `yt-dlp` is spawned with `-c` (continue) to resume from the partial file.

### Cancel
- **Running**: Process tree is terminated. State becomes `CANCELLED`. Partial files (`.part`) and any merged files are deleted to clean up.
- **Queued**: State becomes `CANCELLED`.

### Retry
- **Manual Retry**: Resets a `FAILED` or `CANCELLED` task to `QUEUED` with a fresh retry counter.
- **Auto-Retry**: Handled internally by the worker for transient errors (network, timeouts).

## Usage

The queue is initialized in `lib.rs` and managed as Tauri state.

```rust
// In lib.rs
let queue = queue::DownloadQueue::new(app.handle().clone(), 3);
app.manage(queue.clone());
tauri::async_runtime::spawn(async move {
    queue.start_scheduler().await;
});
```
