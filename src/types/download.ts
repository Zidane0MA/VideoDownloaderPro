export enum DownloadStatus {
  Queued = "QUEUED",
  Processing = "PROCESSING",
  Paused = "PAUSED",
  Completed = "COMPLETED",
  Failed = "FAILED",
  Cancelled = "CANCELLED",
}

export interface DownloadTask {
  id: string;
  url: string;
  status: DownloadStatus | string; // Allow string for flexibility, but prefer enum
  priority: number;
  progress: number;
  speed?: string;
  eta?: string;
  error_message?: string;
  retries: number;
  max_retries: number;
  created_at: string;
  started_at?: string;
  completed_at?: string;
  downloaded_bytes?: number;
  total_bytes?: number;
  title?: string;
  thumbnail?: string;
}

export interface CreateDownloadRequest {
  url: string;
  format_selection?: string;
}

export interface DownloadProgressPayload {
  task_id: string;
  progress: number;
  speed: string;
  eta: string;
  downloaded_bytes: number;
  total_bytes?: number;
}
