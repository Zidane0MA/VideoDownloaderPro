# Queue System

The Download Queue System manages concurrent video downloads to ensure system stability and predictable resource usage. It is built on `tokio` for async execution and `Sea-ORM` for state persistence.

## Architecture

```mermaid
graph TD
    UI[Frontend] -->|IPC: create_download_task| CMD[Command Handler]
    CMD -->|Insert| DB[(SQLite)]
    CMD -->|Notify| Queue[DownloadQueue]
    
    subgraph Background Scheduler
        Queue -->|Acquire Permit| Sem[Semaphore (Limit: 3)]
        Queue -->|Poll| DB
        Queue -->|Spawn| Worker[DownloadWorker]
    end
    
    Worker -->|Update Status| DB
    Worker -->|Emit Events| UI
    Worker -->|Release Permit| Sem
```

## Key Components

### 1. DownloadQueue (`src-tauri/src/queue/manager.rs`)
- **Semaphore**: Limits concurrent downloads (default: 3).
- **Notify**: Async notification mechanism to wake up the scheduler when new tasks are added.
- **Scheduler Loop**: Continuously runs in the background, waiting for available slots and new tasks.

### 2. Task States (`download_task` table)
- `QUEUED`: Waiting for a slot.
- `PROCESSING`: Currently downloading.
- `COMPLETED`: Finished successfully.
- `FAILED`: Encountered an error.

### 3. IPC Integration
- `create_download_task`: Creates a DB entry with `QUEUED` status and notifies the scheduler. The scheduler picks it up when a slot is free.

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
