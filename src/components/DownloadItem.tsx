import React from 'react';
import { DownloadTask, DownloadStatus } from '../types/download';
import { useDownloadManager } from '../hooks/useDownloadManager';
import { 
  Play, 
  Pause, 
  X, 
  RotateCw, 
  AlertCircle, 
  CheckCircle2,
  Image as ImageIcon
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
      case DownloadStatus.Processing: return 'text-brand-400';
      default: return 'text-zinc-400';
    }
  };

  const formatBytes = (bytes?: number) => {
    if (!bytes) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  };

  const isActive = task.status === DownloadStatus.Processing || task.status === DownloadStatus.Paused;
  const isPaused = task.status === DownloadStatus.Paused;

  return (
    <div className="group bg-surface-800 border border-surface-700 rounded-xl p-4 transition-all hover:border-surface-600 hover:shadow-lg hover:shadow-black/20">
      <div className="flex gap-4">
        {/* Thumbnail Section */}
        <div className="relative w-32 aspect-video bg-surface-900 rounded-lg overflow-hidden flex-shrink-0 border border-surface-700/50">
          {task.thumbnail ? (
            <img 
              src={task.thumbnail} 
              alt={task.title || "Video thumbnail"} 
              className="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105"
            />
          ) : (
            <div className="w-full h-full flex items-center justify-center text-surface-600">
              <ImageIcon size={24} />
            </div>
          )}
          
          {/* Status Overlay on Thumbnail */}
          <div className="absolute inset-0 bg-black/20 group-hover:bg-transparent transition-colors" />
          {isActive && (
            <div className="absolute bottom-1 right-1 px-1.5 py-0.5 bg-black/60 backdrop-blur-md rounded text-[10px] font-medium text-white tabular-nums">
              {Math.round(task.progress)}%
            </div>
          )}
        </div>

        {/* Content Section */}
        <div className="flex-1 min-w-0 flex flex-col justify-between py-0.5">
          <div>
            <h3 className="text-sm font-semibold text-surface-100 truncate leading-tight mb-1" title={task.title || task.url}>
              {task.title || task.url}
            </h3>
            {task.title && (
               <p className="text-xs text-surface-400 truncate font-mono opacity-60 hover:opacity-100 transition-opacity">
                 {task.url}
               </p>
            )}
          </div>

          <div className="space-y-2">
            {/* Progress Bar (Only for active/paused) */}
            {isActive && (
              <div className="h-1.5 w-full bg-surface-900 rounded-full overflow-hidden">
                <div
                  className={`h-full transition-all duration-300 ${
                     isPaused ? 'bg-yellow-500/80' : 'bg-brand-500 shadow-[0_0_10px_rgba(59,130,246,0.5)]'
                  }`}
                  style={{ width: `${task.progress}%` }}
                />
              </div>
            )}

            {/* Metadata Row */}
            <div className="flex items-center gap-3 text-xs text-surface-300 font-medium">
               <div className={`flex items-center gap-1.5 ${getStatusColor(task.status as string)}`}>
                  {task.status === DownloadStatus.Processing && <Play size={12} className="fill-current" />}
                  {task.status === DownloadStatus.Paused && <Pause size={12} className="fill-current" />}
                  {task.status === DownloadStatus.Completed && <CheckCircle2 size={12} />}
                  {task.status === DownloadStatus.Failed && <AlertCircle size={12} />}
                  <span>{task.status}</span>
               </div>

               {isActive && (
                 <>
                   <span className="text-surface-600">•</span>
                   <span>{formatBytes(task.downloaded_bytes)} / {formatBytes(task.total_bytes)}</span>
                   {task.speed && (
                     <>
                        <span className="text-surface-600">•</span>
                        <span className="text-surface-200">{task.speed}</span>
                     </>
                   )}
                   {task.eta && (
                     <>
                        <span className="text-surface-600">•</span>
                        <span className="text-brand-300">ETA: {task.eta}</span>
                     </>
                   )}
                 </>
               )}
                
               {task.status === DownloadStatus.Completed && task.total_bytes && (
                 <>
                    <span className="text-surface-600">•</span>
                    <span>{formatBytes(task.total_bytes)}</span>
                 </>
               )}
               
               {task.error_message && (
                  <span className="text-red-400 truncate max-w-[200px]" title={task.error_message}>
                    — {task.error_message}
                  </span>
               )}
            </div>
          </div>
        </div>

        {/* Actions Section */}
        <div className="flex flex-col items-end gap-1 pl-2 border-l border-surface-700/30">
          {task.status === DownloadStatus.Processing && (
            <button
              onClick={() => pauseDownload(task.id)}
              className="p-2 text-surface-400 hover:text-yellow-400 hover:bg-surface-700/50 rounded-lg transition-colors"
              title="Pause"
            >
              <Pause size={18} />
            </button>
          )}

          {task.status === DownloadStatus.Paused && (
            <button
              onClick={() => resumeDownload(task.id)}
              className="p-2 text-surface-400 hover:text-green-400 hover:bg-surface-700/50 rounded-lg transition-colors"
              title="Resume"
            >
              <Play size={18} />
            </button>
          )}

          {(task.status === DownloadStatus.Failed || task.status === DownloadStatus.Cancelled) && (
            <button
              onClick={() => retryDownload(task.id)}
              className="p-2 text-surface-400 hover:text-blue-400 hover:bg-surface-700/50 rounded-lg transition-colors"
              title="Retry"
            >
              <RotateCw size={18} />
            </button>
          )}

          {(isActive || task.status === DownloadStatus.Queued) && (
            <button
              onClick={() => cancelDownload(task.id)}
              className="p-2 text-surface-400 hover:text-red-400 hover:bg-surface-700/50 rounded-lg transition-colors"
              title="Cancel"
            >
              <X size={18} />
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
