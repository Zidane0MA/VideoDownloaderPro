---
name: tauri-state-sync
description: "Backend↔Frontend state synchronization in Tauri v2: emitting Rust events, listening in React, React Query cache invalidation, Zustand store patterns, and tauri-plugin-store for persistent settings. Use when syncing download progress, queue state, or settings between Rust and React in VideoDownloaderPro."
---

# Tauri State Sync Patterns

Patterns for robust state synchronization between a Tauri v2 Rust backend and a React frontend. Covers event emission, React Query invalidation, Zustand UI state, and persistent settings.

## When to Use

- Emitting progress/status events from Rust to React
- Keeping React Query cache in sync with backend changes
- Managing UI-only state (selection, filters) with Zustand
- Persisting user settings with `tauri-plugin-store`

---

## Pattern 1: Emitting Events from Rust

```rust
use tauri::Emitter;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct DownloadProgressPayload {
    pub task_id: String,
    pub progress: f64,    // 0.0 to 1.0
    pub speed: String,
    pub eta: String,
}

#[derive(Clone, Serialize)]
pub struct StatusChangedPayload {
    pub task_id: String,
    pub old_status: String,
    pub new_status: String,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
}

/// Emit progress — call this from the download worker loop (~500ms intervals).
fn emit_progress(app: &tauri::AppHandle, payload: DownloadProgressPayload) {
    let _ = app.emit("download-progress", &payload);
}

/// Emit status transition — call on every state machine transition.
fn emit_status_changed(app: &tauri::AppHandle, payload: StatusChangedPayload) {
    let _ = app.emit("download-status-changed", &payload);
}

/// Emit queue summary — call after any task state change.
fn emit_queue_summary(app: &tauri::AppHandle, summary: QueueSummaryPayload) {
    let _ = app.emit("queue-summary", &summary);
}

#[derive(Clone, Serialize)]
pub struct QueueSummaryPayload {
    pub active_count: u32,
    pub queued_count: u32,
    pub paused_count: u32,
    pub completed_count: u32,
    pub failed_count: u32,
}
```

> **Key rule:** `emit()` is fire-and-forget. It never blocks. Always use `let _ =` to discard the Result — if no frontend is listening, that's fine.

---

## Pattern 2: Listening in React

```tsx
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useEffect, useRef } from 'react';

// Type-safe event hook
export function useTauriEvent<T>(
  eventName: string,
  handler: (payload: T) => void
) {
  const handlerRef = useRef(handler);
  handlerRef.current = handler;

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    listen<T>(eventName, (event) => {
      handlerRef.current(event.payload);
    }).then((fn) => { unlisten = fn; });

    return () => { unlisten?.(); };
  }, [eventName]);
}

// Usage
function DownloadManager() {
  useTauriEvent<DownloadProgressPayload>('download-progress', (payload) => {
    // Update local state or store
    updateProgress(payload.task_id, payload.progress);
  });

  useTauriEvent<StatusChangedPayload>('download-status-changed', (payload) => {
    // Invalidate relevant queries
    if (payload.new_status === 'COMPLETED') {
      queryClient.invalidateQueries({ queryKey: ['posts'] });
    }
    queryClient.invalidateQueries({ queryKey: ['download-tasks'] });
  });
}
```

---

## Pattern 3: React Query + Tauri Events Integration

React Query handles data fetching; Tauri events trigger selective invalidation.

```tsx
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';

// Fetch download tasks
export function useDownloadTasks(statusFilter?: string[]) {
  return useQuery({
    queryKey: ['download-tasks', statusFilter],
    queryFn: () => invoke<DownloadTask[]>('get_download_tasks', {
      status: statusFilter,
    }),
    refetchInterval: false,  // Don't poll — we use events
    staleTime: 30_000,
  });
}

// Global event sync (mount once at App level)
export function useGlobalEventSync() {
  const qc = useQueryClient();

  useTauriEvent<StatusChangedPayload>('download-status-changed', () => {
    qc.invalidateQueries({ queryKey: ['download-tasks'] });
  });

  useTauriEvent<QueueSummaryPayload>('queue-summary', (summary) => {
    // Optimistic update: set cache directly without refetch
    qc.setQueryData(['queue-summary'], summary);
  });

  // When a download completes, the posts list needs updating
  useTauriEvent<StatusChangedPayload>('download-status-changed', (p) => {
    if (p.new_status === 'COMPLETED') {
      qc.invalidateQueries({ queryKey: ['posts'] });
      qc.invalidateQueries({ queryKey: ['creators'] });
    }
  });
}
```

### When to invalidate vs optimistic update

| Scenario | Strategy | Why |
|---|---|---|
| Download progress (%) | Zustand store | High frequency, no DB query needed |
| Task status changed | `invalidateQueries` | Need fresh data from DB |
| Queue summary counts | `setQueryData` | Payload contains full data |
| New post completed | `invalidateQueries` | Post data comes from DB |
| Settings changed | `setQueryData` + `invalidateQueries` | Update cache, then verify |

