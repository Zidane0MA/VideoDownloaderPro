import React from 'react';
import { DownloadTask, DownloadStatus } from '../types/download';
import { useDownloadManager } from '../hooks/useDownloadManager';
import { 
  Play, 
  Pause, 
  X, 
  RotateCw, 
  FileVideo, 
  AlertCircle, 
  CheckCircle2 
} from 'lucide-react';

interface DownloadItemProps {
  task: DownloadTask;
}

export const DownloadItem: React.FC<DownloadItemProps> = ({ task }) => {
  const { pauseDownload, resumeDownload, cancelDownload, retryDownload } = useDownloadManager();

  const getStatusColor = (status: string) => {
    switch (status) {
      case DownloadStatus.Completed: return 'text-green-400';
      case DownloadStatus.Failed: return 'text-red-400';
      case DownloadStatus.Paused: return 'text-yellow-400';
      case DownloadStatus.Processing: return 'text-blue-400';
      default: return 'text-zinc-400';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case DownloadStatus.Completed: return <CheckCircle2 size={16} />;
      case DownloadStatus.Failed: return <AlertCircle size={16} />;
      case DownloadStatus.Paused: return <Pause size={16} />;
      case DownloadStatus.Processing: return <Play size={16} />;
      default: return <FileVideo size={16} />;
    }
  };

  const formatBytes = (bytes?: number) => {
    if (!bytes) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  };

  return (
    <div className="bg-surface-800 border border-surface-700 rounded-xl p-4 transition-all hover:border-surface-600">
      <div className="flex items-start justify-between gap-4">
        {/* Icon & Info */}
        <div className="flex items-start gap-3 flex-1 min-w-0">
          <div className={`mt-1 ${getStatusColor(task.status as string)}`}>
             {getStatusIcon(task.status as string)}
          </div>
          <div className="flex-1 min-w-0">
            <h3 className="text-sm font-medium text-surface-100 truncate pr-4" title={task.url}>
              {task.url}
            </h3>
            <div className="flex items-center gap-2 mt-1 text-xs text-surface-200/60">
              <span className={`uppercase font-semibold ${getStatusColor(task.status as string)}`}>
                {task.status}
              </span>
              <span>•</span>
              {task.total_bytes ? (
                <span>{formatBytes(task.downloaded_bytes)} / {formatBytes(task.total_bytes)}</span>
              ) : (
                <span>{formatBytes(task.downloaded_bytes)} downloaded</span>
              )}
              {task.speed && (
                <>
                  <span>•</span>
                  <span>{task.speed}</span>
                </>
              )}
              {task.eta && (
                <>
                  <span>•</span>
                  <span>ETA: {task.eta}</span>
                </>
              )}
            </div>
            {task.error_message && (
               <p className="mt-1 text-xs text-red-400">{task.error_message}</p>
            )}
          </div>
        </div>

        {/* Actions */}
        <div className="flex items-center gap-1">
          {task.status === DownloadStatus.Processing && (
            <button
              onClick={() => pauseDownload(task.id)}
              className="p-2 text-surface-200 hover:text-yellow-400 hover:bg-surface-700 rounded-lg transition-colors"
              title="Pause"
            >
              <Pause size={16} />
            </button>
          )}

          {task.status === DownloadStatus.Paused && (
            <button
              onClick={() => resumeDownload(task.id)}
              className="p-2 text-surface-200 hover:text-green-400 hover:bg-surface-700 rounded-lg transition-colors"
              title="Resume"
            >
              <Play size={16} />
            </button>
          )}

          {(task.status === DownloadStatus.Failed || task.status === DownloadStatus.Cancelled) && (
            <button
              onClick={() => retryDownload(task.id)}
              className="p-2 text-surface-200 hover:text-blue-400 hover:bg-surface-700 rounded-lg transition-colors"
              title="Retry"
            >
              <RotateCw size={16} />
            </button>
          )}

          {(task.status === DownloadStatus.Processing || task.status === DownloadStatus.Paused || task.status === DownloadStatus.Queued) && (
            <button
              onClick={() => cancelDownload(task.id)}
              className="p-2 text-surface-200 hover:text-red-400 hover:bg-surface-700 rounded-lg transition-colors"
              title="Cancel"
            >
              <X size={16} />
            </button>
          )}
        </div>
      </div>

      {/* Progress Bar */}
      {(task.status === DownloadStatus.Processing || task.status === DownloadStatus.Paused) && (
        <div className="mt-3 h-1.5 w-full bg-surface-900 rounded-full overflow-hidden">
          <div
            className={`h-full transition-all duration-300 ${
               task.status === DownloadStatus.Paused ? 'bg-yellow-500' : 'bg-brand-500'
            }`}
            style={{ width: `${task.progress}%` }}
          />
        </div>
      )}
    </div>
  );
};
