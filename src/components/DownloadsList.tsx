import React from 'react';
import { useDownloadManager } from '../hooks/useDownloadManager';
import { DownloadItem } from './DownloadItem';
import { DownloadStatus } from '../types/download';
import { DownloadCloud } from 'lucide-react';

export const DownloadsList: React.FC = () => {
  const { tasks, isQueuePaused, pauseQueue, resumeQueue } = useDownloadManager();

  // ... (filters remain same) ...
  const activeTasks = tasks.filter(task =>
    task.status === DownloadStatus.Processing ||
    task.status === DownloadStatus.Paused ||
    task.status === DownloadStatus.Queued
  );

  const historyTasks = tasks.filter(task =>
    task.status === DownloadStatus.Completed ||
    task.status === DownloadStatus.Failed ||
    task.status === DownloadStatus.Cancelled
  );

  return (
    <div className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
      {/* Active Downloads Section */}
      <section>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold text-surface-100 flex items-center gap-2">
            Active Downloads
            <span className="text-xs font-normal text-surface-400 bg-surface-800 px-2 py-0.5 rounded-full border border-surface-700">
              {activeTasks.length}
            </span>
          </h2>
          <div className="flex items-center gap-2">
            {!isQueuePaused ? (
              <button
                onClick={pauseQueue}
                disabled={activeTasks.length === 0}
                className="text-xs font-medium px-3 py-1.5 rounded-lg bg-surface-800 hover:bg-surface-700 text-surface-300 hover:text-surface-100 transition-all disabled:opacity-50 disabled:cursor-not-allowed border border-surface-700"
              >
                Pause Queue
              </button>
            ) : (
              <button
                onClick={resumeQueue}
                className="text-xs font-medium px-3 py-1.5 rounded-lg bg-surface-800 hover:bg-surface-700 text-yellow-500 hover:text-yellow-400 transition-all border border-yellow-500/20 hover:border-yellow-500/40"
              >
                Resume Queue
              </button>
            )}
          </div>
        </div>

        {activeTasks.length > 0 ? (
          <div className="grid gap-3">
            {activeTasks.map(task => (
              <DownloadItem key={task.id} task={task} />
            ))}
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center p-12 border border-dashed border-surface-700/70 rounded-2xl bg-surface-800/50">
            <div className="p-4 bg-surface-900 rounded-full mb-3 border border-surface-700/50 shadow-inner">
              <DownloadCloud className="w-8 h-8 text-surface-500" />
            </div>
            <p className="text-surface-200 font-medium">No active downloads</p>
            <p className="text-sm text-surface-400">Add a URL to get started</p>
          </div>
        )}
      </section>

      {/* History Section */}
      {historyTasks.length > 0 && (
        <section>
          <h2 className="text-xl font-semibold text-surface-100 mb-4">History</h2>
          <div className="grid gap-4">
            {historyTasks.map(task => (
              <DownloadItem key={task.id} task={task} />
            ))}
          </div>
        </section>
      )}
    </div>
  );
};
