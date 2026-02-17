# Active Context

**Phase 3: Download Engine (Rust)**
Phase 2 is complete. Database layer is set up with SQLite + Sea-ORM. Moving to Phase 3: building the download engine.

## Recent Changes
- **Download Engine Backend Completed**:
    - Implemented `worker.rs` with cancellation, throttling, and stderr capture.
    - Implemented `manager.rs` with priority queue, retry logic, and global/per-task pause/resume.
    - Added 8 IPC commands for full download management.
- **Documentation Updated**:
    - Updated `09_queue_system.md` and `07_ipc_api_contract.md` to match implementation.

## Current State
- Backend is fully functional for downloads (add, cancel, pause, resume, retry, get status).
- Tests passing for parser logic.
- **Missing**: Frontend integration (React hooks/UI), Disk space checking, System tray.

## Next Steps
1.  Implement Frontend Download Manager (React + Zustand).
2.  Create "Add Download" modal with format selection.
3.  Build "Downloads" page with active/queued/completed lists.
4.  Add System Tray support for background behavior.
