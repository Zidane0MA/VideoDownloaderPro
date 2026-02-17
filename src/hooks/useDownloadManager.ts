import { useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useDownloadStore } from '../store/downloadStore';
import {
  DownloadTask,
  DownloadStatus,
  CreateDownloadRequest,
  DownloadProgressPayload
} from '../types/download';

export function useDownloadManager() {
  const {
    tasks: tasksRecord,
    isQueuePaused,
    setTasks,
    updateTask,
    setQueuePaused
  } = useDownloadStore();

  const tasks = useMemo(() => {
    return Object.values(tasksRecord).sort((a, b) =>
      new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
    );
  }, [tasksRecord]);

  // Initialize: Fetch initial queue status
  useEffect(() => {
    fetchQueueStatus();
  }, []);

  // Set up event listeners
  useEffect(() => {
    const setupListeners = async () => {
      // Progress updates
      const unlistenProgress = await listen<DownloadProgressPayload>('download-progress', (event) => {
        const payload = event.payload;
        updateTask(payload.task_id, {
          progress: payload.progress,
          speed: payload.speed,
          eta: payload.eta,
          downloaded_bytes: payload.downloaded_bytes,
          total_bytes: payload.total_bytes,
          // Let the completion event handle the final status transition
          status: DownloadStatus.Processing
        });
      });

      // Task completion — authoritative status from backend
      const unlistenCompleted = await listen<string>('download-completed', (event) => {
        updateTask(event.payload, {
          status: DownloadStatus.Completed,
          progress: 100,
          completed_at: new Date().toISOString(),
          speed: undefined,
          eta: undefined,
          error_message: undefined
        });
      });

      // Task failure
      const unlistenFailed = await listen<string>('download-failed', (event) => {
        updateTask(event.payload, {
          status: DownloadStatus.Failed,
          speed: undefined,
          eta: undefined
        });
        // Refresh to get error message from backend
        fetchQueueStatus();
      });

      // Task paused
      const unlistenPaused = await listen<string>('download-paused', (event) => {
        updateTask(event.payload, {
          status: DownloadStatus.Paused,
          speed: undefined,
          eta: undefined,
          error_message: undefined
        });
      });

      // Task cancelled
      const unlistenCancelled = await listen<string>('download-cancelled', (event) => {
        updateTask(event.payload, {
          status: DownloadStatus.Cancelled,
          speed: undefined,
          eta: undefined
        });
      });

      return () => {
        unlistenProgress();
        unlistenCompleted();
        unlistenFailed();
        unlistenPaused();
        unlistenCancelled();
      };
    };

    const unlistenPromise = setupListeners();

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const fetchQueueStatus = async () => {
    try {
      const response = await invoke<{ is_paused: boolean; tasks: DownloadTask[] }>('get_queue_status');
      setTasks(response.tasks);
      setQueuePaused(response.is_paused);
    } catch (error) {
      console.error('Failed to fetch queue status:', error);
    }
  };

  const createDownload = async (url: string, formatSelection?: string) => {
    try {
      const request: CreateDownloadRequest = { url, format_selection: formatSelection };
      const taskId = await invoke<string>('create_download_task', { request });

      // Fetch status to get the full task object created by backend
      await fetchQueueStatus();
      return taskId;
    } catch (error) {
      console.error('Failed to create download:', error);
      throw error;
    }
  };

  const cancelDownload = async (taskId: string) => {
    try {
      // Optimistic update
      updateTask(taskId, { status: DownloadStatus.Cancelled, speed: undefined, eta: undefined });
      await invoke('cancel_download_task', { taskId });
    } catch (error) {
      console.error('Failed to cancel download:', error);
      fetchQueueStatus();
    }
  };

  const pauseDownload = async (taskId: string) => {
    try {
      // Optimistic update — clear stale speed/eta
      updateTask(taskId, { status: DownloadStatus.Paused, speed: undefined, eta: undefined });
      await invoke('pause_download_task', { taskId });
    } catch (error) {
      console.error('Failed to pause download:', error);
      fetchQueueStatus();
    }
  };

  const resumeDownload = async (taskId: string) => {
    try {
      // Optimistic update — clear stale data
      updateTask(taskId, {
        status: DownloadStatus.Queued,
        error_message: undefined,
        speed: undefined,
        eta: undefined
      });
      await invoke('resume_download_task', { taskId });
    } catch (error) {
      console.error('Failed to resume download:', error);
      fetchQueueStatus();
    }
  };

  const retryDownload = async (taskId: string) => {
    try {
      await invoke('retry_download_task', { taskId });
      // Task will go back to QUEUED
      await fetchQueueStatus();
    } catch (error) {
      console.error('Failed to retry download:', error);
    }
  };

  const pauseQueue = async () => {
    try {
      await invoke('pause_queue');
      setQueuePaused(true);
    } catch (error) {
      console.error('Failed to pause queue:', error);
    }
  };

  const resumeQueue = async () => {
    try {
      await invoke('resume_queue');
      setQueuePaused(false);
    } catch (error) {
      console.error('Failed to resume queue:', error);
    }
  };

  return {
    tasks,
    isQueuePaused,
    createDownload,
    cancelDownload,
    pauseDownload,
    resumeDownload,
    retryDownload,
    pauseQueue,
    resumeQueue,
    refreshQueue: fetchQueueStatus,
  };
}
