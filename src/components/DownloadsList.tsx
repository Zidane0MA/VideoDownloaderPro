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
          <h2 className="text-xl font-semibold text-white flex items-center gap-2">
            Active Downloads
            <span className="text-xs font-normal text-surface-400 bg-surface-800 px-2 py-0.5 rounded-full">
              {activeTasks.length}
            </span>
          </h2>
          <div className="flex items-center gap-2">
            {!isQueuePaused ? (
              <button 
                onClick={pauseQueue}
                disabled={activeTasks.length === 0}
                className="text-xs font-medium px-3 py-1.5 rounded-lg bg-surface-800 hover:bg-surface-700 text-surface-300 hover:text-white transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Pause Queue
              </button>
            ) : (
              <button 
                onClick={resumeQueue}
                className="text-xs font-medium px-3 py-1.5 rounded-lg bg-surface-800 hover:bg-surface-700 text-yellow-400 hover:text-yellow-300 transition-all border border-yellow-400/20 hover:border-yellow-400/40"
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
          <div className="flex flex-col items-center justify-center p-12 border border-dashed border-surface-700/50 rounded-2xl bg-surface-800/30">
            <div className="p-4 bg-surface-800 rounded-full mb-3 shadow-inner">
              <DownloadCloud className="w-8 h-8 text-surface-500" />
            </div>
            <p className="text-surface-300 font-medium">No active downloads</p>
            <p className="text-sm text-surface-500">Add a URL to get started</p>
          </div>
        )}
      </section>

      {/* History Section */}
      {historyTasks.length > 0 && (
        <section>
          <h2 className="text-xl font-semibold text-white mb-4">History</h2>
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