---

## Pattern 4: Zustand for High-Frequency UI State

React Query is for server state. Zustand handles UI state that changes rapidly.

```tsx
import { create } from 'zustand';

interface DownloadProgressState {
  // Map of taskId → progress data (updated via events, ~500ms)
  progress: Record<string, {
    percent: number;
    speed: string;
    eta: string;
  }>;
  setProgress: (taskId: string, percent: number, speed: string, eta: string) => void;
  removeProgress: (taskId: string) => void;
}

export const useDownloadProgress = create<DownloadProgressState>((set) => ({
  progress: {},

  setProgress: (taskId, percent, speed, eta) =>
    set((state) => ({
      progress: {
        ...state.progress,
        [taskId]: { percent, speed, eta },
      },
    })),

  removeProgress: (taskId) =>
    set((state) => {
      const { [taskId]: _, ...rest } = state.progress;
      return { progress: rest };
    }),
}));
```

Wire it up with the event:
```tsx
function AppEventProvider({ children }: { children: React.ReactNode }) {
  const { setProgress, removeProgress } = useDownloadProgress();

  useTauriEvent<DownloadProgressPayload>('download-progress', (p) => {
    setProgress(p.task_id, p.progress, p.speed, p.eta);
  });

  useTauriEvent<StatusChangedPayload>('download-status-changed', (p) => {
    if (['COMPLETED', 'FAILED', 'CANCELLED'].includes(p.new_status)) {
      removeProgress(p.task_id);
    }
  });

  return <>{children}</>;
}
```

---

## Pattern 5: Persistent Settings with tauri-plugin-store

```bash
# Install plugin
cargo add tauri-plugin-store    # in src-tauri/
npm install @tauri-apps/plugin-store
```

**Rust setup:**
```rust
// src-tauri/src/lib.rs
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("error");
}
```

**Frontend — settings store:**
```tsx
import { Store } from '@tauri-apps/plugin-store';

const STORE_PATH = 'settings.json';

// Singleton store instance
let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
  if (!storeInstance) {
    storeInstance = await Store.load(STORE_PATH);
  }
  return storeInstance;
}

export async function getSetting<T>(key: string, defaultValue: T): Promise<T> {
  const store = await getStore();
  const value = await store.get<T>(key);
  return value ?? defaultValue;
}

export async function setSetting<T>(key: string, value: T): Promise<void> {
  const store = await getStore();
  await store.set(key, value);
  await store.save();
}

// Common settings
export const Settings = {
  downloadPath: (v?: string) =>
    v ? setSetting('downloadPath', v) : getSetting('downloadPath', 'C:\\Downloads'),
  maxConcurrent: (v?: number) =>
    v ? setSetting('maxConcurrent', v) : getSetting('maxConcurrent', 3),
  cookieBrowser: (v?: string) =>
    v ? setSetting('cookieBrowser', v) : getSetting<string | null>('cookieBrowser', null),
  language: (v?: string) =>
    v ? setSetting('language', v) : getSetting('language', 'en'),
  theme: (v?: string) =>
    v ? setSetting('theme', v) : getSetting('theme', 'dark'),
};
```

**Capabilities required:**
```json
// src-tauri/capabilities/default.json
{
  "permissions": [
    "core:default",
    "store:allow-get",
    "store:allow-set",
    "store:allow-save",
    "store:allow-load"
  ]
}
```

---

## Architecture Summary

```
┌─────────────── Frontend (React) ────────────────┐
│                                                   │
│  React Query        Zustand          Plugin-Store │
│  (server state)     (UI state)       (persistent) │
│  - posts            - progress %     - settings   │
│  - creators         - selection      - theme      │
│  - download tasks   - filters        - language   │
│                                                   │
│         ▲ invalidate    ▲ setState                │
│         │               │                         │
│    ┌────┴───────────────┴────┐                    │
│    │  useTauriEvent() hooks  │                    │
│    └────────────┬────────────┘                    │
└─────────────────┼─────────────────────────────────┘
                  │ listen()
         ─────────┼──────────
                  │ emit()
┌─────────────────┼─────────────────────────────────┐
│        Tauri Rust Backend                         │
│                                                   │
│  app.emit("download-progress", payload)           │
│  app.emit("download-status-changed", payload)     │
│  app.emit("queue-summary", payload)               │
└───────────────────────────────────────────────────┘
```

## Best Practices

- **Events for push, invoke for pull** — use events for real-time updates, `invoke()` for on-demand queries
- **Throttle frequent events** — don't emit progress faster than 500ms
- **Zustand for high-frequency** — progress bars update via Zustand, not React Query
- **Invalidate selectively** — never `invalidateQueries()` without a `queryKey` filter
- **Always `useRef` for handlers** — prevents event listener re-registration on every render
